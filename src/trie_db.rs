// ethereum_trie.rs
//
// A complete Ethereum-style state database built on a
// Modified Merkle Patricia Trie (MPT).
//
// Architecture:
//
//   StateDB                          ← top-level API
//     └─ State Trie (MPT)            ← address → AccountState
//          ├─ Account 0xABCD...
//          │    ├─ nonce, balance, code_hash
//          │    └─ Storage Trie (MPT) ← slot → value
//          ├─ Account 0xDEAD...
//          │    └─ Storage Trie (MPT)
//          └─ …
//
//   KVStore (HashMap<Hash, Bytes>)   ← all trie nodes persisted by hash
//
//   State Root = MPT root hash over all accounts
//              = Keccak256 of the RLP-encoded root node
//
// Key Ethereum MPT concepts implemented:
//   - Nibble paths     (bytes split into 4-bit nibbles)
//   - Hex-prefix encoding (distinguish leaf vs extension + odd/even length)
//   - Three node kinds: Leaf, Extension, Branch (16 children + value slot)
//   - Inline encoding  (nodes ≤ 32 bytes stored inline, not by hash)
//   - Node caching     (dirty nodes collected before flush to KV store)
//
// Simplifications vs production (noted inline):
//   - Keccak256 replaced by a strong mixing function (swap hash_bytes() for sha3)
//   - RLP is hand-rolled (swap encode_node() for the `rlp` crate)
//   - No proof-of-work / EVM / networking

/*

#![allow(dead_code, clippy::all)]

use std::collections::HashMap;
use std::fmt;

fn main() {
    demo();
}

// =============================================================================
// § 1.  Hash — 32-byte Keccak256 digest (simulated)
// =============================================================================

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct H256([u8; 32]);

impl H256 {
    pub const ZERO: H256 = H256([0u8; 32]);

    pub fn from_bytes(b: &[u8; 32]) -> Self { H256(*b) }
    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
    pub fn short(&self) -> String { self.to_hex()[..8].to_string() }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}…", self.short())
    }
}
impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

/// Simulated Keccak256.
/// PRODUCTION: replace body with `keccak256` from the `sha3` or `ethers` crate.
pub fn keccak256(data: &[u8]) -> H256 {
    let mut h: u64 = 0x6c62272e07bb0142;
    let mut state = [0u64; 4];
    state[0] = 0xcbf29ce484222325;
    state[1] = 0x9e3779b97f4a7c15;
    state[2] = 0x517cc1b727220a95;
    state[3] = 0xbf58476d1ce4e5b9;

    for (i, &byte) in data.iter().enumerate() {
        let lane = i % 4;
        state[lane] ^= (byte as u64).wrapping_mul(0x100000001b3);
        state[lane] = state[lane]
            .rotate_left(((i % 13) + 5) as u32)
            .wrapping_add(state[(lane + 1) % 4]);
        h ^= state[lane];
    }
    // Squeeze 32 bytes from 4 u64s
    let mut out = [0u8; 32];
    for (i, lane) in state.iter().enumerate() {
        let mixed = lane
            .wrapping_add(h)
            .rotate_left((i * 7 + 11) as u32)
            .wrapping_mul(0x94d049bb133111eb);
        out[i * 8..(i + 1) * 8].copy_from_slice(&mixed.to_le_bytes());
    }
    H256(out)
}

// =============================================================================
// § 2.  Nibbles — 4-bit path units
// =============================================================================
// Ethereum encodes trie keys as sequences of nibbles (hex digits 0-15).
// A 20-byte address becomes a 40-nibble path.
// A 32-byte storage slot becomes a 64-nibble path.

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Nibbles(Vec<u8>);  // each element is 0..=15

impl Nibbles {
    /// Convert a byte slice to nibbles (high nibble first).
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut v = Vec::with_capacity(bytes.len() * 2);
        for &b in bytes {
            v.push(b >> 4);
            v.push(b & 0x0f);
        }
        Nibbles(v)
    }

    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn get(&self, i: usize) -> u8 { self.0[i] }

    /// How many leading nibbles do `self` and `other` share?
    pub fn common_prefix_len(&self, other: &Nibbles) -> usize {
        self.0.iter().zip(other.0.iter()).take_while(|(a, b)| a == b).count()
    }

    /// Nibbles after position `pos`.
    pub fn slice(&self, pos: usize) -> Nibbles {
        Nibbles(self.0[pos..].to_vec())
    }

    /// First nibble.
    pub fn first(&self) -> Option<u8> { self.0.first().copied() }

    // ---- Hex-prefix encoding (EIP-159) ----
    //
    // Flag byte prefix encodes:
    //   bit 5 (0x20): is_leaf
    //   bit 4 (0x10): odd number of nibbles
    //
    //  Even extension: 0x00 <paired nibbles>
    //  Odd  extension: 0x1n <remaining nibbles>  (n = first nibble)
    //  Even leaf:      0x20 <paired nibbles>
    //  Odd  leaf:      0x3n <remaining nibbles>

    pub fn encode_hp(&self, is_leaf: bool) -> Vec<u8> {
        let flag: u8 = if is_leaf { 0x20 } else { 0x00 };
        if self.0.len() % 2 == 0 {
            // Even: prefix byte 0x00 or 0x20, then pairs
            let mut out = vec![flag];
            for pair in self.0.chunks(2) {
                out.push((pair[0] << 4) | pair[1]);
            }
            out
        } else {
            // Odd: prefix nibble 0x1 or 0x3 combined with first nibble
            let prefix_nibble = flag | 0x10;
            let mut out = vec![(prefix_nibble << 4) | self.0[0]];
            for pair in self.0[1..].chunks(2) {
                out.push((pair[0] << 4) | pair[1]);
            }
            out
        }
    }

    pub fn decode_hp(encoded: &[u8]) -> (Nibbles, bool) {
        let flag = encoded[0] >> 4;
        let is_leaf = flag >= 2;
        let odd = flag & 1 == 1;
        let mut nibs = Vec::new();
        if odd {
            nibs.push(encoded[0] & 0x0f);
        }
        for &b in &encoded[1..] {
            nibs.push(b >> 4);
            nibs.push(b & 0x0f);
        }
        (Nibbles(nibs), is_leaf)
    }
}

// =============================================================================
// § 3.  Trie Node
// =============================================================================

/// A reference to a child node — either inlined (small) or stored by hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeRef {
    Empty,
    Hash(H256),             // stored in KVStore, fetch before use
    Inline(Box<Node>),      // encoded ≤ 32 bytes → skip the KV round-trip
}

/// The three MPT node kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// Leaf: (remaining_path, value)
    /// Stores the actual value bytes at a complete path.
    Leaf {
        path:  Nibbles,
        value: Vec<u8>,
    },

    /// Extension: (shared_prefix, next)
    /// Collapses a sequence of single-child branch nodes into one hop.
    Extension {
        prefix: Nibbles,
        next:   NodeRef,
    },

    /// Branch: 16 children (one per nibble 0-f) + optional value at this node.
    Branch {
        children: [NodeRef; 16],
        value:    Option<Vec<u8>>,
    },
}

impl Node {
    fn empty_branch() -> Self {
        Node::Branch {
            children: Default::default(),
            value: None,
        }
    }
}

// Default for NodeRef (needed for array init)
impl Default for NodeRef {
    fn default() -> Self { NodeRef::Empty }
}

// =============================================================================
// § 4.  Minimal RLP Encoder / Decoder
// =============================================================================
// PRODUCTION: use the `rlp` crate which is battle-tested and spec-compliant.

fn rlp_encode_bytes(b: &[u8]) -> Vec<u8> {
    if b.len() == 1 && b[0] < 0x80 {
        b.to_vec()                                 // single byte, no prefix
    } else if b.len() <= 55 {
        let mut out = vec![0x80 + b.len() as u8];
        out.extend_from_slice(b);
        out
    } else {
        let len_bytes = encode_length(b.len());
        let mut out = vec![0xb7 + len_bytes.len() as u8];
        out.extend_from_slice(&len_bytes);
        out.extend_from_slice(b);
        out
    }
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload: Vec<u8> = items.iter().flat_map(|i| i.clone()).collect();
    if payload.len() <= 55 {
        let mut out = vec![0xc0 + payload.len() as u8];
        out.extend_from_slice(&payload);
        out
    } else {
        let len_bytes = encode_length(payload.len());
        let mut out = vec![0xf7 + len_bytes.len() as u8];
        out.extend_from_slice(&len_bytes);
        out.extend_from_slice(&payload);
        out
    }
}

fn encode_length(n: usize) -> Vec<u8> {
    if n < 0x100       { vec![n as u8] }
    else if n < 0x1_0000 { vec![(n >> 8) as u8, n as u8] }
    else               { vec![(n >> 16) as u8, (n >> 8) as u8, n as u8] }
}

fn rlp_decode_list(data: &[u8]) -> Vec<Vec<u8>> {
    let mut items = Vec::new();
    let payload = if data[0] <= 0xf7 {
        &data[1..]
    } else {
        let ll = (data[0] - 0xf7) as usize;
        &data[1 + ll..]
    };
    let mut pos = 0;
    while pos < payload.len() {
        let (item, consumed) = rlp_decode_item(&payload[pos..]);
        items.push(item);
        pos += consumed;
    }
    items
}

fn rlp_decode_item(data: &[u8]) -> (Vec<u8>, usize) {
    let first = data[0];
    if first < 0x80 {
        (vec![first], 1)
    } else if first <= 0xb7 {
        let len = (first - 0x80) as usize;
        (data[1..1 + len].to_vec(), 1 + len)
    } else if first <= 0xbf {
        let ll  = (first - 0xb7) as usize;
        let len = decode_usize(&data[1..1 + ll]);
        (data[1 + ll..1 + ll + len].to_vec(), 1 + ll + len)
    } else if first <= 0xf7 {
        let len = (first - 0xc0) as usize;
        (data[..1 + len].to_vec(), 1 + len)
    } else {
        let ll  = (first - 0xf7) as usize;
        let len = decode_usize(&data[1..1 + ll]);
        (data[..1 + ll + len].to_vec(), 1 + ll + len)
    }
}

fn decode_usize(b: &[u8]) -> usize {
    b.iter().fold(0, |acc, &x| (acc << 8) | x as usize)
}

// =============================================================================
// § 5.  Node Encoding / Decoding
// =============================================================================

/// Encode a node to its canonical byte representation.
/// Returns (encoded_bytes, should_hash) — nodes ≥ 32 bytes are stored by hash.
pub fn encode_node(node: &Node) -> Vec<u8> {
    match node {
        Node::Leaf { path, value } => {
            let hp = path.encode_hp(true);
            rlp_encode_list(&[rlp_encode_bytes(&hp), rlp_encode_bytes(value)])
        }
        Node::Extension { prefix, next } => {
            let hp   = prefix.encode_hp(false);
            let next_enc = encode_node_ref(next);
            rlp_encode_list(&[rlp_encode_bytes(&hp), next_enc])
        }
        Node::Branch { children, value } => {
            let mut items: Vec<Vec<u8>> = children
                .iter()
                .map(encode_node_ref)
                .collect();
            items.push(match value {
                Some(v) => rlp_encode_bytes(v),
                None    => rlp_encode_bytes(&[]),
            });
            rlp_encode_list(&items)
        }
    }
}

fn encode_node_ref(r: &NodeRef) -> Vec<u8> {
    match r {
        NodeRef::Empty       => rlp_encode_bytes(&[]),
        NodeRef::Hash(h)     => rlp_encode_bytes(h.as_bytes()),
        NodeRef::Inline(n)   => encode_node(n),
    }
}

pub fn decode_node(data: &[u8]) -> Node {
    let items = rlp_decode_list(data);
    match items.len() {
        // 2-item list → Leaf or Extension
        2 => {
            let (path, is_leaf) = Nibbles::decode_hp(&items[0]);
            if is_leaf {
                Node::Leaf { path, value: items[1].clone() }
            } else {
                let next = decode_node_ref(&items[1]);
                Node::Extension { prefix: path, next }
            }
        }
        // 17-item list → Branch
        17 => {
            let mut children: [NodeRef; 16] = Default::default();
            for i in 0..16 {
                children[i] = decode_node_ref(&items[i]);
            }
            let value = if items[16].is_empty() {
                None
            } else {
                Some(items[16].clone())
            };
            Node::Branch { children, value }
        }
        _ => panic!("invalid node encoding: {} items", items.len()),
    }
}

fn decode_node_ref(data: &[u8]) -> NodeRef {
    if data.is_empty() { return NodeRef::Empty; }
    if data.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(data);
        return NodeRef::Hash(H256(arr));
    }
    // Inline node — decode recursively
    if data[0] >= 0xc0 {
        NodeRef::Inline(Box::new(decode_node(data)))
    } else {
        // Raw 32-byte hash stored as RLP bytes
        if data.len() > 1 && data.len() <= 33 {
            let bytes = &data[1..];
            if bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(bytes);
                return NodeRef::Hash(H256(arr));
            }
        }
        NodeRef::Empty
    }
}

// =============================================================================
// § 6.  KV Store  (in-memory; swap for RocksDB / LevelDB in production)
// =============================================================================

#[derive(Default, Debug)]
pub struct KVStore {
    data: HashMap<H256, Vec<u8>>,
    writes: usize,
    reads:  usize,
}

impl KVStore {
    pub fn new() -> Self { Self::default() }

    pub fn put(&mut self, key: H256, value: Vec<u8>) {
        self.writes += 1;
        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: &H256) -> Option<&Vec<u8>> {
        self.reads += 1;
        self.data.get(key)
    }

    pub fn contains(&self, key: &H256) -> bool {
        self.data.contains_key(key)
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.data.len(), self.writes, self.reads)
    }
}

// =============================================================================
// § 7.  Modified Merkle Patricia Trie (MPT)
// =============================================================================

pub struct MerkleTrie {
    root:  NodeRef,
    store: KVStore,        // shared backing store (pass Arc<Mutex<>> for threads)
}

impl MerkleTrie {
    pub fn new() -> Self {
        MerkleTrie { root: NodeRef::Empty, store: KVStore::new() }
    }

    // ---- Public API ----

    /// The state root hash (empty trie = zero hash).
    pub fn root_hash(&self) -> H256 {
        match &self.root {
            NodeRef::Empty    => H256::ZERO,
            NodeRef::Hash(h)  => h.clone(),
            NodeRef::Inline(n) => keccak256(&encode_node(n)),
        }
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let path = Nibbles::from_bytes(key);
        self.get_node(&self.root.clone(), &path)
    }

    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let path = Nibbles::from_bytes(key);
        let old_root = self.root.clone();
        let new_root = self.insert_node(old_root, &path, 0, value);
        self.root = self.commit_node(new_root);
    }

    pub fn delete(&mut self, key: &[u8]) -> bool {
        let path = Nibbles::from_bytes(key);
        let old_root = self.root.clone();
        if let Some(new_root) = self.delete_node(old_root, &path, 0) {
            self.root = self.commit_node(new_root);
            true
        } else {
            false
        }
    }

    // ---- get ----

    fn get_node(&mut self, node_ref: &NodeRef, path: &Nibbles) -> Option<Vec<u8>> {
        let node = self.resolve(node_ref)?;
        match node {
            Node::Leaf { path: lpath, value } => {
                if lpath == *path { Some(value) } else { None }
            }
            Node::Extension { prefix, next } => {
                let cp = prefix.common_prefix_len(path);
                if cp < prefix.len() { return None; }
                self.get_node(&next.clone(), &path.slice(cp))
            }
            Node::Branch { children, value } => {
                if path.is_empty() {
                    return value;
                }
                let idx = path.get(0) as usize;
                self.get_node(&children[idx].clone(), &path.slice(1))
            }
        }
    }

    // ---- insert ----

    fn insert_node(&mut self, node_ref: NodeRef, path: &Nibbles, depth: usize, value: Vec<u8>) -> Node {
        match self.resolve(&node_ref) {
            None => {
                // Empty slot → create leaf with remaining path
                Node::Leaf { path: path.slice(depth), value }
            }
            Some(node) => match node {
                Node::Leaf { path: lpath, value: lvalue } => {
                    let cp = lpath.common_prefix_len(&path.slice(depth));
                    if cp == lpath.len() && cp == path.slice(depth).len() {
                        // Same path → update value
                        return Node::Leaf { path: lpath, value };
                    }
                    // Fork into a Branch, possibly with a shared Extension
                    let mut branch = Node::empty_branch();
                    let remaining_depth = depth + cp;

                    // Place existing leaf
                    if cp < lpath.len() {
                        let old_leaf = Node::Leaf {
                            path:  lpath.slice(cp + 1),
                            value: lvalue,
                        };
                        let old_idx = lpath.get(cp) as usize;
                        if let Node::Branch { ref mut children, .. } = branch {
                            children[old_idx] = self.commit_node(old_leaf);
                        }
                    } else {
                        // Old leaf path is exhausted — its value lives in the branch
                        if let Node::Branch { ref mut value: ref mut bv, .. } = branch {
                            *bv = Some(lvalue);
                        }
                    }

                    // Place new value
                    let new_path = path.slice(depth);
                    if cp < new_path.len() {
                        let new_leaf = Node::Leaf {
                            path:  new_path.slice(cp + 1),
                            value,
                        };
                        let new_idx = new_path.get(cp) as usize;
                        if let Node::Branch { ref mut children, .. } = branch {
                            children[new_idx] = self.commit_node(new_leaf);
                        }
                    } else {
                        if let Node::Branch { ref mut value: ref mut bv, .. } = branch {
                            *bv = Some(value);
                        }
                    }

                    // Wrap in extension if shared prefix exists
                    if cp > 0 {
                        Node::Extension {
                            prefix: lpath.slice(0).0[..cp].to_vec().into(),
                            next:   self.commit_node(branch),
                        }
                    } else {
                        branch
                    }
                }

                Node::Extension { prefix, next } => {
                    let remaining = path.slice(depth);
                    let cp = prefix.common_prefix_len(&remaining);

                    if cp == prefix.len() {
                        // Full prefix match → recurse into child
                        let new_next = self.insert_node(next, path, depth + cp, value);
                        Node::Extension { prefix, next: self.commit_node(new_next) }
                    } else {
                        // Partial match → branch here
                        let mut branch = Node::empty_branch();

                        // Rest of old extension
                        let old_suffix = prefix.slice(cp + 1);
                        let old_branch_idx = prefix.get(cp) as usize;
                        let old_child = if old_suffix.is_empty() {
                            self.resolve_to_node_ref(next)
                        } else {
                            self.commit_node(Node::Extension { prefix: old_suffix, next })
                        };
                        if let Node::Branch { ref mut children, .. } = branch {
                            children[old_branch_idx] = old_child;
                        }

                        // New value
                        let new_suffix = remaining.slice(cp + 1);
                        if cp < remaining.len() {
                            let new_leaf = Node::Leaf { path: new_suffix, value };
                            let new_idx  = remaining.get(cp) as usize;
                            if let Node::Branch { ref mut children, .. } = branch {
                                children[new_idx] = self.commit_node(new_leaf);
                            }
                        } else {
                            if let Node::Branch { ref mut value: ref mut bv, .. } = branch {
                                *bv = Some(value);
                            }
                        }

                        if cp > 0 {
                            Node::Extension {
                                prefix: prefix.0[..cp].to_vec().into(),
                                next:   self.commit_node(branch),
                            }
                        } else {
                            branch
                        }
                    }
                }

                Node::Branch { mut children, mut value: bvalue } => {
                    let remaining = path.slice(depth);
                    if remaining.is_empty() {
                        bvalue = Some(value);
                    } else {
                        let idx = remaining.get(0) as usize;
                        let old_child = std::mem::take(&mut children[idx]);
                        let new_child = self.insert_node(old_child, path, depth + 1, value);
                        children[idx] = self.commit_node(new_child);
                    }
                    Node::Branch { children, value: bvalue }
                }
            }
        }
    }

    // ---- delete ----

    fn delete_node(&mut self, node_ref: NodeRef, path: &Nibbles, depth: usize) -> Option<Node> {
        let node = self.resolve(&node_ref)?;
        match node {
            Node::Leaf { path: lpath, .. } => {
                if lpath == path.slice(depth) { None }  // deleted
                else { Some(Node::Leaf { path: lpath, value: vec![] }) }
            }
            Node::Extension { prefix, next } => {
                let remaining = path.slice(depth);
                let cp = prefix.common_prefix_len(&remaining);
                if cp < prefix.len() { return Some(Node::Extension { prefix, next }); }
                let new_next = self.delete_node(next, path, depth + cp)?;
                Some(Node::Extension { prefix, next: self.commit_node(new_next) })
            }
            Node::Branch { mut children, mut value } => {
                let remaining = path.slice(depth);
                if remaining.is_empty() {
                    value = None;
                } else {
                    let idx = remaining.get(0) as usize;
                    let old = std::mem::take(&mut children[idx]);
                    children[idx] = match self.delete_node(old, path, depth + 1) {
                        None    => NodeRef::Empty,
                        Some(n) => self.commit_node(n),
                    };
                }
                // Collapse branch if only one child remains
                Some(self.collapse_branch(children, value))
            }
        }
    }

    fn collapse_branch(&mut self, children: [NodeRef; 16], value: Option<Vec<u8>>) -> Node {
        let live: Vec<usize> = (0..16)
            .filter(|&i| !matches!(children[i], NodeRef::Empty))
            .collect();

        if live.is_empty() {
            if let Some(v) = value {
                return Node::Leaf { path: Nibbles::default(), value: v };
            }
        }

        if live.len() == 1 && value.is_none() {
            let idx = live[0];
            let child = children[idx].clone();
            if let Some(child_node) = self.resolve(&child) {
                match child_node {
                    Node::Leaf { path, value } => {
                        let mut new_path = vec![idx as u8];
                        new_path.extend_from_slice(&path.0);
                        return Node::Leaf { path: Nibbles(new_path), value };
                    }
                    Node::Extension { prefix, next } => {
                        let mut new_prefix = vec![idx as u8];
                        new_prefix.extend_from_slice(&prefix.0);
                        return Node::Extension { prefix: Nibbles(new_prefix), next };
                    }
                    _ => {}
                }
            }
        }

        Node::Branch { children, value }
    }

    // ---- Node commitment (hash vs inline) ----

    /// Encode a node. If encoded ≥ 32 bytes, store in KVStore and return Hash ref.
    /// If < 32 bytes, return Inline ref (avoids unnecessary KV round-trip).
    fn commit_node(&mut self, node: Node) -> NodeRef {
        let encoded = encode_node(&node);
        if encoded.len() < 32 {
            NodeRef::Inline(Box::new(node))
        } else {
            let hash = keccak256(&encoded);
            self.store.put(hash.clone(), encoded);
            NodeRef::Hash(hash)
        }
    }

    fn resolve_to_node_ref(&mut self, nr: NodeRef) -> NodeRef {
        nr  // already a NodeRef — used to avoid double-commit
    }

    /// Fetch and decode a node from KVStore (or inline).
    fn resolve(&mut self, node_ref: &NodeRef) -> Option<Node> {
        match node_ref {
            NodeRef::Empty       => None,
            NodeRef::Inline(n)   => Some(*n.clone()),
            NodeRef::Hash(h)     => {
                let enc = self.store.get(h)?.clone();
                Some(decode_node(&enc))
            }
        }
    }

    pub fn kv_stats(&self) -> (usize, usize, usize) {
        self.store.stats()
    }
}

// Helper to build a Nibbles from a raw Vec<u8>
impl From<Vec<u8>> for Nibbles {
    fn from(v: Vec<u8>) -> Self { Nibbles(v) }
}

// =============================================================================
// § 8.  Account State
// =============================================================================

/// Ethereum account state (EIP-161 layout).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountState {
    /// Number of transactions sent (or contracts created for contract accounts).
    pub nonce:        u64,
    /// Balance in Wei (u128 supports up to ~3.4×10³⁸ Wei).
    pub balance:      u128,
    /// Root of this account's storage MPT.
    pub storage_root: H256,
    /// Keccak256 of the account's EVM bytecode (EOA = keccak256([])).
    pub code_hash:    H256,
}

impl AccountState {
    /// Create a fresh Externally Owned Account (no code, no storage).
    pub fn new_eoa(balance: u128) -> Self {
        AccountState {
            nonce:        0,
            balance,
            storage_root: H256::ZERO,
            code_hash:    keccak256(&[]),   // empty code hash
        }
    }

    /// Create a contract account with the given code.
    pub fn new_contract(balance: u128, code: &[u8]) -> Self {
        AccountState {
            nonce:        1,
            balance,
            storage_root: H256::ZERO,
            code_hash:    keccak256(code),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nonce == 0 && self.balance == 0 && self.code_hash == keccak256(&[])
    }

    /// RLP-encode the account for storage in the state trie.
    pub fn encode(&self) -> Vec<u8> {
        let nonce_bytes   = self.nonce.to_be_bytes();
        let balance_bytes = self.balance.to_be_bytes();
        rlp_encode_list(&[
            rlp_encode_bytes(trim_leading_zeros(&nonce_bytes)),
            rlp_encode_bytes(trim_leading_zeros(&balance_bytes)),
            rlp_encode_bytes(self.storage_root.as_bytes()),
            rlp_encode_bytes(self.code_hash.as_bytes()),
        ])
    }

    /// Decode an RLP-encoded account.
    pub fn decode(data: &[u8]) -> Self {
        let items = rlp_decode_list(data);
        AccountState {
            nonce:        decode_u64(&items[0]),
            balance:      decode_u128(&items[1]),
            storage_root: to_h256(&items[2]),
            code_hash:    to_h256(&items[3]),
        }
    }
}

fn trim_leading_zeros(b: &[u8]) -> &[u8] {
    let start = b.iter().position(|&x| x != 0).unwrap_or(b.len() - 1);
    &b[start..]
}

fn decode_u64(b: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr[8 - b.len()..].copy_from_slice(b);
    u64::from_be_bytes(arr)
}

fn decode_u128(b: &[u8]) -> u128 {
    let mut arr = [0u8; 16];
    arr[16 - b.len()..].copy_from_slice(b);
    u128::from_be_bytes(arr)
}

fn to_h256(b: &[u8]) -> H256 {
    let mut arr = [0u8; 32];
    let start = 32usize.saturating_sub(b.len());
    arr[start..].copy_from_slice(&b[..b.len().min(32)]);
    H256(arr)
}

// =============================================================================
// § 9.  Address type
// =============================================================================

/// 20-byte Ethereum address.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Address([u8; 20]);

impl Address {
    pub fn from_bytes(b: &[u8; 20]) -> Self { Address(*b) }

    pub fn from_hex(s: &str) -> Self {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let mut arr = [0u8; 20];
        for i in 0..20 {
            arr[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
        Address(arr)
    }

    pub fn as_bytes(&self) -> &[u8; 20] { &self.0 }

    pub fn to_hex(&self) -> String {
        format!("0x{}", self.0.iter().map(|b| format!("{b:02x}")).collect::<String>())
    }

    /// The trie key for an address is keccak256(address) — prevents
    /// adversarial path crafting that degrades trie performance.
    pub fn trie_key(&self) -> H256 {
        keccak256(&self.0)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// =============================================================================
// § 10.  Storage Trie  (per-account key-value store)
// =============================================================================
// Maps 32-byte slot keys → 32-byte slot values.
// The root of this trie is stored as `storage_root` in the account state.

pub struct StorageTrie {
    trie: MerkleTrie,
}

impl StorageTrie {
    pub fn new() -> Self {
        StorageTrie { trie: MerkleTrie::new() }
    }

    /// The storage root — goes into AccountState.storage_root.
    pub fn root(&self) -> H256 {
        self.trie.root_hash()
    }

    /// Set a storage slot. `slot` and `value` are both 32-byte quantities.
    /// Setting to zero removes the slot (EIP-2929 / EIP-3529 gas refund logic
    /// not modelled here).
    pub fn set(&mut self, slot: H256, value: H256) {
        if value == H256::ZERO {
            self.trie.delete(slot.as_bytes());
        } else {
            // Value is RLP-encoded before insertion (Ethereum spec)
            let encoded = rlp_encode_bytes(trim_leading_zeros(value.as_bytes()));
            self.trie.insert(slot.as_bytes(), encoded);
        }
    }

    /// Get a storage slot (returns zero if not set).
    pub fn get(&mut self, slot: &H256) -> H256 {
        match self.trie.get(slot.as_bytes()) {
            None => H256::ZERO,
            Some(encoded) => {
                let (raw, _) = rlp_decode_item(&encoded);
                to_h256(&raw)
            }
        }
    }
}

// =============================================================================
// § 11.  StateDB — top-level database
// =============================================================================
// This is the object a block executor holds.  It:
//   - Manages the world-state MPT (address → AccountState)
//   - Maintains per-account storage tries
//   - Exposes a simple read/write API
//   - Computes the global state root after each block

pub struct StateDB {
    /// World-state trie: keccak256(address) → RLP(AccountState)
    state_trie:    MerkleTrie,
    /// Per-account storage tries (loaded lazily, flushed on commit)
    storage_tries: HashMap<Address, StorageTrie>,
    /// Contract bytecode store: code_hash → bytecode
    code_store:    HashMap<H256, Vec<u8>>,
}

impl StateDB {
    pub fn new() -> Self {
        StateDB {
            state_trie:    MerkleTrie::new(),
            storage_tries: HashMap::new(),
            code_store:    HashMap::new(),
        }
    }

    // ---- Account operations ----

    /// Read an account (returns None if it doesn't exist).
    pub fn get_account(&mut self, addr: &Address) -> Option<AccountState> {
        let key = addr.trie_key();
        let raw = self.state_trie.get(key.as_bytes())?;
        Some(AccountState::decode(&raw))
    }

    /// Write an account back to the state trie.
    pub fn set_account(&mut self, addr: &Address, account: AccountState) {
        let key     = addr.trie_key();
        let encoded = account.encode();
        self.state_trie.insert(key.as_bytes(), encoded);
    }

    /// Delete an account (e.g. after SELFDESTRUCT).
    pub fn delete_account(&mut self, addr: &Address) {
        let key = addr.trie_key();
        self.state_trie.delete(key.as_bytes());
        self.storage_tries.remove(addr);
    }

    /// Create a new EOA with a given balance.
    pub fn create_eoa(&mut self, addr: &Address, balance: u128) {
        self.set_account(addr, AccountState::new_eoa(balance));
    }

    /// Deploy a contract (sets code, returns code_hash).
    pub fn deploy_contract(
        &mut self,
        addr: &Address,
        balance: u128,
        code: &[u8],
    ) -> H256 {
        let code_hash = keccak256(code);
        self.code_store.insert(code_hash.clone(), code.to_vec());
        self.set_account(addr, AccountState::new_contract(balance, code));
        code_hash
    }

    // ---- Balance helpers ----

    pub fn get_balance(&mut self, addr: &Address) -> u128 {
        self.get_account(addr).map_or(0, |a| a.balance)
    }

    pub fn set_balance(&mut self, addr: &Address, balance: u128) {
        let mut acc = self.get_account(addr).unwrap_or_else(AccountState::new_eoa_zero);
        acc.balance = balance;
        self.set_account(addr, acc);
    }

    pub fn transfer(&mut self, from: &Address, to: &Address, amount: u128) -> Result<(), &'static str> {
        let from_bal = self.get_balance(from);
        if from_bal < amount { return Err("insufficient balance"); }
        let to_bal = self.get_balance(to);
        self.set_balance(from, from_bal - amount);
        self.set_balance(to,   to_bal   + amount);
        Ok(())
    }

    // ---- Nonce helpers ----

    pub fn get_nonce(&mut self, addr: &Address) -> u64 {
        self.get_account(addr).map_or(0, |a| a.nonce)
    }

    pub fn increment_nonce(&mut self, addr: &Address) {
        let mut acc = self.get_account(addr).unwrap_or_else(AccountState::new_eoa_zero);
        acc.nonce += 1;
        self.set_account(addr, acc);
    }

    // ---- Storage operations ----

    /// Read a storage slot from an account's storage trie.
    pub fn get_storage(&mut self, addr: &Address, slot: &H256) -> H256 {
        self.storage_tries
            .entry(addr.clone())
            .or_insert_with(StorageTrie::new)
            .get(slot)
    }

    /// Write a storage slot and update the account's storage_root.
    pub fn set_storage(&mut self, addr: &Address, slot: H256, value: H256) {
        let strie = self.storage_tries
            .entry(addr.clone())
            .or_insert_with(StorageTrie::new);
        strie.set(slot, value);
        let new_storage_root = strie.root();

        // Persist the new storage_root into the account state
        if let Some(mut acc) = self.get_account(addr) {
            acc.storage_root = new_storage_root;
            self.set_account(addr, acc);
        }
    }

    // ---- Code ----

    pub fn get_code(&self, code_hash: &H256) -> Option<&Vec<u8>> {
        self.code_store.get(code_hash)
    }

    pub fn get_code_for(&self, addr_code_hash: &H256) -> Option<&Vec<u8>> {
        self.code_store.get(addr_code_hash)
    }

    // ---- Commit / Root ----

    /// Compute the global state root (= root of the world-state MPT).
    /// This is what goes into the block header.
    pub fn state_root(&self) -> H256 {
        self.state_trie.root_hash()
    }

    /// Print a summary.
    pub fn summary(&self) {
        let (nodes, writes, reads) = self.state_trie.kv_stats();
        println!("StateDB summary:");
        println!("  state root:       {}", self.state_root());
        println!("  state trie nodes: {} ({} writes, {} reads)", nodes, writes, reads);
        println!("  storage tries:    {}", self.storage_tries.len());
        println!("  contracts stored: {}", self.code_store.len());
    }
}

impl AccountState {
    fn new_eoa_zero() -> Self { AccountState::new_eoa(0) }
}

// =============================================================================
// § 12.  Demo
// =============================================================================

fn demo() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║         Ethereum MPT State Database — Demo           ║");
    println!("╚══════════════════════════════════════════════════════╝");

    let mut db = StateDB::new();

    // ── Addresses ──────────────────────────────────────────────────────────
    let alice   = Address::from_hex("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let bob     = Address::from_hex("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let charlie = Address::from_hex("0xcccccccccccccccccccccccccccccccccccccccc");
    let vault   = Address::from_hex("0x1234567890abcdef1234567890abcdef12345678");

    // ── 1. Create EOAs ──────────────────────────────────────────────────────
    println!("\n[1] Creating accounts");
    db.create_eoa(&alice,   1_000_000);
    db.create_eoa(&bob,       500_000);
    db.create_eoa(&charlie,   250_000);

    println!("  alice   balance: {}", db.get_balance(&alice));
    println!("  bob     balance: {}", db.get_balance(&bob));
    println!("  charlie balance: {}", db.get_balance(&charlie));

    let root_1 = db.state_root();
    println!("  state root: {root_1:?}");

    // ── 2. Transfers ────────────────────────────────────────────────────────
    println!("\n[2] Transfers");
    db.transfer(&alice, &bob, 100_000).unwrap();
    db.transfer(&bob, &charlie, 50_000).unwrap();
    db.increment_nonce(&alice);
    db.increment_nonce(&bob);

    println!("  alice   balance: {} nonce: {}", db.get_balance(&alice), db.get_nonce(&alice));
    println!("  bob     balance: {} nonce: {}", db.get_balance(&bob),   db.get_nonce(&bob));
    println!("  charlie balance: {}", db.get_balance(&charlie));

    let root_2 = db.state_root();
    println!("  state root changed: {}", root_1 != root_2);
    println!("  state root: {root_2:?}");

    // ── 3. Deploy a contract ────────────────────────────────────────────────
    println!("\n[3] Deploy contract (Vault)");
    let bytecode = b"\x60\x80\x60\x40\x52\x34\x80\x15";   // fake EVM bytecode
    let code_hash = db.deploy_contract(&vault, 0, bytecode);
    println!("  code_hash: {code_hash:?}");
    println!("  code len:  {} bytes", db.get_code(&code_hash).map_or(0, |c| c.len()));

    let root_3 = db.state_root();
    println!("  state root: {root_3:?}");

    // ── 4. Storage reads and writes ─────────────────────────────────────────
    println!("\n[4] Contract storage (Vault)");

    // Slot 0: total deposits
    let slot_total = H256([0u8; 32]);
    // Slot 1: owner address (packed into bytes32)
    let mut slot_owner_bytes = [0u8; 32];
    slot_owner_bytes[12..].copy_from_slice(alice.as_bytes());
    let slot_owner = H256([1u8; 32]);

    // Write storage
    let mut total_bytes = [0u8; 32];
    total_bytes[24..].copy_from_slice(&500_000u64.to_be_bytes());
    db.set_storage(&vault, slot_total.clone(), H256(total_bytes));
    db.set_storage(&vault, slot_owner.clone(), H256(slot_owner_bytes));

    // Read back
    let stored_total = db.get_storage(&vault, &slot_total);
    let stored_owner = db.get_storage(&vault, &slot_owner);
    println!("  vault.totalDeposits: {stored_total:?}");
    println!("  vault.owner:         {stored_owner:?}");

    // Verify storage_root updated in account
    let vault_acc = db.get_account(&vault).unwrap();
    println!("  vault.storage_root:  {:?}", vault_acc.storage_root);
    assert_ne!(vault_acc.storage_root, H256::ZERO, "storage root should be non-zero");

    let root_4 = db.state_root();
    println!("  state root: {root_4:?}");

    // ── 5. Update storage slot ──────────────────────────────────────────────
    println!("\n[5] Update vault storage");
    let mut new_total_bytes = [0u8; 32];
    new_total_bytes[24..].copy_from_slice(&750_000u64.to_be_bytes());
    db.set_storage(&vault, slot_total.clone(), H256(new_total_bytes));

    let updated = db.get_storage(&vault, &slot_total);
    println!("  updated totalDeposits: {updated:?}");
    let root_5 = db.state_root();
    println!("  root changed: {}", root_4 != root_5);

    // ── 6. Delete an account ────────────────────────────────────────────────
    println!("\n[6] SELFDESTRUCT charlie");
    // Send charlie's balance to alice first
    let charlie_bal = db.get_balance(&charlie);
    db.transfer(&charlie, &alice, charlie_bal).unwrap();
    db.delete_account(&charlie);
    println!("  charlie exists: {}", db.get_account(&charlie).is_some());
    println!("  alice balance:  {}", db.get_balance(&alice));

    let root_6 = db.state_root();
    println!("  state root: {root_6:?}");

    // ── 7. Low-level trie test ───────────────────────────────────────────────
    println!("\n[7] Low-level MPT — address prefix clustering");
    let mut trie = MerkleTrie::new();

    // Addresses sharing a long common prefix (tests extension node collapsing)
    let keys: &[&[u8]] = &[
        b"\xaa\xbb\x00\x01",
        b"\xaa\xbb\x00\x02",
        b"\xaa\xbb\x01\x00",
        b"\xcc\x00\x00\x00",
    ];
    for (i, k) in keys.iter().enumerate() {
        trie.insert(k, vec![i as u8]);
    }
    for (i, k) in keys.iter().enumerate() {
        assert_eq!(trie.get(k), Some(vec![i as u8]));
    }
    println!("  all {} keys verified ✓", keys.len());
    println!("  trie root: {:?}", trie.root_hash());

    trie.delete(keys[1]);
    assert_eq!(trie.get(keys[1]), None);
    assert_eq!(trie.get(keys[0]), Some(vec![0u8])); // sibling unaffected
    println!("  delete + sibling check ✓");

    // ── 8. Final summary ────────────────────────────────────────────────────
    println!();
    db.summary();
    println!("\n All assertions passed ✓");
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(b: u8) -> Address {
        Address([b; 20])
    }

    fn slot(n: u8) -> H256 {
        let mut arr = [0u8; 32];
        arr[31] = n;
        H256(arr)
    }

    // ---- Nibble encoding ----

    #[test]
    fn nibbles_from_bytes_roundtrip() {
        let bytes = [0xab, 0xcd];
        let nibs  = Nibbles::from_bytes(&bytes);
        assert_eq!(nibs.0, vec![0xa, 0xb, 0xc, 0xd]);
    }

    #[test]
    fn hex_prefix_even_leaf() {
        let nibs = Nibbles(vec![1, 2, 3, 4]);
        let enc  = nibs.encode_hp(true);
        let (dec, is_leaf) = Nibbles::decode_hp(&enc);
        assert!(is_leaf);
        assert_eq!(dec, nibs);
    }

    #[test]
    fn hex_prefix_odd_extension() {
        let nibs = Nibbles(vec![1, 2, 3]);
        let enc  = nibs.encode_hp(false);
        let (dec, is_leaf) = Nibbles::decode_hp(&enc);
        assert!(!is_leaf);
        assert_eq!(dec, nibs);
    }

    // ---- MPT operations ----

    #[test]
    fn insert_and_get() {
        let mut t = MerkleTrie::new();
        t.insert(b"key1", b"value1".to_vec());
        t.insert(b"key2", b"value2".to_vec());
        assert_eq!(t.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(t.get(b"key2"), Some(b"value2".to_vec()));
        assert_eq!(t.get(b"key3"), None);
    }

    #[test]
    fn update_changes_root() {
        let mut t = MerkleTrie::new();
        t.insert(b"k", b"v1".to_vec());
        let r1 = t.root_hash();
        t.insert(b"k", b"v2".to_vec());
        let r2 = t.root_hash();
        assert_ne!(r1, r2);
        assert_eq!(t.get(b"k"), Some(b"v2".to_vec()));
    }

    #[test]
    fn delete_removes_key() {
        let mut t = MerkleTrie::new();
        t.insert(b"a", b"1".to_vec());
        t.insert(b"b", b"2".to_vec());
        assert!(t.delete(b"a"));
        assert_eq!(t.get(b"a"), None);
        assert_eq!(t.get(b"b"), Some(b"2".to_vec()));
    }

    #[test]
    fn shared_prefix_keys() {
        let mut t = MerkleTrie::new();
        t.insert(b"\xaa\xbb\x00", b"x".to_vec());
        t.insert(b"\xaa\xbb\x01", b"y".to_vec());
        t.insert(b"\xaa\xcc\x00", b"z".to_vec());
        assert_eq!(t.get(b"\xaa\xbb\x00"), Some(b"x".to_vec()));
        assert_eq!(t.get(b"\xaa\xbb\x01"), Some(b"y".to_vec()));
        assert_eq!(t.get(b"\xaa\xcc\x00"), Some(b"z".to_vec()));
    }

    #[test]
    fn empty_trie_root_is_zero() {
        let t = MerkleTrie::new();
        assert_eq!(t.root_hash(), H256::ZERO);
    }

    #[test]
    fn root_is_deterministic() {
        let mut t1 = MerkleTrie::new();
        let mut t2 = MerkleTrie::new();
        for (k, v) in [(b"a" as &[u8], b"1" as &[u8]), (b"b", b"2")] {
            t1.insert(k, v.to_vec());
            t2.insert(k, v.to_vec());
        }
        assert_eq!(t1.root_hash(), t2.root_hash());
    }

    // ---- Account encoding ----

    #[test]
    fn account_encode_decode_roundtrip() {
        let acc = AccountState {
            nonce:        42,
            balance:      1_000_000_000_000_000_000,
            storage_root: H256::ZERO,
            code_hash:    keccak256(&[]),
        };
        let enc = acc.encode();
        let dec = AccountState::decode(&enc);
        assert_eq!(acc, dec);
    }

    // ---- StateDB ----

    #[test]
    fn create_and_read_account() {
        let mut db = StateDB::new();
        let a = addr(0xaa);
        db.create_eoa(&a, 999);
        assert_eq!(db.get_balance(&a), 999);
        assert_eq!(db.get_nonce(&a), 0);
    }

    #[test]
    fn transfer_succeeds() {
        let mut db = StateDB::new();
        let a = addr(1); let b = addr(2);
        db.create_eoa(&a, 1000);
        db.create_eoa(&b, 0);
        db.transfer(&a, &b, 400).unwrap();
        assert_eq!(db.get_balance(&a), 600);
        assert_eq!(db.get_balance(&b), 400);
    }

    #[test]
    fn transfer_insufficient_balance_fails() {
        let mut db = StateDB::new();
        let a = addr(1); let b = addr(2);
        db.create_eoa(&a, 100);
        db.create_eoa(&b, 0);
        assert!(db.transfer(&a, &b, 999).is_err());
    }

    #[test]
    fn storage_get_set() {
        let mut db = StateDB::new();
        let contract = addr(0xcc);
        db.create_eoa(&contract, 0);

        let s = slot(1);
        let val = H256([7u8; 32]);
        db.set_storage(&contract, s.clone(), val.clone());
        assert_eq!(db.get_storage(&contract, &s), val);
    }

    #[test]
    fn storage_zero_clears_slot() {
        let mut db = StateDB::new();
        let contract = addr(0xdd);
        db.create_eoa(&contract, 0);

        let s = slot(0);
        db.set_storage(&contract, s.clone(), H256([1u8; 32]));
        db.set_storage(&contract, s.clone(), H256::ZERO);   // clear
        assert_eq!(db.get_storage(&contract, &s), H256::ZERO);
    }

    #[test]
    fn state_root_changes_on_transfer() {
        let mut db = StateDB::new();
        let a = addr(1); let b = addr(2);
        db.create_eoa(&a, 500);
        db.create_eoa(&b, 500);
        let r1 = db.state_root();
        db.transfer(&a, &b, 100).unwrap();
        assert_ne!(db.state_root(), r1);
    }

    #[test]
    fn delete_account_removes_from_trie() {
        let mut db = StateDB::new();
        let a = addr(0xee);
        db.create_eoa(&a, 100);
        assert!(db.get_account(&a).is_some());
        db.delete_account(&a);
        assert!(db.get_account(&a).is_none());
    }

    #[test]
    fn deploy_contract_stores_code() {
        let mut db = StateDB::new();
        let contract = addr(0xff);
        let code = b"\x60\x00\x60\x00\xf3";
        let hash = db.deploy_contract(&contract, 0, code);
        assert_eq!(db.get_code(&hash).unwrap().as_slice(), code);
        let acc = db.get_account(&contract).unwrap();
        assert_eq!(acc.code_hash, hash);
    }

    #[test]
    fn storage_root_updates_on_set() {
        let mut db = StateDB::new();
        let c = addr(0x42);
        db.create_eoa(&c, 0);
        let before = db.get_account(&c).unwrap().storage_root;
        db.set_storage(&c, slot(0), H256([9u8; 32]));
        let after = db.get_account(&c).unwrap().storage_root;
        assert_ne!(before, after);
    }

    #[test]
    fn multiple_accounts_independent_storage() {
        let mut db = StateDB::new();
        let a = addr(0x01);
        let b = addr(0x02);
        db.create_eoa(&a, 0);
        db.create_eoa(&b, 0);

        let s = slot(0);
        db.set_storage(&a, s.clone(), H256([0xaa; 32]));
        db.set_storage(&b, s.clone(), H256([0xbb; 32]));

        assert_eq!(db.get_storage(&a, &s), H256([0xaa; 32]));
        assert_eq!(db.get_storage(&b, &s), H256([0xbb; 32]));
    }
}*/