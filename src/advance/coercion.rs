// Coercion, What is Coercion?
// Coercion is an implicit type conversion the compiler performs automatically at specific sites —
// no as cast needed. Unlike explicit casts, coercions are always safe and never lose information.

// When does it happen?
fn when_coercion_happen() {
    // 1. Let bindings with explicit type annotation
    let s: &str = &String::from("hello");

    // 2. Function arguments
    fn foo1(s: &str) {}
    foo1(&String::from("hello"));  // &String coerced to &str

    // 3. Return expressions
    // fn foo() -> &'static str { &String::from("hi") } // won't work — but shows the site

    // 4. Struct/enum field initialization
    // struct Foo { name: &'static str }
    // Foo { name: &String::from("x") }; // coercion happens here

    // 5. Array/vec elements
    let arr: [&str; 2] = [&String::from("a"), &String::from("b")];
}

// Deref Coercion - the most common.
// When T implements Deref<Target = U>, &T coerces to &U automatically:
fn deref_coercion() {
    // &String -> &str  (String: Deref<Target=str>)
    let s = String::from("hello");
    let r: &str = &s;           // coercion
    fn greet(name: &str) {}
    greet(&s);                  // coercion at call site

    // &Vec<T> -> &[T]  (Vec<T>: Deref<Target=[T]>)
    let v = vec![1, 2, 3];
    let sl: &[i32] = &v;        // coercion
    fn sum(s: &[i32]) {}
    sum(&v);                    // coercion at call site

    // &Box<T> -> &T  (Box<T>: Deref<Target=T>)
    let b = Box::new(42i32);
    let r: &i32 = &b;           // coercion


    // Deref coercions chain — the compiler keeps dereffing until it finds the target type:
    // Box<String> -> String -> str — two hops!
    let b = Box::new(String::from("hello"));
    let s: &str = &b;           // &Box<String> -> &String -> &str, automatic
}

// Unsized Coercion - sized to unsized.
// Coercing from a concrete sized type to its unsized counterpart:
fn size_to_unsize_coercion() {
    // [T; N] -> [T]  (array to slice)
    let arr = [1, 2, 3];
    let sl: &[i32] = &arr;      // &[i32; 3] coerced to &[i32]

    // T -> dyn Trait  (concrete type to trait object)
    trait Animal { fn bark(&self); }
    struct Dog;
    impl Animal for Dog { fn bark(&self) { println!("woof"); } }

    let d = Dog;
    let a: &dyn Animal = &d;    // &Dog coerced to &dyn Animal — vtable created here

    // Same with Box:
    let b: Box<dyn Animal> = Box::new(Dog); // Box<Dog> coerced to Box<dyn Animal>
}

// Pointer Coercion
fn pointer_coercion() {
    // &mut T -> &T  (mutable to shared reference)
    let mut x = 5;
    let r: &i32 = &mut x;       // safe — downgrading mutability

    // &T -> *const T  (reference to raw pointer)
    let x = 42;
    let p: *const i32 = &x;     // coercion

    // &mut T -> *mut T
    let mut x = 42;
    let p: *mut i32 = &mut x;   // coercion

    // *mut T -> *const T
    let mut x = 42;
    let p: *mut i32 = &mut x;
    let cp: *const i32 = p;     // coercion
}

// Function pointer coercions.
// Non-capturing closures and function items coerce to function pointers:
fn function_pointer_coercion() {
    // fn item -> fn pointer
    fn double(x: i32) -> i32 { x * 2 }
    let f: fn(i32) -> i32 = double;    // coercion

    // non-capturing closure -> fn pointer
    let f: fn(i32) -> i32 = |x| x * 2; // coercion

    // capturing closure — does NOT coerce (it has state)
    // let factor = 3;
    // let f: fn(i32) -> i32 = |x| x * factor; // ❌ compile error
}

// Coercions vs as cast -- key distinction.
fn diff_coercion_as_cast() {
    // Coercion — implicit, safe, no data loss
    let s = String::from("hello");
    let r: &str = &s;           // fine

    // as cast — explicit, can truncate or reinterpret
    let x: i32 = 1000;
    let y = x as u8;            // truncates to 232 — data loss!
    let p = &x as *const i32;   // raw pointer cast

    // as cannot do deref coercions:
    fn greet(s: &str) {}
    // greet(&s as &str);       // awkward — just write greet(&s)
}

// Where coercion does not happen.
fn not_coercion() {
    // Generic functions — coercion does NOT fire through generics
    fn foo1<T>(x: T) {}
    let s = String::from("hi");
    foo1(&s);    // T = &String, NOT &str — no coercion into generic T

    // You need explicit trait bounds to get the coercion:
    fn foo2<T: AsRef<str>>(x: T) {}  // now works with both &String and &str
    // or just:
    fn foo3(x: &str) {}              // coercion fires at concrete type sites
}

// Deref coercion with your own types.
// You can enable deref coercion for your own smart pointers:
fn deref_coercion_custom_types() {
    use std::ops::Deref;

    struct MyBox<T>(T);

    impl<T> Deref for MyBox<T> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.0
        }
    }

    let b = MyBox(String::from("hello"));
    let s: &str = &b;   // MyBox<String> -> String -> str — two-hop deref coercion!

    fn greet(s: &str) { println!("{s}"); }
    greet(&b);          // works — coercion fires automatically
}

// Quick reference
// Coercion                         From                        To
// Deref                            &String                     &str
// Deref                            &Vec<T>                     &[T]
// Deref                            &Box<T>                     &T
// Deref chain                      &Box<String>                &str
// Unsized                          &[T; N]                      &[T]
// Unsized                          &T (concrete)               &dyn Trait
// Unsized                          Box<T>                      Box<dyn Trait>
// Mutability                       &mut T                      &T
// Raw pointer                      &T                          *const T
// Raw pointer                      &mut T                      *mut T
// Fn pointer                       fn item                     fn(T) -> U
// Fn pointer                       non-capturing closure       fn(T) -> U
// The key intuition: coercions widen or relax a type — more specific to more general,
// owned to borrowed, mutable to immutable. They never narrow, never lose data, and never
// introduce runtime cost beyond what the conversion inherently requires
// (e.g. building a vtable for dyn Trait).