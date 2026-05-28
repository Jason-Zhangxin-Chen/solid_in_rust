// smart_pointer_methods.rs
// Complete public API demo for every major Rust smart pointer / wrapper type.
// Compile: rustc --edition 2021 smart_pointer_methods.rs && ./smart_pointer_methods

#![allow(dead_code, unused_variables, unused_mut, clippy::all)]

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Mutex, Weak as ArcWeak};

fn main() {
    option_methods();
    result_methods();
    box_methods();
    rc_methods();
    arc_methods();
    cell_methods();
    refcell_methods();
    cow_methods();
}

// =============================================================================
// Option<T>
// =============================================================================
// Represents an optional value: either Some(T) or None.
// Zero-cost abstraction — no heap allocation, no runtime overhead.
// Use instead of null pointers or sentinel values.
// =============================================================================
fn option_methods() {
    println!("\n========== Option<T> ==========");

    let some: Option<i32> = Some(42);
    let none: Option<i32> = None;

    // ----- Querying -----

    assert_eq!(some.is_some(), true);           // true if Some(_)
    assert_eq!(none.is_none(), true);           // true if None
    assert_eq!(some.is_some_and(|x| x > 0), true); // is_some AND predicate holds

    // ----- Unwrapping (extracting the value) -----

    let v = some.unwrap();                     // 42 — PANICS if None
    let v = none.unwrap_or(0);          // 0  — safe default value
    let v = none.unwrap_or_else(|| 99);        // 99 — lazy: closure only runs if None
    let v = none.unwrap_or_default();          // 0  — uses i32::default()

    // expect: like unwrap but with a custom panic message
    let v = some.expect("value must exist");   // 42

    // ----- Transforming (mapping) -----

    // map: transform Some(T) → Some(U), None stays None
    let doubled: Option<i32> = some.map(|x| x * 2);        // Some(84)
    let doubled: Option<i32> = none.map(|x| x * 2);        // None — closure not called

    // map_or: map + unwrap_or in one step
    let v: i32 = some.map_or(0, |x| x + 1);               // 43
    let v: i32 = none.map_or(0, |x| x + 1);               // 0

    // map_or_else: lazy map_or — both arms are closures
    let v: i32 = some.map_or_else(|| 0, |x| x + 1);       // 43
    let v: i32 = none.map_or_else(|| 0, |x| x + 1);       // 0

    // ----- Chaining (flatMap equivalent) -----

    // and_then: like map but the closure returns Option — used for fallible chains
    let r = some.and_then(|x| if x > 0 { Some(x * 2) } else { None }); // Some(84)
    let r = none.and_then(|x| Some(x * 2));                             // None

    // and: replaces Some(_) with another Option, None stays None
    let r = some.and(Some("hello"));     // Some("hello")
    let r = none.and(Some("hello"));    // None

    // or: provides a fallback Option when None
    let r = none.or(Some(7));           // Some(7)
    let r = some.or(Some(7));           // Some(42) — first Some wins

    // or_else: lazy fallback — closure only runs if None
    let r = none.or_else(|| Some(99)); // Some(99)

    // ----- Filtering -----

    // filter: Some(x) → None if predicate fails, Some(x) if it passes
    let r = some.filter(|&x| x > 100); // None  — 42 not > 100
    let r = some.filter(|&x| x > 10);  // Some(42)

    // ----- Converting to/from Result -----

    let r: Result<i32, &str> = some.ok_or("missing");         // Ok(42)
    let r: Result<i32, &str> = none.ok_or("missing");         // Err("missing")
    let r: Result<i32, String> = none.ok_or_else(|| "missing".to_string()); // lazy Err

    // ----- Reference adapters -----

    // as_ref: Option<T> → Option<&T>  (borrow the inner value)
    let opt_ref: Option<&i32> = some.as_ref();

    // as_mut: Option<T> → Option<&mut T>
    let mut opt2 = Some(42i32);
    let opt_mut: Option<&mut i32> = opt2.as_mut();
    if let Some(v) = opt_mut { *v += 1; }
    assert_eq!(opt2, Some(43));

    // as_deref: Option<String> → Option<&str>  (deref the inner value)
    let s: Option<String> = Some("hello".to_string());
    let s_ref: Option<&str> = s.as_deref();                // Some("hello")

    // as_deref_mut: mutable version of as_deref
    let mut s2: Option<String> = Some("hello".to_string());
    if let Some(r) = s2.as_deref_mut() { r.make_ascii_uppercase(); }

    // ----- Mutation helpers -----

    // take: moves the value out, leaving None behind
    let mut opt = Some(10);
    let taken = opt.take();                 // taken=Some(10), opt=None

    // replace: swaps in a new value, returns the old one
    let mut opt = Some(1);
    let old = opt.replace(2);       // old=Some(1), opt=Some(2)

    // get_or_insert: insert a value if None, return &mut to the inner value
    let mut opt: Option<i32> = None;
    let r: &mut i32 = opt.get_or_insert(5);    // opt=Some(5), r=&mut 5
    *r += 1;
    assert_eq!(opt, Some(6));

    // get_or_insert_with: lazy version of get_or_insert
    let mut opt: Option<i32> = None;
    opt.get_or_insert_with(|| 99);

    // insert: always inserts (overwrites), returns &mut T
    let mut opt: Option<i32> = None;
    let r = opt.insert(42);                    // opt=Some(42)

    // ----- Combinators -----

    // zip: combines two Options into Option<(A, B)>
    let a: Option<i32>  = Some(1);
    let b: Option<&str> = Some("hi");
    let zipped = a.zip(b);                     // Some((1, "hi"))
    let neither = a.zip(None::<&str>);         // None — one is None

    // unzip: Option<(A,B)> → (Option<A>, Option<B>)
    let (x, y): (Option<i32>, Option<&str>) = zipped.unzip();

    // flatten: Option<Option<T>> → Option<T>
    let nested: Option<Option<i32>> = Some(Some(42));
    let flat: Option<i32> = nested.flatten();  // Some(42)
    let nested2: Option<Option<i32>> = Some(None);
    let flat2 = nested2.flatten();             // None

    // transpose: Option<Result<T,E>> ↔ Result<Option<T>,E>
    let or: Option<Result<i32, &str>> = Some(Ok(1));
    let ro: Result<Option<i32>, &str> = or.transpose(); // Ok(Some(1))

    // ----- Iteration -----

    // Option implements IntoIterator — useful in flat_map chains
    let v: Vec<i32> = some.into_iter().collect();  // [42]
    let v: Vec<i32> = none.into_iter().collect();  // []

    // iter() — borrow as a single-element iterator
    for val in some.iter() { println!("{val}"); }

    // ----- Pattern matching (most idiomatic) -----
    match some {
        Some(v) if v > 0 => println!("positive {v}"),
        Some(v)           => println!("non-positive {v}"),
        None              => println!("nothing"),
    }
    if let Some(v) = some    { println!("got {v}"); }
    let Some(v) = some else  { return; };          // let-else: early return on None

    println!("option_methods done ✓");
}

// =============================================================================
// Result<T, E>
// =============================================================================
// Represents success Ok(T) or failure Err(E).
// The primary error-handling type in Rust — use instead of exceptions.
// =============================================================================
fn result_methods() {
    println!("\n========== Result<T, E> ==========");

    let ok:  Result<i32, &str> = Ok(42);
    let err: Result<i32, &str> = Err("oops");

    // ----- Querying -----

    assert_eq!(ok.is_ok(),   true);
    assert_eq!(err.is_err(), true);
    assert_eq!(ok.is_ok_and(|x| x > 0), true);    // is_ok AND predicate holds
    assert_eq!(err.is_err_and(|e| !e.is_empty()), true);

    // ----- Unwrapping -----

    let v = ok.unwrap();                            // 42 — PANICS on Err
    let v = err.unwrap_or(0);                // 0
    let v = err.unwrap_or_else(|e| e.len() as i32); // 4 (length of "oops")
    let v = err.unwrap_or_default();               // 0 (i32::default())
    let v = ok.expect("should have value");    // 42 — custom panic msg

    // unwrap_err: extracts Err value, panics on Ok
    let e = err.unwrap_err();                      // "oops"

    // ----- Transforming the Ok side -----

    // map: Ok(T) → Ok(U), Err passes through unchanged
    let doubled: Result<i32, &str>  = ok.map(|x| x * 2);   // Ok(84)
    let doubled: Result<i32, &str>  = err.map(|x| x * 2);  // Err("oops")

    // ----- Transforming the Err side -----

    // map_err: Err(E) → Err(F), Ok passes through unchanged
    let mapped: Result<i32, String> = err.map_err(|e| format!("ERROR: {e}")); // Err("ERROR: oops")
    let mapped: Result<i32, String> = ok.map_err(|e| format!("ERROR: {e}")); // Ok(42)

    // ----- map_or / map_or_else -----

    let v: i32 = ok.map_or(0, |x| x + 1);          // 43
    let v: i32 = err.map_or(0, |x| x + 1);         // 0
    let v: i32 = ok.map_or_else(|_| 0, |x| x + 1); // 43 — both arms are closures

    // ----- Chaining -----

    // and_then: chain fallible steps — short-circuits on first Err
    let r = ok.and_then(|x| if x > 0 { Ok(x * 2) } else { Err("negative") }); // Ok(84)
    let r = err.and_then(|x| Ok(x * 2));           // Err("oops") — closure never runs

    // and: replaces Ok(_) with another Result
    let r: Result<&str, &str> = ok.and(Ok("done")); // Ok("done")
    let r: Result<&str, &str> = err.and(Ok("done")); // Err("oops")

    // or: recover from Err with a fallback Result
    let r: Result<i32, &str> = err.or(Ok(0));      // Ok(0)
    let r: Result<i32, &str> = ok.or(Ok(0));       // Ok(42) — first Ok wins

    // or_else: lazy recovery — closure only runs on Err
    let r: Result<i32, usize> = err.or_else(|e| Ok(e.len() as i32)); // Ok(4)

    // ----- Converting to Option -----

    let opt: Option<i32>  = ok.ok();    // Some(42) — discards Err
    let opt: Option<i32>  = err.ok();   // None
    let opt: Option<&str> = err.err();  // Some("oops") — discards Ok
    let opt: Option<&str> = ok.err();   // None

    // ----- Reference adapters -----

    let r: Result<&i32, &&str> = ok.as_ref();   // borrows Ok/Err inner value
    let mut ok2: Result<i32, &str> = Ok(42);
    let r: Result<&mut i32, &mut &str> = ok2.as_mut();
    if let Ok(v) = r { *v += 1; }
    assert_eq!(ok2, Ok(43));

    // as_deref / as_deref_mut — deref the inner value
    let ok_s: Result<String, &str> = Ok("hello".to_string());
    let r: Result<&str, &&str> = ok_s.as_deref(); // Ok("hello")

    // ----- Mutation helpers -----

    // transpose: Result<Option<T>,E> ↔ Option<Result<T,E>>
    let ro: Result<Option<i32>, &str> = Ok(Some(1));
    let or: Option<Result<i32, &str>> = ro.transpose(); // Some(Ok(1))

    // flatten: Result<Result<T,E>,E> → Result<T,E>
    let nested: Result<Result<i32, &str>, &str> = Ok(Ok(42));
    let flat: Result<i32, &str> = nested.flatten(); // Ok(42)

    // iter / iter_mut — iterate over Ok value (0 or 1 elements)
    for v in ok.iter() { println!("{v}"); }

    // into_ok / into_err — only available on infallible Results
    // let v: i32 = Ok::<i32, std::convert::Infallible>(1).into_ok();

    // ----- The ? operator — most important pattern -----
    fn parse_double(s: &str) -> Result<i32, std::num::ParseIntError> {
        let n: i32 = s.trim().parse()?;    // ? unwraps Ok or returns Err early
        Ok(n * 2)
    }
    println!("{:?}", parse_double("21")); // Ok(42)
    println!("{:?}", parse_double("x"));  // Err(ParseIntError)

    // ----- Collecting Results -----
    // Iterator<Item=Result<T,E>>.collect() → Result<Vec<T>,E>
    // Fails fast on first Err:
    let nums: Result<Vec<i32>, _> = ["1","2","3"]
        .iter().map(|s| s.parse::<i32>()).collect();
    assert_eq!(nums, Ok(vec![1,2,3]));

    let bad: Result<Vec<i32>, _> = ["1","x","3"]
        .iter().map(|s| s.parse::<i32>()).collect();
    assert!(bad.is_err());

    // ----- let-else (early exit on Err) -----
    let Ok(val) = parse_double("10") else { return; };
    assert_eq!(val, 20);

    println!("result_methods done ✓");
}

// =============================================================================
// Box<T>
// =============================================================================
// Owned heap allocation with single ownership.
// No ref-counting — exactly as fast as a raw pointer.
// Use for: large values, recursive types, trait objects (dyn Trait).
// =============================================================================
fn box_methods() {
    println!("\n========== Box<T> ==========");

    // ----- Construction -----
    let b: Box<i32> = Box::new(42);

    // Box::default — requires T: Default
    let b: Box<i32> = Box::default();              // Box(0)

    // Box::pin — pin data to a stable memory address (needed for self-referential types)
    let pinned: std::pin::Pin<Box<i32>> = Box::pin(42);

    // ----- Accessing -----
    let b = Box::new(42i32);
    println!("{}", *b);                             // Deref coercion: 42
    println!("{b}");                               // Display also works

    // as_ref / as_mut — get &T or &mut T
    let r: &i32 = b.as_ref();
    let mut b2 = Box::new(10i32);
    let r: &mut i32 = b2.as_mut();
    *r = 20;
    assert_eq!(*b2, 20);

    // ----- Moving out of Box -----
    let b = Box::new(String::from("hello"));
    let s: String = *b;                            // unbox — moves value out, Box is consumed

    // ----- Checking the pointer -----
    let b = Box::new(42i32);
    // leak: consume the Box, return a &'static mut T — memory never freed!
    // Use for intentional static allocation (e.g., global resources).
    let leaked: &'static mut i32 = Box::leak(b);
    *leaked = 99;

    // ----- Raw pointer round-trip (for FFI / unsafe code) -----
    let b = Box::new(42i32);
    let raw: *mut i32 = Box::into_raw(b);          // Box gives up ownership
    unsafe {
        *raw += 1;
        let _b = Box::from_raw(raw);               // reclaim — Box frees on drop
    }

    // ----- Recursive types (Box breaks infinite-size cycle) -----
    enum List<T> {
        Cons(T, Box<List<T>>),
        Nil,
    }
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));

    // ----- Trait objects: Box<dyn Trait> -----
    trait Speak { fn speak(&self) -> &str; }
    struct Dog; struct Cat;
    impl Speak for Dog { fn speak(&self) -> &str { "woof" } }
    impl Speak for Cat { fn speak(&self) -> &str { "meow" } }

    let animals: Vec<Box<dyn Speak>> = vec![Box::new(Dog), Box::new(Cat)];
    for a in &animals { println!("{}", a.speak()); }

    // ----- Box<dyn Error> — common in fallible functions -----
    fn might_fail(fail: bool) -> Result<(), Box<dyn std::error::Error>> {
        if fail { return Err("something went wrong".into()); }
        Ok(())
    }
    println!("{:?}", might_fail(false)); // Ok(())
    println!("{:?}", might_fail(true));  // Err("something went wrong")

    println!("box_methods done ✓");
}

// =============================================================================
// Rc<T> and Rc::Weak<T>
// =============================================================================
// Rc: Reference-counted shared ownership — single-threaded only.
// Weak: Non-owning reference — does NOT prevent deallocation.
// Use Rc when multiple owners are needed but threads are not involved.
// Use Weak to break reference cycles (parent ↔ child, observer pattern).
// =============================================================================
fn rc_methods() {
    println!("\n========== Rc<T> + Weak<T> ==========");

    // ----- Construction -----
    let a: Rc<i32> = Rc::new(42);

    // Rc::default — requires T: Default
    let b: Rc<i32> = Rc::default();               // Rc(0)

    // Rc::pin — pin to stable address
    let p: std::pin::Pin<Rc<i32>> = Rc::pin(42);

    // Rc::new_cyclic — create Rc that knows its own Weak during construction
    // Useful for self-referential structures
    let a: Rc<i32> = Rc::new_cyclic(|_weak| 42);

    // ----- Cloning = sharing (ref count bump, NOT a deep copy) -----
    let a = Rc::new(42);
    let b = Rc::clone(&a);                         // same heap object, count = 2
    let c = a.clone();                             // identical — both idioms are fine

    // ----- Reference counts -----
    println!("strong: {}", Rc::strong_count(&a)); // 3
    println!("weak:   {}", Rc::weak_count(&a));   // 0

    // ----- Accessing -----
    println!("{}", *a);                           // Deref: 42
    let r: &i32 = a.as_ref();                     // &T

    // ----- Pointer identity -----
    // ptr_eq: are two Rcs pointing at the same allocation?
    let x = Rc::new(1);
    let y = Rc::clone(&x);
    let z = Rc::new(1);
    assert!( Rc::ptr_eq(&x, &y));               // same object
    assert!(!Rc::ptr_eq(&x, &z));               // different objects, same value

    // as_ptr: raw pointer to the inner value (does NOT transfer ownership)
    let raw: *const i32 = Rc::as_ptr(&x);

    // ----- Mutation (only when exclusively owned) -----

    // get_mut: &mut T only if strong_count == 1
    let mut sole = Rc::new(10);
    if let Some(v) = Rc::get_mut(&mut sole) {
        *v = 99;                                  // safe — we're the only owner
    }
    let _shared = Rc::clone(&sole);
    assert!(Rc::get_mut(&mut sole).is_none());   // None — now shared

    // make_mut: clone-on-write — clones inner value if shared, gives &mut T
    let mut p = Rc::new(vec![1, 2, 3]);
    let _q = Rc::clone(&p);
    Rc::make_mut(&mut p).push(4);                // p is now a new clone: [1,2,3,4]
    // _q still sees [1, 2, 3]

    // ----- Unwrapping -----

    // try_unwrap: Ok(T) if we're the last strong owner, Err(Rc<T>) otherwise
    let sole = Rc::new(99);
    match Rc::try_unwrap(sole) {
        Ok(val)  => println!("unwrapped: {val}"),  // prints this
        Err(rc)  => println!("still shared"),
    }

    // unwrap_or_clone: unwrap if last owner, else clone the inner value
    let a = Rc::new(String::from("hello"));
    let _b = Rc::clone(&a);
    let s: String = Rc::unwrap_or_clone(a);        // clones because _b still exists

    // ----- Weak references -----

    let strong = Rc::new(42);
    let weak: RcWeak<i32> = Rc::downgrade(&strong); // weak count++ but strong count unchanged

    println!("strong={} weak={}", Rc::strong_count(&strong), Rc::weak_count(&strong)); // 1, 1

    // upgrade: returns Some(Rc<T>) while at least one strong owner lives
    if let Some(val) = weak.upgrade() {
        println!("alive: {}", *val);              // 42
    }

    drop(strong);                                  // strong count → 0, allocation freed

    assert!(weak.upgrade().is_none());            // None — gone

    // Weak::new: a Weak that always upgrades to None
    let dangling: RcWeak<i32> = RcWeak::new();
    assert!(dangling.upgrade().is_none());

    // Weak::ptr_eq, as_ptr, strong_count, weak_count
    let a = Rc::new(1);
    let w1 = Rc::downgrade(&a);
    let w2 = Rc::downgrade(&a);
    assert!(RcWeak::ptr_eq(&w1, &w2));
    let _ = w1.as_ptr();                          // raw *const T (may be dangling)
    println!("weak strong_count: {}", w1.strong_count()); // 1
    println!("weak weak_count:   {}", w1.weak_count());   // 2

    // ----- Classic pattern: Rc<RefCell<T>> for shared mutation -----
    let shared = Rc::new(RefCell::new(vec![1, 2]));
    let clone  = Rc::clone(&shared);
    clone.borrow_mut().push(3);
    println!("{:?}", shared.borrow());            // [1, 2, 3]

    // ----- Cycle breaking pattern -----
    struct Parent { child: Option<Rc<Child>> }
    struct Child  { parent: RcWeak<Parent>   }    // Weak breaks the cycle

    let parent = Rc::new(Parent { child: None });
    let child  = Rc::new(Child  { parent: Rc::downgrade(&parent) });
    // When parent drops, strong count hits 0 even though child holds a Weak.

    println!("rc_methods done ✓");
}

// =============================================================================
// Arc<T> and Arc::Weak<T>
// =============================================================================
// Arc: Atomically reference-counted shared ownership — thread-safe.
// Same API as Rc but uses atomic operations (slightly more expensive).
// Combine with Mutex<T> or RwLock<T> for shared mutable state across threads.
// =============================================================================
fn arc_methods() {
    println!("\n========== Arc<T> + Weak<T> ==========");

    // ----- Construction -----
    let a: Arc<i32> = Arc::new(42);
    let b: Arc<i32> = Arc::default();             // Arc(0)
    let p: std::pin::Pin<Arc<i32>> = Arc::pin(42);

    // Arc::new_cyclic — like Rc::new_cyclic
    let a: Arc<i32> = Arc::new_cyclic(|_weak| 42);

    // ----- Cloning = sharing -----
    let a = Arc::new(42);
    let b = Arc::clone(&a);               // atomic ref count bump

    println!("strong: {}", Arc::strong_count(&a)); // 2
    println!("weak:   {}", Arc::weak_count(&a));   // 0

    // ----- Accessing -----
    println!("{}", *a);                            // Deref: 42
    let r: &i32 = a.as_ref();
    let raw: *const i32 = Arc::as_ptr(&a);

    // ----- Pointer identity -----
    let x = Arc::new(1);
    let y = Arc::clone(&x);
    assert!(Arc::ptr_eq(&x, &y));

    // ----- Mutation -----

    // get_mut: Some only when strong_count == 1 (no other owners)
    let mut sole = Arc::new(10);
    if let Some(v) = Arc::get_mut(&mut sole) { *v = 99; }

    // make_mut: clone-on-write (requires T: Clone)
    let mut p = Arc::new(vec![1, 2, 3]);
    let _q = Arc::clone(&p);
    Arc::make_mut(&mut p).push(4);                // new clone for p

    // try_unwrap / unwrap_or_clone — same as Rc
    let sole = Arc::new(String::from("hi"));
    let s: String = Arc::try_unwrap(sole).unwrap();
    let a = Arc::new(String::from("hi"));
    let _b = Arc::clone(&a);
    let s: String = Arc::unwrap_or_clone(a);

    // ----- Weak references -----
    let strong = Arc::new(42);
    let weak: ArcWeak<i32> = Arc::downgrade(&strong);

    // upgrade, ptr_eq, as_ptr, strong_count, weak_count — same API as Rc::Weak
    assert!(weak.upgrade().is_some());
    drop(strong);
    assert!(weak.upgrade().is_none());

    let dangling: ArcWeak<i32> = ArcWeak::new();
    assert!(dangling.upgrade().is_none());

    // ----- Thread-safe shared mutation: Arc<Mutex<T>> -----
    let counter = Arc::new(Mutex::new(0i32));
    let mut handles = Vec::new();

    for _ in 0..4 {
        let c = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || {
            let mut guard = c.lock().unwrap();
            *guard += 1;
            // guard (MutexGuard) drops here — lock released automatically
        }));
    }
    for h in handles { h.join().unwrap(); }
    println!("counter: {}", *counter.lock().unwrap()); // 4

    // ----- Thread-safe shared read: Arc<RwLock<T>> -----
    use std::sync::RwLock;
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let d = Arc::clone(&data);
    let h = std::thread::spawn(move || {
        let read = d.read().unwrap();             // many readers at once
        println!("reader sees: {:?}", *read);
    });
    data.write().unwrap().push(4);               // exclusive writer
    h.join().unwrap();

    // ----- Passing Arc across threads -----
    let shared = Arc::new(String::from("hello"));
    let s = Arc::clone(&shared);
    std::thread::spawn(move || println!("{s}")).join().unwrap();

    println!("arc_methods done ✓");
}

// =============================================================================
// Cell<T>
// =============================================================================
// Interior mutability for Copy types — no borrow checking, no locking.
// Use when you need to mutate through &self and T is Copy.
// Zero runtime cost — compiles to plain loads/stores.
// =============================================================================
fn cell_methods() {
    println!("\n========== Cell<T> ==========");

    let c: Cell<i32> = Cell::new(0);

    // ----- Core get / set (the primary API) -----
    c.set(42);                                    // overwrite value
    let v: i32 = c.get();                        // copy value out
    assert_eq!(v, 42);

    // ----- Update: get + transform + set -----
    c.update(|v| v + 1);                         // 43
    assert_eq!(c.get(), 43);

    // ----- Swap two Cells -----
    let d = Cell::new(100);
    c.swap(&d);
    assert_eq!(c.get(), 100);
    assert_eq!(d.get(), 43);

    // ----- Replace: set new value, return old -----
    let old = c.replace(7);                       // old=100, c=7
    assert_eq!(old, 100);
    assert_eq!(c.get(), 7);

    // ----- Take: move value out, leave T::default() behind -----
    let taken = c.take();                         // taken=7, c=0
    assert_eq!(taken, 7);
    assert_eq!(c.get(), 0);

    // ----- as_ptr: raw pointer to inner value (unsafe mutation) -----
    let c = Cell::new(10i32);
    let ptr: *mut i32 = c.as_ptr();
    unsafe { *ptr += 1; }
    assert_eq!(c.get(), 11);

    // ----- from_mut: convert &mut T → &Cell<T> (zero-cost) -----
    let mut x = 42i32;
    let cell: &Cell<i32> = Cell::from_mut(&mut x);
    cell.set(99);
    assert_eq!(x, 99);

    // ----- as_slice_of_cells: &Cell<[T]> → &[Cell<T>] -----
    let arr = [Cell::new(1), Cell::new(2), Cell::new(3)];
    arr[1].set(20);
    assert_eq!(arr[1].get(), 20);

    // ----- Typical use case: shared reference that needs mutation -----
    struct Logger {
        count: Cell<u32>,
    }
    impl Logger {
        fn log(&self, msg: &str) {
            self.count.set(self.count.get() + 1); // mutate through &self
            println!("[{}] {msg}", self.count.get());
        }
    }
    let logger = Logger { count: Cell::new(0) };
    logger.log("first");                          // [1] first
    logger.log("second");                         // [2] second

    // ----- Cell with non-Copy types (use take/replace instead of get) -----
    let s: Cell<String> = Cell::new("hello".to_string());
    let val: String = s.take();                  // moves out, s now holds ""
    println!("{val}");
    s.set("world".to_string());

    println!("cell_methods done ✓");
}

// =============================================================================
// RefCell<T>
// =============================================================================
// Interior mutability for any T. Borrow rules enforced at RUNTIME.
// Use when you need &mut through &self and T is not Copy.
// PANICS if borrow rules are violated (use try_borrow for non-panicking version).
// =============================================================================
fn refcell_methods() {
    println!("\n========== RefCell<T> ==========");

    let rc: RefCell<Vec<i32>> = RefCell::new(vec![1, 2, 3]);

    // ----- Immutable borrow → Ref<T> (like &T) -----
    {
        let guard = rc.borrow();                  // runtime check: not mutably borrowed
        println!("{:?}", *guard);
        // guard (Ref) drops here — borrow released
    }

    // ----- Mutable borrow → RefMut<T> (like &mut T) -----
    {
        let mut guard = rc.borrow_mut();          // runtime check: no other borrows
        guard.push(4);
        // guard drops here
    }
    println!("{:?}", rc.borrow());                // [1, 2, 3, 4]

    // ----- Non-panicking variants -----
    let _guard = rc.borrow();                     // hold a shared borrow
    match rc.try_borrow_mut() {
        Ok(_)  => println!("got &mut"),
        Err(e) => println!("borrow conflict: {e}"), // prints this
    }
    drop(_guard);

    match rc.try_borrow() {
        Ok(g)  => println!("{:?}", *g),           // [1, 2, 3, 4]
        Err(e) => println!("conflict: {e}"),
    }

    // ----- into_inner: consume RefCell, extract T -----
    let rc2 = RefCell::new(99i32);
    let val: i32 = rc2.into_inner();             // 99
    println!("{val}");

    // ----- replace: swap inner value, return old -----
    let r = RefCell::new(1i32);
    let old = r.replace(2);                      // old=1, r now holds 2
    assert_eq!(old, 1);
    assert_eq!(*r.borrow(), 2);

    // replace_with: replace using a closure that receives &mut T
    r.replace_with(|&mut v| v * 10);             // r now holds 20

    // ----- swap: exchange contents of two RefCells -----
    let r1 = RefCell::new(10i32);
    let r2 = RefCell::new(20i32);
    r1.swap(&r2);
    assert_eq!(*r1.borrow(), 20);
    assert_eq!(*r2.borrow(), 10);

    // ----- as_ptr: raw *mut T pointer (bypasses borrow check — unsafe) -----
    let r = RefCell::new(42i32);
    let ptr: *mut i32 = r.as_ptr();
    unsafe { *ptr += 1; }
    assert_eq!(*r.borrow(), 43);

    // ----- take: set inner value to Default, return old (T: Default) -----
    let r = RefCell::new(vec![1, 2, 3]);
    let v: Vec<i32> = r.take();                  // v=[1,2,3], r=[]
    println!("{v:?}");

    // ----- Ref / RefMut map: project into a field -----
    let rc = RefCell::new((1i32, "hello"));
    let mapped = RefCell::borrow(&rc);
    let num: std::cell::Ref<i32> = std::cell::Ref::map(mapped, |t| &t.0);
    println!("{}", *num);                        // 1

    // ----- Classic pattern: Rc<RefCell<T>> -----
    let shared = Rc::new(RefCell::new(vec![1]));
    let clone  = Rc::clone(&shared);
    clone.borrow_mut().push(2);              // mutate through shared reference
    println!("{:?}", shared.borrow());            // [1, 2]

    println!("refcell_methods done ✓");
}

// =============================================================================
// Cow<'a, T>
// =============================================================================
// Clone-On-Write: either borrows data (&T) or owns it (T::Owned).
// Zero-cost when reading, only clones when mutation is needed.
// Use when you want to accept both &str and String (or &[T] and Vec<T>)
// in the same type, or defer cloning until it's actually required.
// =============================================================================
fn cow_methods() {
    println!("\n========== Cow<'a, T> ==========");

    // ----- Construction -----

    // Borrowed variant — no allocation
    let borrowed: Cow<str> = Cow::Borrowed("hello world");

    // Owned variant — heap allocated
    let owned: Cow<str> = Cow::Owned(String::from("hello world"));

    // From impls — most ergonomic way to construct
    let c: Cow<str>    = "hello".into();                     // Borrowed
    let c: Cow<str>    = String::from("hello").into();     // Owned
    let c: Cow<[i32]>  = vec![1, 2, 3].into();               // Owned
    let c: Cow<[i32]>  = Cow::Borrowed(&[1, 2, 3]);          // Borrowed

    // ----- Querying variant -----
    let b: Cow<str> = Cow::Borrowed("hi");
    let o: Cow<str> = Cow::Owned("hi".to_string());

    println!("is borrowed: {}", matches!(b, Cow::Borrowed(_))); // true
    println!("is owned:    {}", matches!(o, Cow::Owned(_)));    // true

    // ----- Accessing the value (read) -----
    // Deref always gives &B regardless of variant — transparent read access
    let c: Cow<str> = Cow::Borrowed("hello");
    println!("len: {}", c.len());                // calls &str method directly
    println!("upper: {}", c.to_uppercase());     // still no clone

    // as_ref: &B
    let r: &str = c.as_ref();

    // ----- to_mut: get &mut T::Owned, cloning if currently Borrowed -----
    let mut c: Cow<str> = Cow::Borrowed("hello");
    println!("before mutation: borrowed={}", matches!(c, Cow::Borrowed(_))); // true
    let s: &mut String = c.to_mut();             // clones "hello" into a String
    s.push_str(" world");
    println!("after mutation:  owned=  {}", matches!(c, Cow::Owned(_)));     // true
    println!("{c}");                             // "hello world"

    // If already Owned, to_mut is free (no clone):
    let mut c: Cow<str> = Cow::Owned(String::from("hello"));
    c.to_mut().push_str(" world");              // no clone — already owned
    println!("{c}");

    // ----- into_owned: consume Cow, always return T::Owned -----
    let c: Cow<str> = Cow::Borrowed("hello");
    let owned: String = c.into_owned();          // clones because it was Borrowed

    let c: Cow<str> = Cow::Owned(String::from("hello"));
    let owned: String = c.into_owned();          // moves — no clone needed

    // ----- is_borrowed / is_owned (nightly, shown as pattern match) -----
    let c: Cow<str> = "hi".into();
    let borrowed = matches!(c, Cow::Borrowed(_));

    // ----- Key use case 1: accept &str or String uniformly -----
    fn process(input: Cow<str>) -> Cow<str> {
        if input.contains("bad") {
            // Only clone if we actually need to modify
            Cow::Owned(input.replace("bad", "good"))
        } else {
            input  // pass through untouched — no allocation
        }
    }
    let a = process("clean input".into());       // Cow::Borrowed — no alloc
    let b = process("bad input".into());         // Cow::Owned — clone on write
    println!("{a}");
    println!("{b}");

    // ----- Key use case 2: avoid unnecessary allocation in serialization -----
    fn escape(s: &str) -> Cow<str> {
        if s.contains('&') {
            Cow::Owned(s.replace('&', "&amp;"))  // clone only when needed
        } else {
            Cow::Borrowed(s)                     // borrow the original
        }
    }
    println!("{}", escape("hello"));             // Borrowed — no alloc
    println!("{}", escape("hello & world"));     // Owned — cloned + replaced

    // ----- Key use case 3: Cow<[T]> for slices -----
    fn ensure_sorted(v: Cow<[i32]>) -> Cow<[i32]> {
        if v.windows(2).all(|w| w[0] <= w[1]) {
            v                                    // already sorted — borrow
        } else {
            let mut owned = v.into_owned();
            owned.sort();
            Cow::Owned(owned)                    // clone + sort only when needed
        }
    }
    let sorted   = ensure_sorted(Cow::Borrowed(&[1, 2, 3]));  // Borrowed
    let unsorted = ensure_sorted(Cow::Borrowed(&[3, 1, 2]));  // Owned (sorted)
    println!("{sorted:?}");
    println!("{unsorted:?}");

    println!("cow_methods done ✓");
}

// =============================================================================
// QUICK REFERENCE CARD
// =============================================================================
//
//  TYPE              OWNERSHIP    THREADS   MUTATION        BORROW CHECK
//  ─────────────────────────────────────────────────────────────────────────
//  Box<T>            single       ✅(T:Send) via &mut T     compile-time
//  Rc<T>             shared       ❌         via RefCell     —
//  Arc<T>            shared       ✅         via Mutex       —
//  Cell<T>           single*      ❌         get/set         none (Copy only)
//  RefCell<T>        single*      ❌         borrow_mut()    RUNTIME (panics)
//  Rc<RefCell<T>>    shared       ❌         borrow_mut()    RUNTIME (panics)
//  Arc<Mutex<T>>     shared       ✅         lock()          RUNTIME (blocks)
//  Cow<'a,T>         borrow/own   ✅(T:Send) to_mut()        compile-time
//  Weak<T>           non-owning   ✅/❌      via upgrade()   —
//
//  * "single" means the value is not shared across threads.
//
//  DECISION TREE
//  ─────────────
//  Need heap allocation?
//    → single owner                           → Box<T>
//    → shared ownership, single thread        → Rc<T>
//    → shared ownership, multi thread         → Arc<T>
//
//  Need interior mutability (mutate through &self)?
//    → T is Copy, single thread               → Cell<T>
//    → T is any,  single thread               → RefCell<T>
//    → T is any,  multi thread                → Mutex<T> / RwLock<T>
//
//  Need to break a reference cycle?
//    → Rc cycle  → Rc::downgrade()  → Rc::Weak<T>
//    → Arc cycle → Arc::downgrade() → Arc::Weak<T>
//
//  Need zero-cost borrowed-or-owned duality?
//    → Cow<'a, T>
//
// =============================================================================