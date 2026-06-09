use std::marker::PhantomPinned;
use std::pin::{pin, Pin};

// Pin and Unpin solve one fundamental problem in Rust: safe manipulation of data that cannot be
// moved in memory. Rust’s ownership model assumes that values can be moved freely
// (just a memcpy, leaving the old location invalid). But some data structures contain
// self-referential pointers that become dangling if the value is moved. Pin prevents moves,
// and Unpin marks types that are safe to move even when pinned.

// async / await and Futures (the primary motivation).
// Every async block or async fn compiles into an anonymous future – a state machine that holds
// local variables across .await points.
//
// rust
// async fn example() {
//     let mut x = vec![1, 2, 3];
//     let y = &x;
//     do_something().await; // yields, then resumes
//     println!("{:?}", y);
// }
// The generated future may store y as a pointer into the future’s own struct
// (it borrows x, which is itself a field of the future). This makes the future self-referential.
// If the future were moved in memory after starting, y would become dangling → undefined behaviour.
//
// The future produced by async blocks is !Unpin by default.
//
// The executor pins the future (using Box::pin or pin!) before polling it, guaranteeing that
// once started, the future never moves.
//
// Unpin is not implemented for these futures, so the compiler prevents moving them after they
// are pinned.
//
// This is the most widespread use of Pin: every asynchronous program in Rust relies on it to
// safely handle suspending and resuming self-referential futures.


// Self-referential data structures.
// Any struct that stores a pointer to another field of itself needs Pin to be used safely if you
// ever want to move it after construction.
fn self_referential_struct() {
    use std::pin::Pin;

    struct SelfRef {
        data: String,
        // The pointer is *into* `data`
        ptr: *const u8,
    }

    impl SelfRef {
        fn new(s: String) -> Pin<Box<Self>> {
            // We must first create the Box, then set the pointer.
            let mut boxed = Box::pin(Self { data: s, ptr: std::ptr::null() });
            let ptr = boxed.data.as_ptr();
            // Safe because the Box is pinned (won't move)
            unsafe { Pin::get_mut(boxed.as_mut()).ptr = ptr; }
            boxed
        }
    }
}

// ---------------------------------------------------------------------------
// 1. A self-referential type that opts out of Unpin
// ---------------------------------------------------------------------------
struct SelfRef {
    data: String,
    ptr: *const String,     // raw pointer into `data`
    _pin: PhantomPinned,    // make SelfRef !Unpin
}

impl SelfRef {
    fn new(data: String) -> Self {
        Self {
            data,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        }
    }

    /// Only callable when the value is pinned.
    fn init(self: Pin<&mut Self>) {
        // We use `get_unchecked_mut` to access the inner `&mut Self`.
        // This is safe because we uphold the pinning contract:
        // the value will never be moved again after this call.
        let this = unsafe { self.get_unchecked_mut() };
        this.ptr = &this.data as *const String;
    }

    /// Returns a reference to `data` through the stored pointer.
    fn get_data(self: Pin<&Self>) -> Option<&String> {
        let this = self.get_ref();
        if this.ptr.is_null() {
            None
        } else {
            // Safe: `ptr` remains valid as long as the value is pinned.
            unsafe { Some(&*this.ptr) }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Illustrate the Unpin auto-trait
// ---------------------------------------------------------------------------
fn assert_unpin<T: Unpin>(name: &str) {
    println!("✅ {name} implements Unpin");
}

fn assert_not_unpin<T>(name: &str) {
    // We can't directly check !Unpin, but we can see if <T as Unpin> fails.
    // Instead, we'll just print the information we already know.
    println!("❌ {name} does NOT implement Unpin");
}

// ---------------------------------------------------------------------------
// 3. The main demonstration
// ---------------------------------------------------------------------------
fn main() {
    println!("=== PIN & UNPIN DEMONSTRATION ===\n");

    // --------------------------------------------------
    // a) Heap pinning with Box::pin
    // --------------------------------------------------
    println!("--- Heap pinning (Box::pin) ---");
    let mut s = Box::pin(SelfRef::new("hello from the heap".into()));

    // Before init, get_data returns None
    assert!(s.as_ref().get_data().is_none());

    // Initialize the self-referential pointer while pinned
    s.as_mut().init();

    // Now it's valid
    println!("data = {:?}", s.as_ref().get_data());

    // We CANNOT move `s` out of the Pin<Box<SelfRef>> –
    // the compiler prevents it.
    // let moved = *s;  // ❌ would not compile

    // --------------------------------------------------
    // b) Stack pinning with the `pin!` macro (stable since 1.68)
    // --------------------------------------------------
    println!("\n--- Stack pinning (pin! macro) ---");
    let mut local = SelfRef::new("hello from the stack".into());
    let mut pinned = pin!(local);       // Pin<&mut SelfRef>
    assert!(pinned.as_ref().get_data().is_none());
    pinned.as_mut().init();
    println!("data = {:?}", pinned.as_ref().get_data());
    // `local` (now pinned) cannot be moved until `pinned` goes out of scope.

    // --------------------------------------------------
    // c) Unpin types: Pin behaves mostly like a normal reference
    // --------------------------------------------------
    println!("\n--- Unpin type (i32) ---");
    let mut x: i32 = 42;
    let mut p = Pin::new(&mut x);   // Pin<&mut i32>
    *p = 100;                       // we can mutate through Pin
    // todo: fix this
    //println!("x = {}", x);        // 100

    // Because i32 is Unpin, we can safely get a `&mut i32` back
    let normal_ref: &mut i32 = Pin::into_inner(p);
    *normal_ref = 7;
    println!("x after into_inner = {}", x); // 7

    // --------------------------------------------------
    // d) Trait bound check
    // --------------------------------------------------
    println!("\n--- Unpin trait checks ---");
    assert_unpin::<i32>("i32");
    assert_unpin::<String>("String");
    assert_not_unpin::<SelfRef>("SelfRef");

    // --------------------------------------------------
    // e) The real-world motivation: async futures (comments only)
    // --------------------------------------------------
    println!("\n--- Async state machine (conceptual) ---");
    println!(
        "When you write an 'async fn', the compiler generates a future\n\
         that is !Unpin if it holds references across an .await point.\n\
         These futures must be pinned before polling, usually via Box::pin,\n\
         to prevent moves that would invalidate those internal references.\n\
         This is the main reason Pin exists in Rust."
    );
}