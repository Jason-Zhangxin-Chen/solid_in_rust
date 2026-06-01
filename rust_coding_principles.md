Here are the most important principles to keep in mind when writing Rust code:

## Ownership & Borrowing (Rust's Core)
- **Every value has one owner** — when the owner goes out of scope, the value is dropped.
- **Borrow rules**: you can have *either* one mutable reference *or* any number of immutable references — never both at the same time.
- Prefer borrowing (`&T`) over cloning unless you truly need ownership.

## Embrace the Type System
- Use the type system to **make illegal states unrepresentable** — encode invariants in types, not runtime checks.
- Prefer `enum` with variants over boolean flags or stringly-typed logic.
- Newtype pattern (`struct Meters(f64)`) to prevent mixing incompatible values.

## Error Handling
- Use `Result<T, E>` for recoverable errors — never ignore them.
- Use `?` to propagate errors cleanly.
- Avoid `unwrap()` and `expect()` in production code; reserve them for prototypes or truly infallible cases.
- Define meaningful error types (or use `thiserror`/`anyhow` crates).

## Prefer Explicitness Over Magic
- Rust has no hidden control flow — be explicit about lifetimes, mutability (`mut`), and conversions.
- Avoid `unsafe` unless absolutely necessary, and isolate it when you must use it.
- Don't fight the borrow checker — if something feels hard, reconsider the design.

## Zero-Cost Abstractions
- Use iterators and closures freely — they compile down to the same machine code as hand-written loops.
- Avoid unnecessary heap allocations (`Box`, `Vec`, `String`) when stack types or slices suffice.
- Understand `&str` vs `String` and `&[T]` vs `Vec<T>`.

## Concurrency Safety
- Rust's type system prevents data races at compile time — trust it.
- Prefer message passing (`std::sync::mpsc`, channels) over shared state where possible.
- When sharing state, use `Arc<Mutex<T>>` correctly and keep locks short-lived.

## Idiomatic Style
- Follow `rustfmt` formatting and `clippy` lint suggestions — they encode community best practices.
- Implement standard traits (`Display`, `Debug`, `From`, `Into`, `Iterator`) where appropriate.
- Use `impl Trait` in function signatures to keep APIs flexible.

## Design for Compile-Time Correctness
- Validate at construction time (constructors that return `Result`), not at use time.
- Use the **typestate pattern** to enforce state machine transitions through the type system.
- Think in terms of *what the compiler can guarantee*, not just what the runtime will do.

The overarching philosophy is: **if it compiles, it probably works correctly** — Rust's whole design pushes bugs from
runtime into compile time. Trust and work *with* the type system rather than against it.