// merkle_tree.rs
// A complete Merkle tree implementation in Rust.
//
// Structure:
//   - Leaves hold hashes of raw data.
//   - Each internal node holds hash(left_child || right_child).
//   - The root hash commits to the entire dataset.
//
// Use cases:
//   - Blockchain transaction verification
//   - Git object storage
//   - Distributed file integrity (IPFS, Cassandra)
//   - Certificate transparency logs
//
// Compile: rustc --edition 2021 merkle_tree.rs && ./merkle_tree
// (Uses only std — no external crates. SHA-256 is simulated via a simple
//  hash for demo; swap hash_data() for a real SHA-256 in production.)

use std::fmt;

// =============================================================================
// Hash type — 32-byte digest
// =============================================================================

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show first 8 hex chars for readability
        write!(f, "{}", &self.to_hex()[..8])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// =============================================================================
// Hashing — swap this for SHA-256 (sha2 crate) in production
// =============================================================================
// This is a simple FNV-1a inspired hash spread across 32 bytes.
// It satisfies all structural requirements of a Merkle tree for demo purposes.

fn hash_data(data: &[u8]) -> Hash {
    let mut state = [0u8; 32];
    // Use a simple but avalanche-exhibiting mix
    let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &byte in data {
        h ^= byte as u64;
        h = h.wrapping_mul(0x100000001b3);     // FNV prime
        h = h.rotate_left(13).wrapping_add(0x9e3779b97f4a7c15);
    }
    // Spread 8-byte state across 32 bytes with further mixing
    for i in 0..4 {
        let mixed = h
            .wrapping_add(i as u64 * 0x6c62272e07bb0142)
            .rotate_left(i * 7 + 5);
        let bytes = mixed.to_le_bytes();
        state[i as usize * 8..(i as usize + 1) * 8].copy_from_slice(&bytes);
    }
    Hash(state)
}

fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    // hash(left || right) — concatenate then hash
    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(&left.0);
    combined.extend_from_slice(&right.0);
    hash_data(&combined)
}

// =============================================================================
// MerkleTree
// =============================================================================
// Storage layout: a flat Vec<Hash> representing a complete binary tree.
// Index arithmetic (1-based, like a binary heap):
//
//           1           ← root
//        /     \
//       2       3
//      / \     / \
//     4   5   6   7     ← leaves start here for a 4-leaf tree
//
//  parent(i)      = i / 2
//  left_child(i)  = i * 2
//  right_child(i) = i * 2 + 1

pub struct MerkleTree {
    // nodes[0] unused; nodes[1] = root; leaves start at nodes[leaf_offset]
    nodes:       Vec<Hash>,
    leaf_count:  usize,    // number of actual leaves (before padding)
    leaf_offset: usize,    // index of the first leaf in `nodes`
}

impl MerkleTree {
    // -------------------------------------------------------------------------
    // Construction
    // -------------------------------------------------------------------------

    /// Build a Merkle tree from a slice of byte slices.
    /// Pads with duplicate of last leaf if count is not a power of two
    /// (Bitcoin-style padding — prevents second-preimage attacks vs zero-padding).
    pub fn build(data: &[&[u8]]) -> Self {
        assert!(!data.is_empty(), "cannot build Merkle tree from empty data");

        // Hash every piece of data to produce the leaves
        let raw_leaves: Vec<Hash> = data.iter().map(|d| hash_data(d)).collect();
        Self::from_hashes(raw_leaves)
    }

    /// Build directly from pre-computed leaf hashes (e.g. when leaves are
    /// already SHA-256 digests from another system).
    pub fn from_hashes(mut leaves: Vec<Hash>) -> Self {
        assert!(!leaves.is_empty());

        let leaf_count = leaves.len();

        // Round up to the next power of two — pad by duplicating the last leaf
        let padded = leaf_count.next_power_of_two();
        let last = leaves.last().unwrap().clone();
        while leaves.len() < padded {
            leaves.push(last.clone());
        }

        // Total nodes in a complete binary tree with `padded` leaves
        let total = 2 * padded;           // indices 0..total, index 0 unused
        let mut nodes = vec![Hash::zero(); total];
        let leaf_offset = padded;         // leaves occupy [padded .. 2*padded)

        // Place leaves
        for (i, leaf) in leaves.into_iter().enumerate() {
            nodes[leaf_offset + i] = leaf;
        }

        // Build internal nodes bottom-up
        let mut i = leaf_offset - 1;
        while i >= 1 {
            nodes[i] = hash_pair(&nodes[i * 2], &nodes[i * 2 + 1]);
            i -= 1;
        }

        MerkleTree { nodes, leaf_count, leaf_offset }
    }

    // -------------------------------------------------------------------------
    // Core accessors
    // -------------------------------------------------------------------------

    /// The root hash — commits to all data in the tree.
    pub fn root(&self) -> &Hash {
        &self.nodes[1]
    }

    /// Number of actual data leaves (before padding).
    pub fn len(&self) -> usize {
        self.leaf_count
    }

    pub fn is_empty(&self) -> bool {
        self.leaf_count == 0
    }

    /// Height of the tree (root = 0, leaves = height).
    pub fn height(&self) -> usize {
        self.leaf_offset.trailing_zeros() as usize
    }

    /// Return the leaf hash at position `index` (0-based).
    pub fn leaf(&self, index: usize) -> Option<&Hash> {
        if index >= self.leaf_count { return None; }
        Some(&self.nodes[self.leaf_offset + index])
    }

    // -------------------------------------------------------------------------
    // Merkle Proof (inclusion proof)
    // -------------------------------------------------------------------------

    /// Generate a proof that the leaf at `index` is included in this tree.
    ///
    /// The proof is a sequence of (hash, Side) pairs — the sibling hashes
    /// you need to walk from leaf to root.
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaf_count { return None; }

        let mut siblings = Vec::new();
        let mut i = self.leaf_offset + index;  // start at the leaf node

        while i > 1 {
            let sibling_idx = if i % 2 == 0 { i + 1 } else { i - 1 };
            let side = if i % 2 == 0 { Side::Right } else { Side::Left };
            siblings.push(ProofNode {
                hash: self.nodes[sibling_idx].clone(),
                side,
            });
            i /= 2;  // move to parent
        }

        Some(MerkleProof {
            leaf:     self.nodes[self.leaf_offset + index].clone(),
            index,
            siblings,
        })
    }

    /// Verify a proof against this tree's root.
    pub fn verify(&self, proof: &MerkleProof) -> bool {
        proof.verify(self.root())
    }

    // -------------------------------------------------------------------------
    // Update a single leaf and recompute affected nodes — O(log n)
    // -------------------------------------------------------------------------

    /// Replace the leaf at `index` with a hash of `new_data` and recompute
    /// all ancestor hashes up to the root.
    pub fn update(&mut self, index: usize, new_data: &[u8]) -> bool {
        self.update_hash(index, hash_data(new_data))
    }

    pub fn update_hash(&mut self, index: usize, new_hash: Hash) -> bool {
        if index >= self.leaf_count { return false; }

        let mut i = self.leaf_offset + index;
        self.nodes[i] = new_hash;

        // Walk up recomputing parent hashes
        while i > 1 {
            i /= 2;  // parent
            self.nodes[i] = hash_pair(&self.nodes[i * 2], &self.nodes[i * 2 + 1]);
        }
        true
    }

    // -------------------------------------------------------------------------
    // Pretty printing
    // -------------------------------------------------------------------------

    /// Print the tree level by level.
    pub fn print(&self) {
        println!("MerkleTree ({} leaves, height {}):", self.leaf_count, self.height());
        let mut level_start = 1usize;
        let mut level = 0usize;
        while level_start < self.nodes.len() {
            let level_end = (level_start * 2).min(self.nodes.len());
            let indent = " ".repeat((self.height() - level) * 2);
            print!("L{level} {indent}");
            for i in level_start..level_end {
                print!("{:?} ", self.nodes[i]);
            }
            println!();
            level_start = level_end;
            level += 1;
        }
    }

    // -------------------------------------------------------------------------
    // Diff two trees — find which leaf indices differ
    // -------------------------------------------------------------------------

    /// Compare two trees of the same size and return indices of differing leaves.
    /// Uses the tree structure to skip unchanged subtrees — O(k log n) where
    /// k is the number of changed leaves.
    pub fn diff(&self, other: &MerkleTree) -> Vec<usize> {
        assert_eq!(self.leaf_offset, other.leaf_offset, "trees must be same size");
        let mut changed = Vec::new();
        self.diff_node(other, 1, &mut changed);
        changed.retain(|&i| i < self.leaf_count); // exclude padding leaves
        changed
    }

    fn diff_node(&self, other: &MerkleTree, i: usize, changed: &mut Vec<usize>) {
        if self.nodes[i] == other.nodes[i] {
            return; // subtree is identical — skip entirely
        }
        if i >= self.leaf_offset {
            // This is a leaf
            changed.push(i - self.leaf_offset);
            return;
        }
        self.diff_node(other, i * 2,     changed);
        self.diff_node(other, i * 2 + 1, changed);
    }
}

// =============================================================================
// Merkle Proof
// =============================================================================

/// Which side is the sibling on relative to our path node?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side { Left, Right }

#[derive(Debug, Clone)]
pub struct ProofNode {
    pub hash: Hash,
    pub side: Side,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf:     Hash,
    pub index:    usize,
    pub siblings: Vec<ProofNode>,  // ordered leaf → root
}

impl MerkleProof {
    /// Recompute the root from this proof and check it matches `expected_root`.
    ///
    /// This is the core of Merkle proofs: a light client can verify membership
    /// in O(log n) without having the full dataset.
    pub fn verify(&self, expected_root: &Hash) -> bool {
        let mut current = self.leaf.clone();

        for node in &self.siblings {
            current = match node.side {
                // sibling is on the right → we are the left child
                Side::Right => hash_pair(&current, &node.hash),
                // sibling is on the left  → we are the right child
                Side::Left  => hash_pair(&node.hash, &current),
            };
        }

        &current == expected_root
    }

    /// Number of hashes in the proof (= tree height = log2(n)).
    pub fn len(&self) -> usize {
        self.siblings.len()
    }

    pub fn print(&self) {
        println!("Proof for leaf index {}:", self.index);
        println!("  leaf: {:?}", self.leaf);
        for (i, node) in self.siblings.iter().enumerate() {
            println!("  step {i}: {:?} ({:?})", node.hash, node.side);
        }
    }
}

// =============================================================================
// Sparse Merkle Tree (bonus)
// =============================================================================
// A Sparse Merkle Tree supports a fixed 256-bit key space (2^256 leaves).
// Most leaves are empty. We store only non-empty leaves in a HashMap and
// compute sibling hashes on-the-fly using precomputed empty-subtree hashes.
//
// Used in: Ethereum state trie, Diem/Aptos blockchain.

use std::collections::HashMap;

pub struct SparseMerkleTree {
    // Map from leaf key (u8 for demo — use [u8;32] in production) to leaf hash
    leaves: HashMap<u8, Hash>,
    // Precomputed hashes of empty subtrees at each depth
    // empty[0] = hash of empty leaf, empty[1] = hash(empty[0], empty[0]), …
    empty:  Vec<Hash>,
    depth:  usize,  // tree depth (8 for u8 key space = 256 leaves)
}

impl SparseMerkleTree {
    pub fn new(depth: usize) -> Self {
        // Precompute empty subtree hashes bottom-up
        let mut empty = vec![hash_data(b"")];  // empty leaf
        for i in 0..depth {
            let h = hash_pair(&empty[i], &empty[i]);
            empty.push(h);
        }
        SparseMerkleTree { leaves: HashMap::new(), empty, depth }
    }

    /// Insert or update a leaf.
    pub fn insert(&mut self, key: u8, data: &[u8]) {
        self.leaves.insert(key, hash_data(data));
    }

    /// Remove a leaf (revert to empty).
    pub fn remove(&mut self, key: u8) {
        self.leaves.remove(&key);
    }

    /// Compute the root hash. O(n log n) where n = number of non-empty leaves.
    pub fn root(&self) -> Hash {
        self.subtree_hash(0u16, self.depth)
    }

    // Recursively compute hash of the subtree rooted at [prefix..prefix+2^depth)
    fn subtree_hash(&self, prefix: u16, depth: usize) -> Hash {
        if depth == 0 {
            // Leaf level
            return self.leaves
                .get(&(prefix as u8))
                .cloned()
                .unwrap_or_else(|| self.empty[0].clone());
        }
        let half = 1u16 << (depth - 1);
        let left  = self.subtree_hash(prefix,        depth - 1);
        let right = self.subtree_hash(prefix + half,  depth - 1);
        // If both children are empty, return precomputed empty hash
        if left == self.empty[depth - 1] && right == self.empty[depth - 1] {
            return self.empty[depth].clone();
        }
        hash_pair(&left, &right)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> MerkleTree {
        MerkleTree::build(&[b"alice", b"bob", b"carol", b"dave"])
    }

    // ---- Basic structure ----

    #[test]
    fn root_is_deterministic() {
        let t1 = sample_tree();
        let t2 = sample_tree();
        assert_eq!(t1.root(), t2.root());
    }

    #[test]
    fn different_data_different_root() {
        let t1 = MerkleTree::build(&[b"alice", b"bob"]);
        let t2 = MerkleTree::build(&[b"alice", b"eve"]);
        assert_ne!(t1.root(), t2.root());
    }

    #[test]
    fn height_correct() {
        // 4 leaves → height 2
        assert_eq!(sample_tree().height(), 2);
        // 1 leaf → height 1 (padded to 2 leaves)
        assert_eq!(MerkleTree::build(&[b"solo"]).height(), 1);
        // 8 leaves → height 3
        let data: Vec<&[u8]> = (0u8..8).map(|i| -> &[u8] {
            // leak to get 'static — fine for tests
            Box::leak(Box::new([i]))
        }).collect();
        assert_eq!(MerkleTree::build(&data).height(), 3);
    }

    #[test]
    fn leaf_accessor() {
        let t = sample_tree();
        assert!(t.leaf(0).is_some());
        assert!(t.leaf(3).is_some());
        assert!(t.leaf(4).is_none());   // out of range
    }

    // ---- Proof generation and verification ----

    #[test]
    fn proof_verifies_all_leaves() {
        let t = sample_tree();
        for i in 0..t.len() {
            let proof = t.proof(i).expect("proof should exist");
            assert!(t.verify(&proof), "proof for leaf {i} failed");
        }
    }

    #[test]
    fn tampered_proof_fails() {
        let t = sample_tree();
        let mut proof = t.proof(0).unwrap();
        // Tamper with the leaf hash
        proof.leaf = hash_data(b"mallory");
        assert!(!t.verify(&proof));
    }

    #[test]
    fn proof_against_wrong_root_fails() {
        let t1 = sample_tree();
        let t2 = MerkleTree::build(&[b"x", b"y", b"z", b"w"]);
        let proof = t1.proof(0).unwrap();
        // Proof from t1 should not verify against t2's root
        assert!(!proof.verify(t2.root()));
    }

    #[test]
    fn proof_length_is_log2() {
        let t = sample_tree();                  // 4 leaves → height 2
        assert_eq!(t.proof(0).unwrap().len(), 2);

        let big: Vec<&[u8]> = vec![b"a"; 16];
        let t16 = MerkleTree::build(&big);       // 16 leaves → height 4
        assert_eq!(t16.proof(0).unwrap().len(), 4);
    }

    // ---- Update ----

    #[test]
    fn update_changes_root() {
        let mut t = sample_tree();
        let old_root = t.root().clone();
        t.update(1, b"updated_bob");
        assert_ne!(t.root(), &old_root);
    }

    #[test]
    fn update_proof_verifies_after_update() {
        let mut t = sample_tree();
        t.update(2, b"updated_carol");
        let proof = t.proof(2).unwrap();
        assert!(t.verify(&proof));
    }

    #[test]
    fn update_unrelated_leaf_unchanged() {
        let mut t = sample_tree();
        let leaf0_before = t.leaf(0).unwrap().clone();
        t.update(3, b"updated_dave");
        assert_eq!(t.leaf(0).unwrap(), &leaf0_before);
    }

    // ---- Diff ----

    #[test]
    fn diff_detects_changed_leaves() {
        let t1 = sample_tree();
        let mut t2 = sample_tree();
        t2.update(1, b"modified");
        t2.update(3, b"also_modified");
        let diffs = t1.diff(&t2);
        assert_eq!(diffs, vec![1, 3]);
    }

    #[test]
    fn diff_identical_trees_is_empty() {
        let t1 = sample_tree();
        let t2 = sample_tree();
        assert!(t1.diff(&t2).is_empty());
    }

    // ---- Non-power-of-two leaf count ----

    #[test]
    fn odd_leaf_count_proofs_verify() {
        let t = MerkleTree::build(&[b"a", b"b", b"c"]); // 3 leaves → padded to 4
        assert_eq!(t.len(), 3);
        for i in 0..3 {
            assert!(t.verify(&t.proof(i).unwrap()), "leaf {i} failed");
        }
        // Index 3 is padding — no proof
        assert!(t.proof(3).is_none());
    }

    // ---- Sparse Merkle Tree ----

    #[test]
    fn sparse_empty_root_deterministic() {
        let t1 = SparseMerkleTree::new(8);
        let t2 = SparseMerkleTree::new(8);
        assert_eq!(t1.root(), t2.root());
    }

    #[test]
    fn sparse_insert_changes_root() {
        let mut t = SparseMerkleTree::new(8);
        let before = t.root();
        t.insert(42, b"hello");
        assert_ne!(t.root(), before);
    }

    #[test]
    fn sparse_remove_restores_root() {
        let mut t = SparseMerkleTree::new(8);
        let empty_root = t.root();
        t.insert(42, b"hello");
        t.remove(42);
        assert_eq!(t.root(), empty_root);
    }
}

// =============================================================================
// Demo
// =============================================================================

fn main() {
    println!("============================================================");
    println!("  Merkle Tree Demo");
    println!("============================================================");

    // ---- 1. Build a tree ----
    let data: &[&[u8]] = &[b"alice:100", b"bob:250", b"carol:75", b"dave:500"];
    let mut tree = MerkleTree::build(data);

    println!("\n[1] Building tree from {} entries:", data.len());
    tree.print();
    println!("Root: {}", tree.root());

    // ---- 2. Generate and verify a proof ----
    println!("\n[2] Proof for 'bob' (index 1):");
    let proof = tree.proof(1).unwrap();
    proof.print();
    println!("Valid: {}", tree.verify(&proof));

    // ---- 3. Tampered proof fails ----
    println!("\n[3] Tampered leaf proof:");
    let mut bad_proof = proof.clone();
    bad_proof.leaf = hash_data(b"mallory:999");
    println!("Tampered valid: {}", tree.verify(&bad_proof));

    // ---- 4. Update a leaf ----
    println!("\n[4] Update bob:250 → bob:300:");
    let root_before = tree.root().clone();
    tree.update(1, b"bob:300");
    println!("Root changed: {}", tree.root() != &root_before);
    println!("New root: {}", tree.root());

    // Proof of updated leaf verifies with new root
    let new_proof = tree.proof(1).unwrap();
    println!("Updated proof valid: {}", tree.verify(&new_proof));

    // ---- 5. Diff two trees ----
    println!("\n[5] Diff two trees:");
    let old_tree = MerkleTree::build(data);
    let mut new_tree = MerkleTree::build(data);
    new_tree.update(0, b"alice:150");
    new_tree.update(3, b"dave:600");
    let diffs = old_tree.diff(&new_tree);
    println!("Changed leaf indices: {:?}", diffs); // [0, 3]

    // ---- 6. Non-power-of-two leaves ----
    println!("\n[6] Tree with 5 leaves (padded to 8):");
    let t5 = MerkleTree::build(&[b"a", b"b", b"c", b"d", b"e"]);
    println!("Height: {}", t5.height());
    println!("Root:   {}", t5.root());
    for i in 0..5 {
        let ok = t5.verify(&t5.proof(i).unwrap());
        println!("  proof[{i}] valid: {ok}");
    }

    // ---- 7. Sparse Merkle Tree ----
    println!("\n[7] Sparse Merkle Tree:");
    let mut smt = SparseMerkleTree::new(8);
    println!("Empty root: {:?}", smt.root());
    smt.insert(10, b"account:10");
    smt.insert(200, b"account:200");
    println!("After 2 inserts: {:?}", smt.root());
    smt.remove(10);
    println!("After remove 10: {:?}", smt.root());

    println!("\n============================================================");
    println!("  All demos complete.");
    println!("============================================================");
}