// Memory — Clone for explicit heap copy, Copy for implicit bit copy,
// Drop for custom destructors, Default for zero values.

// 1. Clone, Explicit heap copy. Call .clone() to duplicate a value. Unlike Copy,
// Clone can run arbitrary code (re-allocate, deep-copy). Always derive unless you need custom logic.
fn clone() {
    // Derive for most types:
    #[derive(Clone, Debug)]
    struct Config { host: String, timeout: u32 }

    let c1 = Config { host: "localhost".into(), timeout: 30 };
    let c2 = c1.clone();   // deep copy — c1 still valid

    struct MyBuffer {
        data: Vec<u8>,
        len: usize,
    }

    // Custom Clone — e.g. for a type with raw pointers:
    impl Clone for MyBuffer {
        fn clone(&self) -> Self {
            MyBuffer { data: self.data.clone(), len: self.len }
        }
    }

    // Clone vs Copy:
    let s1 = String::from("hi");
    let s2 = s1.clone();  // explicit — clear that allocation happens
    let n1: i32 = 5;
    let n2 = n1;          // implicit Copy — no clone needed
}

// 2. Copy, Implicit bit copy. Types that implement Copy can be duplicated by simple bitwise copy.
fn copy() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Point { x: i32, y: i32 }

    let p1 = Point { x: 1, y: 2 };
    let p2 = p1;  // implicit Copy — no clone needed
}

// 3. Drop.
// Custom destructor. Called automatically when a value goes out of scope.
// Use to release resources (file handles, network connections, locks) that Rust doesn't know about.
fn drop() {
    struct FileHandle { path: String }

    impl Drop for FileHandle {
        fn drop(&mut self) {
            println!("closing file: {}", self.path);
            // release OS handle here
        }
    }

    {
        let f = FileHandle { path: "data.txt".into() };
        println!("using file");
    }   // Drop::drop called here automatically — "closing file: data.txt"
}

// 4. Default.
// Provides a sensible zero/empty value for a type. Used by struct update syntax,
// Option::unwrap_or_default(), HashMap::entry().or_default(), and many builder patterns.
fn default() {
    use std::collections::HashMap;
    #[derive(Debug, Default)]
    struct Config {
        host: String,    // ""
        port: u16,       // 0
        retries: u32,    // 0
        verbose: bool,   // false
    }

    let c = Config::default();
    println!("{c:?}");  // Config { host: "", port: 0, ... }

    // Struct update syntax — fill in the rest with defaults:
    let c = Config { port: 8080, ..Config::default() };

    // entry().or_default() — insert default if missing:
    let mut counts: std::collections::HashMap<&str, u32> = HashMap::new();
    *counts.entry("hello").or_default() += 1;  // inserts 0, then adds 1
}

