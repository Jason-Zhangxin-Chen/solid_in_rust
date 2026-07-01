
# solid_in_rust — Rust examples and deep-dives

A curated collection of Rust examples and deep-dives: language fundamentals, data structures (safe and `unsafe` implementations), algorithms, concurrency, smart pointers, design patterns, standard-library traits, and common mistakes. The repository is intended for learners at all levels — beginners can follow guided examples, and intermediate/advanced Rustaceans can inspect `unsafe` implementations to see what the safe abstractions do under the hood.

Key points:
- Single crate (edition 2021) built with Cargo
- Most examples use only `std`; several modules use popular crates (listed in `Cargo.toml`) for systems and concurrency examples

## Prerequisites

- Rust toolchain (stable). No nightly toolchain is required.
- cargo (comes with rustup-installed toolchain)

## Quick start

```bash
git clone <repo-url>
cd solid_in_rust
cargo build --release   # compile the crate
cargo run               # runs the default "Hello, world!" in src/main.rs
cargo test              # run tests if any are present
```

## Running examples

This repository is a single crate with module examples under `src/`. The default `cargo run` executes `src/main.rs`, which is a minimal "Hello, world!" placeholder.

To run code in a module you can:
1. Open `src/main.rs` and call functions or demo code from the module you want to run, or
2. Create a small binary under `examples/` (create `examples/<name>.rs`) and run it with `cargo run --example <name>`.

## Project layout

```
src/
├── main.rs                  # Minimal "Hello, world!" entry point
├── basic/                   # Language fundamentals
│   ├── basic.rs             #   Core syntax, types, control flow, ownership tour
│   ├── bits.rs              #   Bit manipulation and binary operations
│   ├── fat_pointer.rs       #   Fat pointers (dyn Trait, slices) internals
│   ├── iterator.rs          #   Implementing custom iterators
│   ├── visibility.rs        #   Module visibility and privacy rules
│   ├── all_about_trait.rs   #   Traits: definition, bounds, associated types
│   ├── error_raw.rs         #   Manual error handling with Result and enums
│   ├── error_bitmask.rs     #   Bitmask-based error patterns
│   ├── error_thiserror.rs   #   Derive-based errors with `thiserror`
│   └── error_anyhow.rs      #   Flexible error handling with `anyhow`
├── data_structures/         # Safe and unsafe data structure implementations
│   ├── stack.rs             #   Stack
│   ├── heap.rs              #   Binary heap
│   ├── hash_map.rs          #   Hash map
│   ├── ring_buffer.rs       #   Ring buffer / circular queue
│   ├── mem_pool.rs          #   Memory pool allocator
│   ├── merkle_tree.rs       #   Merkle tree
│   ├── trie_db.rs           #   Trie-based key-value store
│   ├── order_book_state_root.rs  # Order book with state-root hashing
│   ├── my_container.rs      #   Generic container example (1)
│   ├── my_container2.rs     #   Generic container example (2)
│   ├── single_linked_list.rs         #   Safe singly-linked list
│   ├── single_linked_list_raw.rs     #   Unsafe singly-linked list (raw pointers)
│   ├── single_linked_list_nonnull.rs #   Unsafe singly-linked list (NonNull)
│   ├── double_linked_list.rs         #   Safe doubly-linked list
│   ├── double_linked_list_raw.rs     #   Unsafe doubly-linked list (raw pointers)
│   ├── binary_tree.rs                #   Safe binary tree
│   ├── binary_tree_raw.rs            #   Unsafe binary tree (raw pointers)
│   ├── general_tree.rs               #   Safe general (n-ary) tree
│   ├── general_tree_raw.rs           #   Unsafe general tree (raw pointers)
│   ├── graph.rs                      #   Safe graph
│   └── graph_raw.rs                  #   Unsafe graph (raw pointers)
├── algo/                    # Algorithmic examples
│   ├── sorts.rs             #   Sorting algorithms
│   ├── fib.rs               #   Fibonacci (iterative, recursive, memoized)
│   └── n_queen.rs           #   N-Queens backtracking solver
├── advance/                 # Advanced Rust topics
│   ├── coercion.rs          #   Type coercions and Deref coercion
│   ├── deref.rs             #   Implementing Deref and DerefMut
│   ├── phantom_data.rs      #   PhantomData for type-level state
│   ├── pin_unpin.rs         #   Pin, Unpin, and self-referential types
│   ├── sub_typing.rs        #   Subtyping, variance, and lifetimes
│   ├── blanket_impl.rs      #   Blanket trait implementations
│   ├── AsRef.rs             #   AsRef and AsMut patterns
│   ├── marcros.rs           #   Declarative and procedural macros
│   ├── custom_allocator.rs  #   Custom memory allocators
│   └── serialize_deserialize.rs  # Serialization with serde
├── concurrency/             # Concurrency and parallelism
│   ├── basic_model.rs       #   Thread spawn, JoinHandle, basic patterns
│   ├── pcm.rs               #   Producer-consumer with channels
│   └── tcp_service.rs       #   Multi-threaded TCP server
├── smart_pointers/          # Smart pointer deep-dives
│   ├── smart_pointers_methods.rs  # Common smart pointer API surface
│   ├── uc_box.rs            #   Box<T> usage cases
│   ├── uc_rc.rs             #   Rc<T> usage cases (single-threaded ref counting)
│   ├── uc_arc.rs            #   Arc<T> usage cases (atomic ref counting)
│   ├── uc_cell.rs           #   Cell<T> interior mutability
│   ├── uc_refcell.rs        #   RefCell<T> runtime borrow checking
│   ├── uc_weak.rs           #   Weak<T> for breaking reference cycles
│   └── uc_cow.rs            #   Cow<T> clone-on-write
├── design_patterns/         # Design patterns in idiomatic Rust
│   ├── builder.rs           #   Builder pattern
│   ├── newtype.rs           #   Newtype pattern
│   ├── strategy.rs          #   Strategy pattern (trait objects)
│   ├── command.rs           #   Command pattern
│   ├── visitor.rs           #   Visitor pattern
│   ├── interpreter.rs       #   Interpreter pattern
│   ├── RAII_resource_guard.rs    # RAII guard pattern
│   ├── fold.rs              #   Fold / accumulate pattern
│   ├── compose.rs           #   Function composition
│   └── avoid_complex_type_bound.rs  # Taming complex trait bounds
├── std_traits/              # Standard-library trait tour
│   ├── conversions.rs       #   From, Into, TryFrom, TryInto
│   ├── operator.rs          #   Add, Index, Deref, and operator overloading
│   ├── iterator.rs          #   Iterator and IntoIterator
│   ├── formatting.rs        #   Display, Debug, and formatting traits
│   ├── memory.rs            #   Drop, Clone, Copy
│   ├── comparison.rs        #   PartialEq, Eq, PartialOrd, Ord
│   ├── marker.rs            #   Send, Sync, Sized, Unpin, and auto traits
│   ├── error.rs             #   Error trait and error handling
│   ├── io.rs                #   Read, Write, and I/O traits
│   └── closure.rs           #   Fn, FnMut, FnOnce
└── mistakes/                # Common pitfalls and how to avoid them
    ├── common_issues.rs     #   Frequent Rust gotchas
    ├── common_refine.rs     #   Refining code to avoid issues
    └── mem_leak.rs          #   Memory leak scenarios and prevention
```

## Dependencies

See `Cargo.toml`. Notable crates used by some modules:

| Crate | Usage |
|-------|-------|
| `serde` / `serde_json` | Serialization / deserialization |
| `rand` | Random number generation |
| `crossbeam-queue`, `crossbeam-channel`, `crossbeam-skiplist`, `crossbeam-utils` | Concurrent data structures |
| `dashmap` | Concurrent hash map |
| `disruptor` | Lock-free multi-producer multi-consumer ring buffer |
| `sha2` | SHA-2 cryptographic hashing |
| `thiserror` | Derive-macro error types |
| `anyhow` | Flexible application-level error handling |
| `log` | Logging facade |
| `mutex` | Mutex utilities |

## Learning path

Recommended progression:
1. **`basic/`** — Start here for language fundamentals: syntax, ownership, traits, and error handling.
2. **`smart_pointers/`** — Understand `Box`, `Rc`, `Arc`, `Cell`, `RefCell`, and `Cow`.
3. **`std_traits/`** — Tour the most important standard-library traits.
4. **`design_patterns/`** — See how classic patterns translate to idiomatic Rust.
5. **`data_structures/`** — Study safe implementations first, then compare with `_raw` variants to understand what unsafe code enables.
6. **`algo/`** — Algorithmic problem-solving in Rust.
7. **`advance/`** — Deep dives into coercion, `Pin`, variance, macros, and allocators.
8. **`concurrency/`** — Threading, channels, and concurrent data structures.
9. **`mistakes/`** — Review common pitfalls and how to avoid them.

## Contributing

Feel free to open issues or PRs. Small improvements that help learners are welcome: clearer examples, more comments, or additional `examples/` binaries.

## Notes for maintainers

- Because the repo prefers single-crate organization, adding runnable examples is best done via `examples/` or by adding small integration tests so contributors can run individual demos without editing `src/main.rs`.

## License

No license file is included in this repo. Add `LICENSE` if you want to make the content explicitly open-source.
