
// Trait vs. Trait Object.
// Trait – A set of method signatures. It is an abstract interface;
// it has no concrete runtime representation until implemented by a type.
//
// Trait object – A runtime construct that allows dynamic dispatch.
// Represented by dyn Trait, it stores a pointer to the concrete value and a pointer to a vtable.
// The type of the concrete value is erased.

use std::fmt::{Debug, Display};

trait Greet {
    fn greet(&self);
}

// Trait used as a bound (static, generics)
fn say_hello<T: Greet>(entity: &T) {
    entity.greet();
}

// Trait object (dynamic, type erased)
fn say_hello_dyn(entity: &dyn Greet) {
    entity.greet();
}

// dyn keyword for trait object.
// dyn is used before a trait name to explicitly create a trait object.
// It signals that dispatch will be dynamic. The dyn keyword is required
// for all trait objects in modern Rust editions, making dynamic dispatch explicit.
fn trait_example() {

    impl Greet for &'static str {
        fn greet(&self) {
            println!("Hello Static");
        }
    }

    let s = "static";
    let greetable: &dyn Greet = &s;   // trait object

    // Without `dyn`, the compiler issues a warning/error (edition 2018+)
}

// Static vs. Dynamic Dispatch.
// Static dispatch – The compiler knows the concrete type at compile time.
// Methods are called directly (monomorphization).
// No runtime overhead, but the code is duplicated for each concrete type.
//
// Dynamic dispatch – The exact type is not known until runtime.
// Calls go through a vtable (indirection).
// Slightly slower, but allows heterogeneous collections and reduces code bloat.

// Static dispatch: return type with impl Trait
// impl Trait in return position is static dispatch: the returned type is concrete,
// just hidden from the caller. The compiler still knows the exact type and monomorphizes.
fn static_trait_example() {
    // Returns some type that implements Iterator<Item = i32>
    fn count_up_to(limit: i32) -> impl Iterator<Item = i32> {
        (0..limit).filter(|x| x % 2 == 0)
    }

    // The caller gets a concrete, but opaque, type (static dispatch).
    let iter = count_up_to(10);
    for i in iter {
        println!("{i}");
    }
}


// Dynamic dispatch: return type with Box<dyn Trait>
// Using a Box<dyn Trait> returns a heap-allocated trait object.
// The concrete type is erased; dispatch is dynamic.
fn dynamic_trait_example() {

    impl Greet for String {
        fn greet(&self) {
            println!("Hello DYN");
        }
    }

    fn make_greetable(s: String) -> Box<dyn Greet> {
        Box::new(s)   // String implements Greet
    }

    let obj = make_greetable("world".into());
    obj.greet();      // dynamic dispatch
}

// Concise Trait Syntax.
// impl Trait is syntax sugar (less verbose than generic parameters)
// Using impl Trait in argument position is a shorthand for a generic parameter.
// It allows the function to accept any type implementing the trait,
// and each argument can be a different type.
fn concise_trait_syntax() {
    pub fn notify(item1: &impl Greet, item2: &impl Greet) {
        // item1 and item2 can be different types, both implementing Summary
    }

    // Equivalent to below fn which is not concise, but more explicit about the generic parameters.
    pub fn notify1<T: Greet, U: Greet>(item1: &T, item2: &U) {
        // same flexibility
    }

    // With trait bound syntax, it enforces a single type.
    // If you need both arguments to be the same type, you must use a trait bound:
    pub fn notify2<T: Greet>(item1: &T, item2: &T) {
        // item1 and item2 must have the same type T
    }

    // where clause for clearer syntax.
    // The where clause becomes essential when using associated type bounds or complex constraints.
    fn some_function<T, U>(t: &T, u: &U) -> i32
    where
        T: Display + Clone,
        U: Clone + Debug,
    {
        // ...
        0
    }

    // without where, you have to write:
    fn some_function2<T: Display + Clone, U: Clone + Debug>(t: &T, u: &U) -> i32 {
        // ...
        0
    }

}


// SuperTrait example.
// CompSciStudent-> Programmer + Student ( Student -> Person)

trait Person {
    fn name(&self) -> String;
}

// Person is a supertrait of Student.
// Implementing Student requires you to also impl Person.
trait Student: Person {
    fn university(&self) -> String;
}

trait Programmer {
    fn fav_language(&self) -> String;
}

// CompSciStudent (computer science student) is a subtrait of both Programmer
// and Student. Implementing CompSciStudent requires you to impl both supertraits.
trait CompSciStudent: Programmer + Student {
    fn git_username(&self) -> String;
}

fn comp_sci_student_greeting(student: &dyn CompSciStudent) -> String {
    format!(
        "My name is {} and I attend {}. My favorite language is {}. My Git username is {}",
        student.name(),
        student.university(),
        student.fav_language(),
        student.git_username()
    )
}

struct CSStudent {
    name: String,
    university: String,
    fav_language: String,
    git_username: String
}

impl Programmer for CSStudent {
    fn fav_language(&self) -> String {
        self.fav_language.clone()
    }
}

impl Student for CSStudent {
    fn university(&self) -> String {
        self.university.clone()
    }
}

impl Person for CSStudent {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl CompSciStudent for CSStudent {
    fn git_username(&self) -> String {
        self.git_username.clone()
    }
}

fn super_trait_example() {
    let student = CSStudent {
        name: String::from("Alice"),
        university: String::from("MIT"),
        fav_language: String::from("Rust"),
        git_username: String::from("alice_codes"),
    };

    let greeting = comp_sci_student_greeting(&student);
    println!("{}", greeting);
}

// Disambiguating overlapping traits.
trait UsernameWidget {
    // Get the selected username out of this widget
    fn get(&self) -> String;
}

trait AgeWidget {
    // Get the selected age out of this widget
    fn get(&self) -> u8;
}

// A form with both a UsernameWidget and an AgeWidget
struct Form {
    username: String,
    age: u8,
}

impl UsernameWidget for Form {
    fn get(&self) -> String {
        self.username.clone()
    }
}

impl AgeWidget for Form {
    fn get(&self) -> u8 {
        self.age
    }
}

fn disambiguating_overlap_traits() {
    let form = Form {
        username: "rustacean".to_owned(),
        age: 28,
    };

    // If you uncomment this line, you'll get an error saying
    // "multiple `get` found". Because, after all, there are multiple methods
    // named `get`.
    // println!("{}", form.get());

    let username = <Form as UsernameWidget>::get(&form);
    assert_eq!("rustacean".to_owned(), username);
    let age = <Form as AgeWidget>::get(&form);
    assert_eq!(28, age);
}