
# solid_in_rust — Rust examples and deep-dives

A curated collection of Rust examples and deep-dives: language fundamentals, data structures (safe and raw/unsafe implementations), algorithms, error handling patterns, and systems-level topics. The repository is intended for learners at all levels — beginners can follow guided examples, and intermediate/advanced Rustaceans can inspect `unsafe` implementations to see what the safe abstractions do under the hood.

Key points:
- Single crate (edition 2021) built with Cargo
- Most examples use only `std`; several modules use popular crates (listed in `Cargo.toml`) for systems and concurrency examples

Prerequisites
- Rust toolchain (stable). No nightly toolchain is required.
- cargo (comes with rustup-installed toolchain)

Quick start

```bash
git clone <repo-url>
cd solid_in_rust
cargo build --release   # compile the crate
cargo run               # runs the default `src/main.rs` demo
cargo test              # run tests if any are present
```

Running examples
- This repository is a single crate with module examples under `src/`. The default `cargo run` executes `src/main.rs`, which contains a small demo (an `MyStore<T>` iterator example).
- To run code in another module you can:
  1) Open `src/main.rs` and call functions or demo code from the module you want to run, or
  2) Create a small binary under `examples/` (create `examples/<name>.rs`) and run it with `cargo run --example <name>`.

Project layout (high level)
- `src/` — source files and modules used for examples
  - `basic/` — language fundamentals and small tours
  - `data_structures/` — safe and unsafe implementations of linked lists, trees, graphs, maps, etc.
  - `algo/` — sorting, Fibonacci, N-Queens, and similar algorithmic examples
  - `advance/` — advanced topics: allocators, serialization, `Pin`, macros, coercions
  - `concurency/` — concurrency patterns, TCP server demo, and `crossbeam` usage
  - `smart_pointers/`, `std_traits/`, `design_patterns/`, `mistakes/` — focused topic modules

Dependencies
- See `Cargo.toml`. Notable crates used by some modules: `serde`, `rand`, `crossbeam-*`, `dashmap`, `sha2`, `thiserror`, `anyhow`, `log`, `serde_json`.

Learning path
- Recommended progression (same as before): start with `basic` and `smart_pointers`, then progress to type-system deep dives, safe data structures, error handling, unsafe internals, algorithms, and finally systems topics.

Contributing
- Feel free to open issues or PRs. Small improvements that help learners are welcome: clearer examples, more comments, or additional `examples/` binaries.

Notes for maintainers
- Because the repo prefers single-crate organization, adding runnable examples is best done via `examples/` or by adding small integration tests so contributors can run individual demos without editing `src/main.rs`.

License
- No license file is included in this repo. Add `LICENSE` if you want to make the content explicitly open-source.

---

If you'd like, I can also:
- add simple `examples/` binaries for a few modules (so they can be run via `cargo run --example <name>`),
- or generate a small CONTRIBUTING.md and a template `LICENSE` (MIT/Apache) for the project.
