// subtyping in rust is only about lifetimes.
// In many OOP languages, subtyping is about classes: Cat is a subtype of Animal because you can
// pass a Cat anywhere an Animal is expected. Rust does not have this kind of structural inheritance.
//
// Instead, Rust’s subtyping relationship exists solely for lifetimes.
//
// Rule: a type A is a subtype of B (written A <: B) if a value of type A can be safely used
// wherever a value of type B is expected.
//
// And in Rust, this happens when one lifetime outlives another.
//
// 'long outlives 'short → 'long is a subtype of 'short (i.e., 'long <: 'short).
//
// Why? Because a reference that lives for 'long can safely be used in a place that only requires
// the reference to live for the shorter 'short. The longer lifetime subsumes the shorter one.
//
// So subtyping in Rust boils down to: a bigger lifetime is a subtype of a smaller lifetime.

// Where subtyping happens in Rust: lifetime coercions.
// The compiler uses subtyping to decide what implicit lifetime coercions are allowed.
// The classic coercion you already know:
fn lifetime_coercion() {

    fn shortest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() < y.len() { x } else { y }
    }

    let s1 = String::from("hello");   // 'long
    let s2 = String::from("world");   // 'long
    let result;
    {
        // r1 and r2 have distinct, shorter lifetimes, but shortest expects both arguments to
        // have exactly the same lifetime 'a. The compiler coerces the longer outer lifetimes
        // ('long) to the shorter inner ones ('short1/'short2), then unifies them to a common
        // lifetime 'a. This is lifetime subtyping in action: 'long is used where 'short is expected.
        // Let's say these inner scopes give us shorter lifetimes...
        let r1 = &s1;   // 'short1
        let r2 = &s2;   // 'short2
        result = shortest(r1, r2);
    }
}

// Variance: how subtyping flows through generic types.
// Now, the truly tricky part: variance defines how the subtyping relationship between inner
// lifetimes propagates through a generic type like &'a T, &'a mut T, or fn(T) -> U.
// There are three kinds of variance:
//
// Variance	            Meaning
//                                      Example
// Covariant	        If 'a <: 'b, then Type<'a> <: Type<'b>	                &'a T

// Contravariant	    If 'a <: 'b, then Type<'b> <: Type<'a> (reversed!)	    fn(T) -> &'a i32 in
//                                                                              the argument T is… wait,
//                                                                              better example: fn(&'a T)
//                                                                              is contravariant in 'a
// Invariant	        No subtyping relation at all,
//                      even if 'a <: 'b. Type<'a> and Type<'b>
//                      are completely different types	                        &'a mut T, UnsafeCell<T>

// Covariant: &'a T
// If 'long >: 'short, then &'long T can be used where &'short T is expected. That’s the common
// case you can always pass a longer borrow where a shorter one is needed.
fn covariant() {
    fn read(val: &i32) {} // expects some lifetime, call it 'short

    static X: i32 = 42;
    let r: &'static i32 = &X; // 'static
    read(r); // 'static >: 'short, so &'static i32 >: &'short i32, works
}

// Contravariant: function types (in argument position)
// Function pointers / closures are contravariant in their parameter lifetimes.
// That means the subtyping direction flips.
fn contravariant() {
    // Contravariance for function arguments means:
    //
    // If 'long >: 'short, then function type fn(&'long T) <: fn(&'short T), function type with 
    // lifetime 'long is a subtype of function type with lifetime 'short.
    //
    // Because: a function that can accept a shorter reference is more flexible – you can
    // pass it a longer reference (which is a subtype of the shorter one). So the function
    // type with the shorter lifetime in its parameter is “more general”. Subtyping flips:
    // you can use fn(&'short T) where fn(&'long T) is expected.


    fn takes_fn(f: fn(&'static i32)) {
        // f expects a reference that lives 'static.
    }

    // g takes a local tmp reference with a short lifetime.
    let g: fn(&i32) = |_| {};
    takes_fn(g); // ERROR? No, wait. This is interesting.

    fn apply(f: fn(&'static str)) {
        f("hello");
    }

    let print: fn(&str) = |s| println!("{s}"); // the lifetime elided, but it's a short anonymous lifetime
    apply(print); // OK: fn(&str) <: fn(&'static str) because &str is contravariant in the parameter.
}


// Invariant: &'a mut T
// Mutable references are invariant in their lifetime. There is no subtyping between
// &'a mut T and &'b mut T, regardless of how 'a and 'b relate. This is the one that
// most often bites people.
fn invariant() {
    fn assign(src: &mut i32, dst: &mut i32) {
        *dst = *src;
    }

    let mut a = 1;
    let mut b = 2;
    let r1: &'static mut i32 = &mut a; // make it 'static for clarity (though not really possible)
    // Actually, you can't get a real 'static mutable reference to a local, but let's pretend.


    fn modify<'a>(r: &'a mut i32) -> &'a i32 {
        &*r
    }

    let mut x = 5;
    let y;
    {
        let mut z = 10;
        y = modify(&mut x); // works
        // y = modify(&mut z); // would constrain 'a to the inner block, making y invalid later
    }
    println!("{y}");

    // Invariance ensures that if you try to use a mutable reference with a shorter lifetime
    // in a place that expects a longer one, the compiler won’t let you. It prevents aliasing
    // soundness holes. The classic example is std::mem::swap:
    // If mutable references were covariant, you could pass &'long mut T and &'short mut T,
    // and the compiler could coerce the longer one to the shorter one. That would then allow
    // modifications through the shorter reference after the longer one supposedly ended,
    // breaking safety. Invariance closes this hole.
    fn swap<T>(x: &mut T, y: &mut T) {/*...*/ }
}

// Demonstrates Rust's lifetime-based subtyping and variance.
// Only immutable references and trait objects exhibit subtyping;
// mutable references, Cell, UnsafeCell are invariant.

fn main() {
    println!("=== 1. Covariance of immutable references ===");
    covariance_examples();

    println!("\n=== 2. Covariance of Box, Vec, and own types ===");
    covariance_in_containers();

    println!("\n=== 3. Contravariance in function arguments ===");
    contravariance_fn_example();

    println!("\n=== 4. Invariance of &mut T ===");
    invariance_mutable_reference();

    println!("\n=== 5. Invariance of Cell<T> and UnsafeCell<T> ===");
    invariance_cell();

    println!("\n=== 6. Trait object subtyping ===");
    trait_object_example();

    println!("\nAll runnable examples completed successfully.");
}

// ------------------------------------------------
// 1. Covariance: longer lifetime → shorter lifetime
// ------------------------------------------------
fn covariance_examples() {
    // 'static is a subtype of any 'a (i.e., 'static: 'a)
    let s: &'static str = "I live forever";

    // A function that expects a reference with ANY lifetime 'a
    fn take_str(_s: &str) {
        println!("  - take_str got: a string slice");
    }
    take_str(s); // &'static str <: &'a str  ✔

    // The same works with struct fields.
    struct Holder<'a> {
        text: &'a str,
    }
    let holder = Holder { text: s }; // &'static str coerced to &'a str
    println!("  - Holder.text: {}", holder.text);

    // Covariant in the T position as well (when T contains lifetimes).
    fn take_ref_ref<'a>(_r: &'a &'a str) {
        println!("  - take_ref_ref accepted &&'a str");
    }
    let ref_s: &'static &'static str = &"hello";
    take_ref_ref(ref_s); // &&'static str <: &&'a str  ✔
}

// ------------------------------------------------
// 2. Covariance propagates into generic types
// ------------------------------------------------
fn covariance_in_containers() {
    let v: Vec<&'static str> = vec!["a", "b"];

    fn consume_slice<'a>(_s: &[&'a str]) {
        println!("  - consume_slice received slice of &str");
    }
    consume_slice(&v); // Vec<&'static str> <: &[&'a str] via covariance ✔

    // Box and other owning pointers are covariant in T.
    let b: Box<&'static str> = Box::new("boxed");
    fn use_boxed<'a>(_b: Box<&'a str>) {
        println!("  - got boxed string ref");
    }
    use_boxed(b);
}

// ------------------------------------------------
// 3. Contravariance: function arguments reverse subtyping
// ------------------------------------------------
fn contravariance_fn_example() {
    // If we need a function accepting a 'static reference...
    fn call_with_static<F>(f: F)
    where
        F: Fn(&'static str),
    {
        f("I am static");
    }

    // ...we can pass a closure that accepts ANY reference 'a (i.e., &'a str).
    // This is safe because a function that works for any 'a works for 'static.
    call_with_static(|s: &str| {
        println!("  - closure received: {}", s);
    });

    // The type system allows this because:
    //   if 'static: 'a, then  fn(&'a str)  <:  fn(&'static str)
    // (function argument type is contravariant).
}

// ------------------------------------------------
// 4. Invariance: &mut T  refuses subtyping
// ------------------------------------------------
fn invariance_mutable_reference() {
    // Although &'static T  <:  &'a T,   &mut &'static T  IS NOT a subtype of  &mut &'a T
    let mut static_str: &'static str = "static";

    // This assignment works because &'static str is a subtype of &'a str for a local 'a.
    let _r: &dyn Fn() = &|| {
        let _: &str = static_str; // coercion inside the closure
    };

    // But mutable references are invariant:
    // Uncomment the next lines to see the compile error:
    // ```rust
    // fn assign_mut<'a>(r: &mut &'a str) {
    //     // ...
    // }
    // let mut_ref: &mut &'static str = &mut static_str;
    // assign_mut(&mut static_str); // ERROR: expected `&mut &'a str`, found `&mut &'static str`
    // ```

    println!("  - (intentional compile error in comments)");
}

// ------------------------------------------------
// 5. Invariance of interior mutability containers
// ------------------------------------------------
use std::cell::Cell;

fn invariance_cell() {
    // Cell<&'static str> is invariant in T, so no coercion to Cell<&'a str>
    let cell_static: Cell<&'static str> = Cell::new("static");

    // fn take_cell<'a>(_c: Cell<&'a str>) {}
    // take_cell(cell_static); // ERROR: invariant

    // UnsafeCell behaves the same, as does RefCell and Mutex for the inner type.

    // This is intentional: allowing subtyping with interior mutability
    // would let you smuggle a shorter-lived reference into a cell and
    // then read it out with the wrong lifetime.
    println!("  - Cell<&'static str> cannot be used as Cell<&'a str> (good!)");
}

// ------------------------------------------------
// 6. Trait object subtyping
// ------------------------------------------------
use std::fmt::Display;

fn trait_object_example() {
    // dyn Display + 'static  <:  dyn Display + 'a
    fn display_static<'a>(d: &'a (dyn Display + 'static)) {
        println!("  - display: {}", d);
    }
    // But what we pass is a reference to a trait object with a shorter lifetime:
    let x: Box<dyn Display + 'static> = Box::new(42);
    let r: &dyn Display = &*x; // r is &(dyn Display + 'static)
    // This works because 'static: 'a, so dyn Display + 'static <: dyn Display + 'a
    display_static(r);

    // The same applies when you need 'static but have 'a? No, 'static is a subtype of 'a,
    // so dyn Display + 'static can be used where dyn Display + 'a is expected, not the reverse.
    // (The 'a in the function signature becomes a fresh 'a, which is longer than 'static in this call.)
    println!("  - Trait object subtyping via lifetime works.");
}