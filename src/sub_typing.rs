// subtyping_demo.rs
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
    fn take_str<'a>(_s: &'a str) {
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