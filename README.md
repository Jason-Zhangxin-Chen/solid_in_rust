# Rust Examples

For those Rust beginners who want to see a wide variety of examples in one place, and for intermediate/advanced Rustaceans who want to understand how things work under the hood — this repo is for you.
A collection of Rust examples covering data structures, algorithms, language internals, and systems programming — from safe idiomatic patterns through to `unsafe` raw-pointer implementations.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Module Overview](#module-overview)
  - [Language Fundamentals](#language-fundamentals)
  - [Data Structures — Safe](#data-structures--safe)
  - [Data Structures — Raw / Unsafe](#data-structures--raw--unsafe)
  - [Algorithms](#algorithms)
  - [Error Handling](#error-handling)
  - [Systems & Advanced](#systems--advanced)
- [Learning Path](#learning-path)

---

## Getting Started

```bash
git clone <repo-url>
cd <repo>
cargo run
```

No nightly toolchain required. A small number of modules pull in third-party crates (noted below); everything else uses only `std`.

---

## Module Overview

### Language Fundamentals

| File | What it covers |
|------|---------------|
| `basic.rs` | A single-file tour of every major Rust feature: primitives, ownership/move/clone/copy, borrowing, lifetimes, slices, `String` vs `&str`, structs, enums, pattern matching, `Option`, `Result`, and the `?` operator. |
| `iterator.rs` | Building custom iterators from scratch — owned (`IntoIterator`), shared (`iter()`), and mutable (`iter_mut()`) variants on a custom container. |
| `my_container.rs` / `my_container2.rs` | Two iterations of a generic container demonstrating iterator protocol with `PhantomData` and lifetime annotations. |
| `smart_pointers.rs` | Full API walkthrough of `Box`, `Rc`, `Arc`, `Cell`, `RefCell`, `Cow`, `Option`, and `Result`. |
| `deref.rs` | Deep dive into the `Deref` / `DerefMut` traits, deref coercions, coercion chains, and when to prefer `AsRef` over `Deref`. |
| `phantom_data.rs` | Using `PhantomData<T>` for type-safe wrappers (e.g. `Id<User>` vs `Id<Post>`), lifetime variance, and ownership hints to the compiler. |
| `pin_unpin.rs` | Self-referential structs, `PhantomPinned`, `Pin<P>`, and why `Unpin` matters for async Rust. |
| `sub_typing.rs` | Lifetime-based subtyping and variance — covariance, contravariance, and invariance illustrated on references, `Box`, `Vec`, `&mut T`, and `Cell<T>`. |
| `bits.rs` | Exhaustive reference for bitwise operators, bit-counting, shifting, inspection, manipulation, byte-order helpers, and overflow-aware arithmetic. |
| `marcros.rs` | Comprehensive `macro_rules!` guide: basic syntax, fragment specifiers, repetition patterns, multiple arms, nested repetition, recursive macros, and notes on procedural macros. |

---

### Data Structures — Safe

Implementations built with safe Rust using `Box`, `Rc`/`Weak`, `RefCell`, and standard collections.

| File | Data structure |
|------|---------------|
| `single_linked_list.rs` | Generic singly linked list via `Box<Node<T>>`. |
| `double_linked_list.rs` | Doubly linked list using `Rc<RefCell<Node<T>>>` and `Weak` back-pointers to avoid reference cycles. |
| `binary_tree.rs` | Binary search tree with BFS/DFS traversals and a custom iterative `Drop` impl to avoid stack overflows on deep trees. |
| `general_tree.rs` | N-ary (general) tree with `Box`-owned children and BFS/DFS traversal. |
| `graph.rs` | Directed graph with `Rc<RefCell<GraphNode<T>>>` nodes and `Weak` edges to break ownership cycles. |
| `stack.rs` | Generic stack backed by a `Vec` with pre-reserved capacity. |
| `ring_buffer.rs` | Fixed-capacity ring buffer using `MaybeUninit<T>` for uninitialized heap storage. |
| `heap.rs` | Generic min-heap with O(n) heapify and O(log n) push/pop. |
| `hash_map.rs` | Hash map from scratch: separate chaining, configurable load factor, and incremental resize. |

---

### Data Structures — Raw / Unsafe

The same data structures re-implemented with raw pointers (`*mut T`, `NonNull<T>`), manual allocation, and `unsafe` blocks. Useful for understanding what the safe abstractions do under the hood.

| File | Data structure |
|------|---------------|
| `single_linked_list_raw.rs` | Singly linked list with raw `*mut Node<T>` pointers and manual `Box::into_raw` / `Box::from_raw` memory management. |
| `single_linked_list_nonnull.rs` | Singly linked list using `NonNull<T>` and the global allocator (`alloc` / `dealloc`) directly — no `Box`. |
| `double_linked_list_raw.rs` | Doubly linked list with `NonNull<Node<T>>` and explicit heap allocation. |
| `binary_tree_raw.rs` | Binary search tree via `NonNull<Node<T>>` with `PhantomData` for variance and a custom iterative drop. |
| `general_tree_raw.rs` | N-ary tree with raw `NonNull` children and explicit `allocate` / `deallocate` helpers. |
| `graph_raw.rs` | Directed graph where edges are raw `NonNull` pointers stored behind `RefCell` — the `Graph` owns all nodes and guarantees pointer validity for its lifetime. |

---

### Algorithms

| File | Algorithm |
|------|-----------|
| `sorts.rs` | Sorting algorithm collection: bubble sort, insertion sort, selection sort, merge sort, quicksort, and heapsort — all generic over `Ord`. |
| `fib.rs` | Fibonacci three ways — iterative, naive recursive, and memoized — with timing comparisons. |
| `n_queen.rs` | N-Queens solver using backtracking with column / diagonal conflict tracking. |

---

### Error Handling

Three complementary approaches to error handling, from first principles to production patterns.

| File | Approach |
|------|----------|
| `error_raw.rs` | Manual error handling with only `std`: implementing the `Error` trait by hand, `Display`, `Debug`, error source chains, and the `?` operator without any external crates. |
| `error_framework.rs` | Ecosystem-crate patterns: `thiserror` for library error types (derive macros, `#[error(...)]`, `#[from]`) and `anyhow` for application-level error propagation. |
| `error_bitmask.rs` | High-performance bitmask error codes (`u128`) for ultra-low-latency scenarios (trading systems, game engines) where heap allocation and dynamic dispatch are unacceptable. |

---

### Systems & Advanced

| File | Topic |
|------|-------|
| `custom_allocator.rs` | A `GlobalAlloc` implementation that wraps the system allocator and tracks live allocations via a `Mutex<HashSet<usize>>` — the foundation of a leak detector. |
| `merkle_tree.rs` | Complete Merkle tree: leaf hashing, pairwise internal hashing, root computation, and inclusion proofs. Covers use cases in blockchains, Git, IPFS, and certificate transparency. |
| `trie_db.rs` | Ethereum-style state database on top of a Modified Merkle Patricia Trie (MPT), with a `StateDB`, per-account storage tries, and a `KVStore` backed by a `HashMap`. |
| `order_book_state_root.rs` | Concurrent order book with a cryptographic state root: `SkipMap`-based price levels, `DashMap` for order storage, `crossbeam` lock-free queues, and SHA-256 state hashing. |
| `tcp_service.rs` | TCP server with a length-prefixed binary wire protocol (little-endian `id` + `payload_len` header), per-connection threads, and `serde`-based payload deserialization. |
| `pcm.rs` | Concurrency patterns using `crossbeam-channel`: MPSC, SPMC, pipeline, fan-out/fan-in, and `select!`-based multiplexing. |

---

## Learning Path

If you are working through the material systematically, this order is recommended:

1. **Start with the language** — `basic.rs` → `smart_pointers.rs` → `deref.rs` → `iterator.rs`
2. **Type system deep dives** — `phantom_data.rs` → `sub_typing.rs` → `pin_unpin.rs`
3. **Safe data structures** — linked lists → trees → graph → heap → hash map
4. **Error handling** — `error_raw.rs` → `error_framework.rs` → `error_bitmask.rs`
5. **Unsafe internals** — re-visit the raw variants of each data structure after the safe versions
6. **Algorithms** — `sorts.rs` → `fib.rs` → `n_queen.rs`
7. **Systems topics** — `custom_allocator.rs` → `merkle_tree.rs` → `trie_db.rs` → `order_book_state_root.rs` → `tcp_service.rs` → `pcm.rs`