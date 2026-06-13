// ============================================================================
// Deref Trait in Rust — Interview Preparation
// ============================================================================
//
// Run:  cargo script deref_usecases.rs   (with cargo-script)
//   or: copy into a cargo project and run with `cargo run`
//
// Table of contents:
//   1.  What Deref IS — the trait definition and the * operator
//   2.  Implementing Deref on a custom smart pointer
//   3.  DerefMut — the mutable counterpart
//   4.  Deref coercion — automatic &T → &U conversion
//   5.  Coercion chains — multiple hops in one call
//   6.  Box<T> and Deref
//   7.  Rc<T> and Deref
//   8.  String → &str via Deref
//   9.  Vec<T> → &[T] via Deref
//  10.  Deref in function argument coercion
//  11.  Deref vs AsRef — when to use which
//  12.  Deref with trait objects
//  13.  The Newtype pattern + Deref
//  14.  Building a Stack<T> with Deref to access the slice API
//  15.  Common interview gotchas
// ============================================================================

use std::ops::{Deref, DerefMut};
use std::rc::Rc;

fn main() {
    section_1_deref_and_star_operator();
    section_2_custom_smart_pointer();
    section_3_deref_mut();
    section_4_deref_coercion();
    section_5_coercion_chains();
    section_6_box_deref();
    section_7_rc_deref();
    section_8_string_deref();
    section_9_vec_deref();
    section_10_function_argument_coercion();
    section_11_deref_vs_as_ref();
    section_12_deref_with_trait_objects();
    section_13_newtype_pattern();
    section_14_stack_with_deref();
    section_15_gotchas();
}

// ============================================================================
// 1. What Deref IS — the trait definition and the * operator
// ============================================================================
//
// pub trait Deref {
//     type Target: ?Sized;
//     fn deref(&self) -> &Self::Target;
// }
//
// When you write *x, the compiler rewrites it as:
//   *(x.deref())         for shared references
//   *(x.deref_mut())     for mutable references
//
// The * is NOT magic — it is syntactic sugar for calling .deref() and then
// dereferencing the resulting &T to reach T.

fn section_1_deref_and_star_operator() {
    println!("\n=== 1. Deref and the * operator ===");

    let x = Box::new(42_i32);

    // These three lines are IDENTICAL — the compiler rewrites them all:
    println!("{}", *x);                      // sugar
    println!("{}", *Box::deref(&x));         // what the compiler actually does
    println!("{}", *x.deref());              // method call syntax

    // With a plain reference, * just follows the pointer:
    let n = 7_i32;
    let r = &n;
    println!("*r = {}", *r);                 // 7 — basic dereference

    // KEY POINT: Deref is only about READING. Writing through * uses DerefMut.
}

// ============================================================================
// 2. Implementing Deref on a custom smart pointer
// ============================================================================

// A minimal Box-like wrapper that owns its value on the heap.
struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        MyBox(value)
    }
}

// Deref lets callers use * to reach the inner T.
impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0   // return a shared reference to the inner value
    }
}

fn section_2_custom_smart_pointer() {
    println!("\n=== 2. Custom smart pointer with Deref ===");

    let b = MyBox::new(100_i32);

    // Without Deref we could not use * at all:
    println!("*b = {}", *b);                   // calls MyBox::deref(&b) → &100, then *

    // Deref coercion also fires: &MyBox<i32> → &i32
    let r: &i32 = &b;
    println!("r = {}", r);

    // Calling methods on the inner type without explicit dereference:
    let s = MyBox::new(String::from("hello"));
    // &MyBox<String> → &String (via MyBox::deref)
    //               → &str    (via String::deref)  — TWO hops, automatic!
    println!("len = {}", s.len());             // String method, called via coercion
    println!("upper = {}", s.to_uppercase());  // also works
}

// ============================================================================
// 3. DerefMut — the mutable counterpart
// ============================================================================
//
// pub trait DerefMut: Deref {
//     fn deref_mut(&mut self) -> &mut Self::Target;
// }
//
// Required when you want to assign through *, e.g. *x = 5;

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

fn section_3_deref_mut() {
    println!("\n=== 3. DerefMut ===");

    let mut b = MyBox::new(0_i32);
    println!("before: {}", *b);    // 0

    *b = 99;                        // calls DerefMut::deref_mut(&mut b) → &mut 0, then assigns
    println!("after:  {}", *b);    // 99

    // Mutable coercion: &mut MyBox<T> → &mut T
    let mut s = MyBox::new(String::from("hello"));
    s.push_str(" world");           // push_str takes &mut str — two mutable hops
    println!("s = {}", *s);

    // RULE: mutable deref coercion requires &mut self AND DerefMut implemented.
    //       If only Deref is implemented, *x = ... will fail to compile.
}

// ============================================================================
// 4. Deref coercion — automatic &T → &U when T: Deref<Target=U>
// ============================================================================
//
// Coercion fires silently in three situations:
//   A. Passing an argument to a function
//   B. Assigning to a reference variable with a declared type
//   C. After a dot (.) — method resolution
//
// The compiler inserts as many .deref() calls as needed to make types match.

fn print_str(s: &str) {                // expects &str
    println!("{}", s);
}

fn section_4_deref_coercion() {
    println!("\n=== 4. Deref coercion ===");

    // A — function argument: &String → &str (one hop via String::deref)
    let owned = String::from("coercion");
    print_str(&owned);                 // &String coerces to &str

    // B — assignment to an explicitly typed reference
    let s: &str = &owned;              // coercion fires here too
    println!("{}", s);

    // C — method call: String methods are found via auto-deref
    let b = Box::new(String::from("boxed"));
    println!("{}", b.to_uppercase());  // Box → String → str → method found

    // Coercion also handles mutable → shared automatically:
    let mut m = String::from("mut");
    let immut: &str = &m;              // &mut String → &String → &str
    println!("{}", immut);

    // INTERVIEW NOTE: coercions NEVER allocate — they are pure reference
    // adjustments resolved entirely at compile time.
}

// ============================================================================
// 5. Coercion chains — multiple hops in one call
// ============================================================================

fn takes_slice(data: &[i32]) {
    println!("slice len = {}", data.len());
}

fn section_5_coercion_chains() {
    println!("\n=== 5. Coercion chains ===");

    let v = vec![1, 2, 3];

    // Chain: &Box<Vec<i32>> → &Vec<i32> (Box::deref)
    //                       → &[i32]    (Vec::deref)
    let boxed_vec: Box<Vec<i32>> = Box::new(v);
    takes_slice(&boxed_vec);           // TWO coercions, zero cost

    // Chain: &Box<String> → &String (Box::deref) → &str (String::deref)
    let boxed_string: Box<String> = Box::new(String::from("chained"));
    print_str(&boxed_string);          // TWO coercions

    // The compiler tries coercions in order until types match or gives up.
    // If no chain can be found, you get a type error and must coerce manually.
}

// ============================================================================
// 6. Box<T> and Deref
// ============================================================================

fn section_6_box_deref() {
    println!("\n=== 6. Box<T> and Deref ===");

    let b: Box<i32> = Box::new(5);

    // Box<i32> implements Deref<Target = i32>
    let r: &i32 = &b;                 // &Box<i32> → &i32
    println!("via ref:   {}", r);

    // Methods on i32 available directly:
    let b2 = Box::new(3.14_f64);
    println!("floor:     {}", b2.floor());   // f64::floor via coercion

    // Box also implements DerefMut:
    let mut b3 = Box::new(0_i32);
    *b3 += 10;
    println!("mutated:   {}", b3);

    // Sized note: Box<dyn Trait> is special — the DST lives on the heap and
    // Box dereferences to the concrete type behind the trait object.
    let b4: Box<dyn std::fmt::Display> = Box::new(42_i32);
    println!("trait obj: {}", b4);    // deref to i32, then Display
}

// ============================================================================
// 7. Rc<T> and Deref
// ============================================================================

fn section_7_rc_deref() {
    println!("\n=== 7. Rc<T> and Deref ===");

    let rc: Rc<String> = Rc::new(String::from("shared"));

    // Rc<String> implements Deref<Target = String>
    println!("len:   {}", rc.len());            // String method via coercion
    println!("upper: {}", rc.to_uppercase());   // goes Rc → String → str → method

    // Clone increases the reference count, but both Rc values deref to the
    // same underlying data — read-only because Rc gives only &T.
    let rc2 = Rc::clone(&rc);
    println!("count: {}", Rc::strong_count(&rc));  // 2

    // INTERVIEW NOTE: Rc does NOT implement DerefMut because multiple Rc
    // pointers to the same data would break Rust's aliasing rules.
    // Use Rc<RefCell<T>> for shared mutability.

    let shared_str: &str = &rc2;    // Rc<String> → String → &str
    println!("coerced: {}", shared_str);
}

// ============================================================================
// 8. String → &str via Deref
// ============================================================================
//
// impl Deref for String {
//     type Target = str;
//     fn deref(&self) -> &str { ... }
// }

fn section_8_string_deref() {
    println!("\n=== 8. String → &str via Deref ===");

    let owned: String = String::from("hello world");

    // &String → &str implicitly
    let s: &str = &owned;
    println!("s = {}", s);

    // str methods are accessible directly on String because of deref coercion:
    println!("contains: {}", owned.contains("world"));   // str::contains
    println!("split:");
    for word in owned.split_whitespace() {               // str::split_whitespace
        println!("  '{}'", word);
    }

    // INTERVIEW NOTE: this is why functions should prefer &str over &String —
    // accepting &str works for both &String (via coercion) AND string literals,
    // whereas &String only accepts String references.
    fn greet(name: &str) { println!("Hello, {}!", name); }
    greet(&owned);                   // &String → &str
    greet("literal");                // &str directly — no String allocation
}

// ============================================================================
// 9. Vec<T> → &[T] via Deref
// ============================================================================
//
// impl<T> Deref for Vec<T> {
//     type Target = [T];
//     fn deref(&self) -> &[T] { ... }
// }

fn section_9_vec_deref() {
    println!("\n=== 9. Vec<T> → &[T] via Deref ===");

    let v: Vec<i32> = vec![10, 20, 30, 40, 50];

    // &Vec<i32> → &[i32] implicitly
    let s: &[i32] = &v;
    println!("slice: {:?}", s);

    // [T] methods accessible directly on Vec:
    println!("first:    {:?}", v.first());         // slice::first
    println!("last:     {:?}", v.last());          // slice::last
    println!("contains: {}",  v.contains(&30));    // slice::contains
    println!("sorted:   {}",  v.windows(2).all(|w| w[0] <= w[1]));

    // Pattern: prefer &[T] over &Vec<T> in function signatures —
    // accepts both Vec (via coercion) and array slices, more flexible.
    fn sum(data: &[i32]) -> i32 { data.iter().sum() }
    println!("sum from vec:   {}", sum(&v));
    println!("sum from array: {}", sum(&[1, 2, 3]));  // no coercion needed for arrays
}

// ============================================================================
// 10. Deref in function argument coercion — a detailed look
// ============================================================================

fn expects_str(s: &str)     { println!("  &str: '{}'", s); }
fn expects_slice(s: &[i32]) { println!("  &[i32] len={}", s.len()); }

fn section_10_function_argument_coercion() {
    println!("\n=== 10. Function argument coercion ===");

    // All of these coerce to &str:
    let string   = String::from("owned");
    let box_str  = Box::new(String::from("boxed"));
    let rc_str   = Rc::new(String::from("rc'd"));
    let my_box   = MyBox::new(String::from("mybox"));

    println!("Coercions to &str:");
    expects_str(&string);            // &String   → &str   (1 hop)
    expects_str(&box_str);           // &Box<String> → &String → &str (2 hops)
    expects_str(&rc_str);            // &Rc<String>  → &String → &str (2 hops)
    expects_str(&my_box);            // &MyBox<String> → &String → &str (2 hops)

    // All of these coerce to &[i32]:
    let vec     = vec![1, 2, 3];
    let box_vec = Box::new(vec![4, 5, 6]);

    println!("Coercions to &[i32]:");
    expects_slice(&vec);             // &Vec<i32> → &[i32]         (1 hop)
    expects_slice(&box_vec);         // &Box<Vec<i32>> → &Vec<i32> → &[i32] (2 hops)
    expects_slice(&[7, 8, 9]);       // &[i32; 3] → &[i32]  (array unsizing, not deref)

    // INTERVIEW NOTE: the compiler tries coercions until it finds a match or
    // exhausts the chain. It never back-tracks or tries alternative paths.
}

// ============================================================================
// 11. Deref vs AsRef — when to use which
// ============================================================================
//
// Deref: implicit, automatic, describes OWNERSHIP semantics of a smart pointer.
//        Only one Deref impl per type (one "canonical" target).
//
// AsRef: explicit (.as_ref()), describes a CONVERSION for function generics.
//        A type can implement AsRef<T> for many different T.
//        Used in function bounds when you want to accept many types of reference.

fn section_11_deref_vs_as_ref() {
    println!("\n=== 11. Deref vs AsRef ===");

    // AsRef in a generic function — accepts String, &str, PathBuf, &Path etc.
    fn print_as_str<S: AsRef<str>>(s: S) {
        println!("  as_ref: '{}'", s.as_ref());
    }

    print_as_str("literal");
    print_as_str(String::from("owned"));

    // Deref in a generic function — accepts Box<String>, Rc<String>, etc.
    fn print_deref<S: Deref<Target = str>>(s: S) {
        println!("  deref:  '{}'", &*s);
    }

    print_deref(String::from("owned"));
    print_deref(*Box::new(String::from("boxed")));
    //          ^ deref Box<String> -> String first
    //            now S = String, String: Deref<Target = str> ✅

    // KEY DISTINCTION:
    //   std::fs::File::open accepts AsRef<Path> → you can pass &str, String,
    //   PathBuf, &Path — all explicitly via .as_ref().
    //
    //   Box<String> auto-coerces to &str at a &str call site — implicit, via Deref.
    //
    // Rule of thumb:
    //   - Library fn parameters → AsRef<T> for maximum flexibility
    //   - Smart pointer behaviour → Deref for transparent access
}

// ============================================================================
// 12. Deref with trait objects
// ============================================================================

trait Greet {
    fn hello(&self) -> String;
}

struct English;
struct Spanish;

impl Greet for English { fn hello(&self) -> String { "Hello!".to_string() } }
impl Greet for Spanish { fn hello(&self) -> String { "¡Hola!".to_string() } }

fn section_12_deref_with_trait_objects() {
    println!("\n=== 12. Deref with trait objects ===");

    // Box<dyn Trait> implements Deref<Target = dyn Trait>, so you can call
    // trait methods directly without explicit dereference.
    let greeters: Vec<Box<dyn Greet>> = vec![
        Box::new(English),
        Box::new(Spanish),
    ];

    for g in &greeters {
        // g is &Box<dyn Greet>
        // → Deref: &Box<dyn Greet> → &dyn Greet  (one hop)
        // → virtual dispatch to concrete hello()
        println!("  {}", g.hello());
    }

    // Rc<dyn Trait> works the same way:
    let rc: Rc<dyn Greet> = Rc::new(English);
    println!("  rc: {}", rc.hello());   // Rc<dyn Greet> → dyn Greet → hello()
}

// ============================================================================
// 13. The Newtype pattern + Deref
// ============================================================================
//
// A newtype wraps an existing type to add behaviour or enforce invariants.
// Implementing Deref gives transparent access to the inner type's methods
// without manually delegating every call.

struct Meters(f64);
struct Kilograms(f64);

impl Deref for Meters {
    type Target = f64;
    fn deref(&self) -> &f64 { &self.0 }
}

impl DerefMut for Meters {
    fn deref_mut(&mut self) -> &mut f64 { &mut self.0 }
}

// A validated email address — invariant: contains '@'
struct Email(String);

impl Email {
    fn new(s: &str) -> Result<Self, &'static str> {
        if s.contains('@') {
            Ok(Email(s.to_string()))
        } else {
            Err("not a valid email")
        }
    }
}

impl Deref for Email {
    type Target = str;          // expose as &str, not &String
    fn deref(&self) -> &str { &self.0 }
}

fn section_13_newtype_pattern() {
    println!("\n=== 13. Newtype pattern + Deref ===");

    // Meters wraps f64 — Deref gives us all f64 methods for free:
    let mut dist = Meters(42.195);
    println!("km:    {:.3}", *dist / 1000.0);  // f64::div via deref
    println!("floor: {}", dist.floor());        // f64::floor via coercion
    *dist += 0.005;                             // f64 mutation via DerefMut
    println!("new:   {}", *dist);

    // Type safety: Meters and Kilograms don't accidentally mix:
    let _m = Meters(100.0);
    let _k = Kilograms(70.0);
    // let wrong = _m + _k;    // ← compile error — Deref doesn't bridge the types

    // Email exposes &str methods but enforces the '@' invariant at construction:
    let email = Email::new("user@example.com").unwrap();
    println!("domain: {}", email.split('@').nth(1).unwrap()); // str::split via deref
    println!("upper:  {}", email.to_uppercase());             // str::to_uppercase

    // INTERVIEW NOTE: Deref on a newtype is a double-edged sword.
    // It provides convenience but can accidentally expose the inner type's
    // full interface. For strong encapsulation, don't impl Deref.
}

// ============================================================================
// 14. Building a Stack<T> with Deref to access the slice API
// ============================================================================
//
// A practical example: we build a Stack that keeps all its data in a Vec,
// then implement Deref<Target=[T]> so callers get the entire slice API
// (contains, iter, len, first, last, windows...) for free.

struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { data: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
}

// Expose the underlying slice — gives us contains, iter, first, last, etc.
impl<T> Deref for Stack<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data
    }
}

impl<T> DerefMut for Stack<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

fn section_14_stack_with_deref() {
    println!("\n=== 14. Stack<T> with Deref<Target=[T]> ===");

    let mut stack: Stack<i32> = Stack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);

    // Stack-specific API:
    println!("pop:      {:?}", stack.pop());       // 3

    // Slice API for free via Deref — no delegation code written:
    println!("len:      {}", stack.len());          // slice::len
    println!("first:    {:?}", stack.first());      // slice::first
    println!("contains: {}", stack.contains(&1));   // slice::contains
    println!("iter sum: {}", stack.iter().sum::<i32>());  // Iterator on &[i32]

    // DerefMut lets us sort the underlying data in place:
    stack.sort();                                   // slice::sort via DerefMut
    println!("sorted:   {:?}", &*stack);

    // Passing Stack to a function expecting &[i32]:
    fn print_slice(s: &[i32]) {
        println!("slice:    {:?}", s);
    }
    print_slice(&stack);                            // Stack<i32> → &[i32]
}

// ============================================================================
// 15. Common interview gotchas
// ============================================================================

fn section_15_gotchas() {
    println!("\n=== 15. Common interview gotchas ===");

    // ── Gotcha 1: * moves out of the smart pointer if T is not Copy ──────────
    //
    // let b = Box::new(String::from("hello"));
    // let s = *b;   // ← COMPILE ERROR if T: !Copy — would move String out of Box,
    //               //   leaving Box with a dangling pointer.
    //               //   Use (*b).clone() or consume the box with Box::into_inner (nightly)
    //               //   or just let b_inner = *b; which moves b entirely.
    //
    // For Copy types it works fine:
    let b_copy = Box::new(42_i32);
    let n = *b_copy;     // i32: Copy → bitwise copy, Box still valid
    println!("Copy out of Box: {}", n);

    let b_string = Box::new(String::from("hello"));
    let s = *b_string;   // String: !Copy → MOVES out, b_string is consumed
    println!("Moved out of Box: {}", s);
    // println!("{}", b_string);  // ← compile error: value moved

    // ── Gotcha 2: Deref coercion does NOT happen in generic bounds ────────────
    //
    // fn wants_t<T>(x: T) where T: SomeTrait { ... }
    // Passing Box<String> when T = String does NOT work — Box<String> ≠ String.
    // Coercion only fires at REFERENCE sites (&Box<String> → &String).
    //
    // This compiles:
    fn take_ref_str(s: &str) { let _ = s; }
    let owned = String::from("hello");
    take_ref_str(&owned);              // &String → &str, coercion fires ✓
    //
    // This would NOT compile (T must be exactly &str, not &String):
    // fn take_str_generic<T: AsRef<str>>(s: T) { }
    // take_str_generic::<&str>(&owned); // type mismatch

    // ── Gotcha 3: infinite Deref loop is impossible — the chain is finite ────
    //
    // Each .deref() must return a reference to a DIFFERENT (smaller) type.
    // The compiler verifies this statically. You cannot write:
    //   impl Deref for Foo { type Target = Foo; ... }  // compile error

    // ── Gotcha 4: method resolution order with Deref ─────────────────────────
    //
    // Methods on the outer type take priority over methods on the inner type.
    // If Box<T> defined a method called `clone()`, it would shadow T::clone().
    // This can cause surprising method resolution in newtypes.
    struct Wrapper(String);
    impl Wrapper {
        fn len(&self) -> usize { 999 }   // shadows String::len via Deref
    }
    impl Deref for Wrapper {
        type Target = String;
        fn deref(&self) -> &String { &self.0 }
    }
    let w = Wrapper(String::from("hi"));
    println!("Wrapper::len = {}", w.len());          // 999 — OUR method wins
    println!("String::len  = {}", w.0.len());        // 2   — explicit access

    // ── Gotcha 5: &**x and &&T ───────────────────────────────────────────────
    //
    // Multiple & layers do NOT chain through Deref automatically.
    // &&String does NOT coerce to &str — only one layer of auto-deref at a time.
    let s = String::from("hello");
    let r: &&String = &&s;
    // print_str(r);           // ← would NOT compile — &&String ≠ &str
    print_str(*r);             // explicitly deref one layer: &&String → &String → &str ✓

    // ── Gotcha 6: Deref coercion is one-way ──────────────────────────────────
    //
    // &str does NOT coerce to &String — that would require allocation.
    // Coercion only goes from "richer" (owned, smart pointer) to "simpler" (&T).
    let s: &str = "literal";
    // let r: &String = s;   // ← compile error — no coercion from &str to &String

    println!("\nAll gotchas demonstrated safely.");
}