// Marker — Send/Sync for thread safety (auto-derived), Copy (also a marker).

// 1. Copy.
// Marker trait for types that can be duplicated by copying bits (stack-only).
// Types implementing Copy are moved by copying, not transferring ownership.
// Cannot be implemented for heap-owning types.
fn copy() {
    // Only for small, stack-resident types:
    #[derive(Debug, Clone, Copy)]
    struct Point { x: f32, y: f32 }

    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;          // copy — p1 still usable
    println!("{p1:?}");   // ✅ p1 still valid

    // Primitives are Copy: i32, f64, bool, char, &T, fn pointers
    let x: i32 = 5;
    let y = x;            // copy
    println!("{x}");      // ✅

    // Cannot implement Copy for heap-owning types:
    // #[derive(Copy)] on String → compile error
    // String owns heap memory — copy semantics would alias the pointer
}

// 2. Send/Sync.
// Auto-traits for thread safety. Send: safe to move to another thread.
// Sync: safe to share a reference across threads (&T is Send iff T is Sync).
// Almost always auto-derived by the compiler.
fn send_sync() {
    // Automatically implemented when all fields are Send/Sync:
    #[derive(Debug)]
    struct Config { host: String, port: u16 }
    // Config is Send + Sync automatically (String and u16 are both)

    use std::sync::{Arc};
    use std::thread;

    let config = Arc::new(Config { host: "localhost".into(), port: 8080 });
    let config2 = Arc::clone(&config);

    thread::spawn(move || {
        println!("{}", config2.host);  // ✅ Arc is Send
    });

    // Rc is NOT Send — can't cross thread boundary:
    // thread::spawn(move || { let _ = Rc::new(1); }); // compile error

    // Manually opt OUT of Send/Sync for a type with raw pointers:
    use std::marker::PhantomData;
    struct NotSend(PhantomData<*mut u8>);  // *mut T is not Send
}

