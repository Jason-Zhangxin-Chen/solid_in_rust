use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use crossbeam_queue::SegQueue;
use crossbeam_skiplist::SkipMap;
use crossbeam_utils::atomic::AtomicCell;
use dashmap::DashMap;
use sha2::{Sha256, Digest};

// ---------- Basic Types ----------
type ProductID = u128;
type Price = u128;
type Id = u128;
type Quantity = u64;
type Hash = [u8; 32];

#[derive(Clone, PartialEq, Debug)]
enum Side {
    Ask,
    Bid,
}

/*
#[derive(Clone, Debug)]
struct Order {
    id: Id,
    side: Side,
    price: Price,
    quantity: Quantity,
}

// ---------- Sparse Merkle Tree ----------
struct SparseMerkleTree {
    nodes: HashMap<(usize, u64), Hash>, // (depth, index) -> hash
    default_nodes: [Hash; 65],          // precomputed hashes of empty subtrees
}

impl SparseMerkleTree {
    fn new() -> Self {
        let mut default_nodes = [[0u8; 32]; 65];
        // default_nodes[0] is already an all-zero leaf
        for d in 1..=64 {
            let mut hasher = Sha256::new();
            hasher.update(&default_nodes[d - 1]);
            hasher.update(&default_nodes[d - 1]);
            default_nodes[d] = hasher.finalize().into();
        }
        Self {
            nodes: HashMap::new(),
            default_nodes,
        }
    }

    fn get_node(&self, depth: usize, index: u64) -> Hash {
        *self
            .nodes
            .get(&(depth, index))
            .unwrap_or(&self.default_nodes[depth])
    }

    /// Set a leaf value, update all ancestors, return the new root.
    fn set_leaf(&mut self, leaf_index: u64, leaf_hash: Hash) -> Hash {
        let mut current_hash = leaf_hash;
        let mut current_idx = leaf_index;

        for depth in 0..64 {
            let sibling_idx = current_idx ^ 1;
            let sibling_hash = self.get_node(depth, sibling_idx);

            let parent_hash = if current_idx & 1 == 0 {
                hash_pair(current_hash, sibling_hash)
            } else {
                hash_pair(sibling_hash, current_hash)
            };

            let parent_depth = depth + 1;
            let parent_idx = current_idx >> 1;
            self.nodes.insert((parent_depth, parent_idx), parent_hash);

            current_hash = parent_hash;
            current_idx = parent_idx;
        }
        current_hash // root at depth 64, index 0
    }

    fn root(&self) -> Hash {
        self.get_node(64, 0)
    }
}

fn hash_pair(left: Hash, right: Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(&left);
    hasher.update(&right);
    hasher.finalize().into()
}

fn hash_order(order: &Order) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(&order.id.to_le_bytes());
    hasher.update(&[match order.side {
        Side::Bid => 0u8,
        Side::Ask => 1u8,
    }]);
    hasher.update(&order.price.to_le_bytes());
    hasher.update(&order.quantity.to_le_bytes());
    hasher.finalize().into()
}

// ---------- PriceLevel (with incremental root) ----------
struct PriceLevel {
    price: Price,
    orders: DashMap<Id, Order>,
    fifo: SegQueue<Order>,
    total_quantity: AtomicU64,
    root: AtomicCell<Hash>,            // current state root (atomically readable)
    tree: Mutex<SparseMerkleTree>,     // guards all tree mutations
    seq_counter: AtomicU64,            // monotonically increasing insertion index
    order_seq: DashMap<Id, u64>,       // order id -> insertion index
}

impl PriceLevel {
    fn new(price: Price) -> Self {
        Self {
            price,
            orders: DashMap::new(),
            fifo: SegQueue::new(),
            total_quantity: AtomicU64::new(0),
            root: AtomicCell::new([0u8; 32]),
            tree: Mutex::new(SparseMerkleTree::new()),
            seq_counter: AtomicU64::new(0),
            order_seq: DashMap::new(),
        }
    }

    /// Add a new resting order at this price level.
    fn add_order(&self, order: Order) {
        // 1. Assign insertion index (time priority)
        let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst);
        self.order_seq.insert(order.id, seq);

        // 2. Push into FIFO for matching
        self.fifo.push(order.clone());

        // 3. Store active order content
        self.orders.insert(order.id, order.clone());

        // 4. Update total quantity
        self.total_quantity.fetch_add(order.quantity, Ordering::SeqCst);

        // 5. Insert leaf into sparse Merkle tree → updates root
        let leaf_hash = hash_order(&order);
        let mut tree = self.tree.lock().unwrap();
        let new_root = tree.set_leaf(seq, leaf_hash);
        self.root.store(new_root);
    }

    /// Pop the oldest *active* order from the FIFO.
    /// Returns `None` if no active order remains.
    fn pop_active_order(&self) -> Option<Order> {
        loop {
            let order = self.fifo.pop().ok()?; // empty queue → None
            // Try to remove it from the active orders map.
            if let Some((_, removed_order)) = self.orders.remove(&order.id) {
                self.total_quantity.fetch_sub(removed_order.quantity, Ordering::SeqCst);
                // Zero out its leaf in the tree.
                if let Some((_, seq)) = self.order_seq.remove(&removed_order.id) {
                    let mut tree = self.tree.lock().unwrap();
                    let new_root = tree.set_leaf(seq, [0u8; 32]);
                    self.root.store(new_root);
                }
                return Some(removed_order);
            }
            // else: the order was already cancelled – skip and continue.
        }
    }

    /// Cancel an arbitrary order by its id.
    /// Returns the removed order if it existed and was active.
    fn cancel_order(&self, order_id: Id) -> Option<Order> {
        let order = self.orders.remove(&order_id)?;
        self.total_quantity.fetch_sub(order.quantity, Ordering::SeqCst);
        if let Some(seq) = self.order_seq.remove(&order_id) {
            let mut tree = self.tree.lock().unwrap();
            let new_root = tree.set_leaf(seq.1, [0u8; 32]);
            self.root.store(new_root);
        }
        Some(order)
    }

    fn get_root(&self) -> Hash {
        self.root.load()
    }
}

// ---------- OrderBook (with incremental root) ----------
struct OrderBook {
    product: ProductID,
    asks: SkipMap<Price, Arc<PriceLevel>>,
    bids: SkipMap<Price, Arc<PriceLevel>>,
    indices: DashMap<Id, (Price, Side)>, // id -> (price, side) for fast lookup
    root: AtomicCell<Hash>,
}

impl OrderBook {
    fn new(product: ProductID) -> Self {
        Self {
            product,
            asks: SkipMap::new(),
            bids: SkipMap::new(),
            indices: DashMap::new(),
            root: AtomicCell::new([0u8; 32]),
        }
    }

    /// Helper: get or create the PriceLevel for a given price and side.
    fn get_or_create_level(&self, side: &Side, price: Price) -> Arc<PriceLevel> {
        let map = match side {
            Side::Ask => &self.asks,
            Side::Bid => &self.bids,
        };
        map.get_or_insert(price, Arc::new(PriceLevel::new(price)))
            .value()
            .clone()
    }

    /// Place a new order.
    fn add_order(&self, order: Order) {
        let level = self.get_or_create_level(&order.side, order.price);
        self.indices.insert(order.id, (order.price, order.side.clone()));
        level.add_order(order);
        self.recompute_root();
    }

    /// Cancel an order by its id.
    fn cancel_order(&self, order_id: Id) -> Option<Order> {
        let (price, side) = self.indices.remove(&order_id)?;
        let level = match side {
            Side::Ask => self.asks.get(&price)?.value().clone(),
            Side::Bid => self.bids.get(&price)?.value().clone(),
        };
        let result = level.cancel_order(order_id);
        if result.is_some() {
            self.recompute_root();
        }
        result
    }

    /// Recompute the order‑book state root from the sorted price levels.
    fn recompute_root(&self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.product.to_le_bytes());

        // Asks: ascending price (lowest sell first)
        for entry in self.asks.iter() {
            let price = entry.key();
            let level = entry.value();
            hasher.update(&price.to_le_bytes());
            hasher.update(&level.get_root());
        }

        // Bids: descending price (highest buy first)
        // collect keys, sort descending, then iterate
        let mut bid_entries: Vec<(Price, Arc<PriceLevel>)> = self
            .bids
            .iter()
            .map(|e| (*e.key(), e.value().clone()))
            .collect();
        bid_entries.sort_by(|a, b| b.0.cmp(&a.0));
        for (price, level) in bid_entries {
            hasher.update(&price.to_le_bytes());
            hasher.update(&level.get_root());
        }

        self.root.store(hasher.finalize().into());
    }

    fn get_root(&self) -> Hash {
        self.root.load()
    }
}

// ---------- BookKeeper (global state root) ----------
struct BookKeeper {
    books: SkipMap<ProductID, Arc<OrderBook>>,
    root: AtomicCell<Hash>,
}

impl BookKeeper {
    fn new() -> Self {
        Self {
            books: SkipMap::new(),
            root: AtomicCell::new([0u8; 32]),
        }
    }

    fn get_or_create_book(&self, product: ProductID) -> Arc<OrderBook> {
        self.books
            .get_or_insert(product, Arc::new(OrderBook::new(product)))
            .value()
            .clone()
    }

    /// Add an order to a given product's book.
    fn add_order(&self, product: ProductID, order: Order) {
        let book = self.get_or_create_book(product);
        book.add_order(order);
        self.recompute_root();
    }

    /// Cancel an order from a given product's book.
    fn cancel_order(&self, product: ProductID, order_id: Id) -> Option<Order> {
        let book = self.books.get(&product)?.value().clone();
        let result = book.cancel_order(order_id);
        if result.is_some() {
            self.recompute_root();
        }
        result
    }

    /// Recompute the global state root from all product order books.
    fn recompute_root(&self) {
        let mut hasher = Sha256::new();
        for entry in self.books.iter() {
            let product_id = entry.key();
            let book = entry.value();
            hasher.update(&product_id.to_le_bytes());
            hasher.update(&book.get_root());
        }
        self.root.store(hasher.finalize().into());
    }

    fn get_root(&self) -> Hash {
        self.root.load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_root_changes() {
        let keeper = BookKeeper::new();
        let product = 1u128;

        let o1 = Order { id: 100, side: Side::Bid, price: 100, quantity: 10 };
        let o2 = Order { id: 101, side: Side::Bid, price: 100, quantity: 5 };
        let o3 = Order { id: 102, side: Side::Ask, price: 101, quantity: 7 };

        // Initially empty
        let root_empty = keeper.get_root();

        // Add first order
        keeper.add_order(product, o1.clone());
        let root1 = keeper.get_root();
        assert_ne!(root1, root_empty);

        // Add second order at same price level
        keeper.add_order(product, o2.clone());
        let root2 = keeper.get_root();
        assert_ne!(root2, root1);

        // Cancel second order
        keeper.cancel_order(product, 101);
        let root3 = keeper.get_root();
        assert_ne!(root3, root2);
        // Now only order 100 remains, should equal root1 (same set)
        assert_eq!(root3, root1, "Cancel should revert to single-order root");

        // Add ask order
        keeper.add_order(product, o3);
        let root4 = keeper.get_root();
        assert_ne!(root4, root1);

        // Cancel the last remaining bid order -> empty bid side
        keeper.cancel_order(product, 100);
        let root5 = keeper.get_root();
        assert_ne!(root5, root4);

        // Cancel the ask -> completely empty book
        keeper.cancel_order(product, 102);
        let root6 = keeper.get_root();
        assert_eq!(root6, root_empty, "Empty book should have the same root as initial");
    }
}*/
