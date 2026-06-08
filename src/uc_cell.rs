
// Cell<T> provides interior mutability over Copy types without mut borrows.
// Note: Only for Copy type!!! The mutability is done by Copy and Write.
// Cell<T> is the most lightweight form of interior mutability in Rust. It lets you mutate a value
// through an immutable reference (&self) without unsafe blocks, and with no runtime borrowing
// overhead. The cost: you can only get/set the whole value at once; you never get a reference
// to the inner data.

// How it works (concept)
// Cell<T> provides:
//
// get() – returns a copy of the value (requires T: Copy).
//
// set(val) – replaces the entire value with val (also needs T: Copy).
//
// take() – replaces the value with the default (e.g., None for Option<T>) and returns the old value.
// Works for non‑Copy types like Option<T>.
//
// replace(val) – swaps the value with val and returns the old value (works for any type that can
// be constructed, often using Default or similar).
//
// Because Cell never hands out a reference, there is no risk of aliasing violations. All accesses
// are through copying or swapping, which is automatically thread‑safe in single‑threaded contexts
// (and Cell itself is not Sync, so it stays on one thread).

// Simple mutable counters in shared (immutable) contexts.
// The classic example: a struct has an &self method that needs to update internal state, but you
// don’t want to change the method signature to &mut self (maybe because it’s a trait method that
// already uses &self, or many callers hold shared references).
fn mutable_counter() {
    use std::cell::Cell;

    struct Counter {
        // You need mutable state behind a &self method, and u32 is Copy and cheap to get/set.
        count: Cell<u32>,
    }

    impl Counter {
        fn increment(&self) {
            // self is &self, yet we can mutate the cell
            let old = self.count.get();
            self.count.set(old + 1);
        }

        fn value(&self) -> u32 {
            self.count.get()
        }
    }

    let c = Counter { count: Cell::new(0) };
    c.increment();
    c.increment();
    println!("{}", c.value()); // 2
    // No &mut needed – can be called while other shared references exist.
}

// Caching / Lazy computation behind a shared reference.
// Suppose you have an expensive computation and you want to cache the result in a struct. The
// struct is accessed via shared reference, so you can’t mutate a regular field. Cell<Option<T>>
// (or Cell<T> with a sentinel) lets you store the result inside &self.
fn caching_computation() {
    use std::cell::Cell;

    struct Expensive {
        // Cache the result of the expensive computation. Option<T> is not Copy, but Cell can still
        // manage it by swapping.
        cache: Cell<Option<u32>>,
    }

    impl Expensive {
        fn compute(&self) -> u32 {
            if let Some(result) = self.cache.get() {
                return result; // Return cached value if available.
            }
            // Simulate an expensive computation.
            let result = (0..1000000).sum();
            self.cache.set(Some(result)); // Cache the result for future calls.
            result
        }
    }

    let e = Expensive { cache: Cell::new(None) };
    println!("{}", e.compute()); // Computes and caches the result.
    println!("{}", e.compute()); // Returns cached result.
}

// Cancellation flag or simple shared boolean.
// A shared &Cell<bool> lets multiple readers check a condition while one writer sets it – all
// without &mut or locks.
fn share_flag() {
    use std::cell::Cell;

    struct Worker {
        // bool is Copy, trivial to get/set, and you want zero‑cost interior mutability with
        // no lock overhead.
        cancelled: Cell<bool>,
    }

    impl Worker {
        fn cancel(&self) {
            self.cancelled.set(true);
        }

        fn do_work(&self) {
            while !self.cancelled.get() {
                // process work units
            }
            println!("Cancelled");
        }
    }

    let w = Worker { cancelled: Cell::new(false) };
    // Somewhere else, w.cancel() can be called from a shared reference.
}

// Implementing the internal iterator pattern (closure state).
// When you pass a closure that needs mutable state, you can wrap the state in a Cell so the
// closure (which is Fn, not FnMut) can still mutate.
fn closure_interior_mutability() {
    use std::cell::Cell;

    fn apply_twice<F>(x: i32, f: F) -> i32
    where
        F: Fn(i32) -> i32, // Fn, not FnMut as Cell<T> provides interior mutability.
    {
        f(f(x))
    }

    let counter = Cell::new(0); // Cell wrapped state that are captured by the closure.
    let add_with_counter = |x| {
        counter.set(counter.get() + 1);
        x + counter.get()
    };

    let result = apply_twice(5, add_with_counter);
    // The closure is Fn (captures &Cell, which is immutable), yet it mutates the Cell.
}

// Cell<Option<T>>, One-time extraction.
// If you have an Option<T> inside a struct and you need to take the inner value out, leaving
// None behind – all through a shared reference – Cell’s take() is perfect.
fn onetime_extraction() {
    use std::cell::Cell;

    struct Connector {
        config: Cell<Option<String>>,
    }

    impl Connector {
        fn get_config(&self) -> Option<String> {
            // Takes the config out, replaces with None.
            self.config.take()
        }
    }

    let c = Connector { config: Cell::new(Some("url".into())) };
    println!("{:?}", c.get_config()); // Some("url")
    println!("{:?}", c.get_config()); // None
}