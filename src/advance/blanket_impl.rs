// Trait bounds can become somewhat unwieldy, especially if one of the Fn traits1 is involved
// and there are specific requirements on the output type. In such cases the introduction of a
// new trait may help reduce verbosity, eliminate some type parameters and thus increase
// expressiveness. Such a trait can be accompanied with a generic impl for all types satisfying
// the original bound.

// Let’s imagine some sort of monitoring or information gathering system. The system retrieves
// values of various types from diverse sources. It may derive from them some sort of status
// indicating issues. For example, the total amount of free memory should be above a certain
// theshold, and the user with the id 0 should always be named “root”.
//
// For management reasons, we probably want type erasure on the top level. However, we also
// need to provide specific (user configurable) assesments for specific types of data sources
// (e.g. thresholds and ranges for numerical types). And since sources for these values are diverse,
// we may choose to supply data sources as closures that return a value when called.
// Because we are probably getting those values from the operating system, we are likely
// confronted with operations that may fail.
//
// We thus may have settled on the following types and traits for handling specific values:

use std::fmt::Display;

// raw design which is too hard to read. The problems are:
// Three generic parameters: G, S, T.
//
// The bounds are scattered: G needs T, S needs T, and T itself must be Display.
//
// The type of the getter’s output (T) is duplicated as both a parameter and a bound on two
// different generics. This makes the signature hard to scan and understand quickly.
struct ValueV1<G: FnMut() -> Result<T, Error>, S: Fn(&T) -> Status, T: Display> {
    value: Option<T>,
    getter: G,
    status: S,
}

impl<G: FnMut() -> Result<T, Error>, S: Fn(&T) -> Status, T: Display> ValueV1<G, S, T> {
    pub fn update(&mut self) -> Result<(), Error> {
        (self.getter)().map(|v| self.value = Some(v))
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.value().map(&self.status)
    }
}

// ...

enum Status {
    // ...
}

struct Error {
    // ...
}

// refine the code with a trait.
// what it improved:
// Only two generic parameters remain: G and S.
//
// The output type is now an associated type of the Getter trait (G::Output), not a standalone T.
//
// The Display bound is on the associated type itself, so it’s enforced without cluttering the
// struct’s signature.
//
// The status closure’s input type is expressed directly as &G::Output – no separate T needed.
//
// The result: the struct’s header is simpler and immediately communicates “this stores a getter
// and a status function that operates on the getter’s output.”

trait Getter {
    type Output: Display;

    fn get_value(&mut self) -> Result<Self::Output, Error>;
}

// The blanket implementation:
// impl Getter trait for a generic type F which should satisfy the original bound:
// FnMut() ->Result<T, Error>, is a trait bound for F, and T is the associated type
// Output of the Getter trait.

// This says: any callable that matches the original FnMut signature automatically implements Getter.
// So you can still pass closures (or function pointers) to ValueV2 exactly as before –
// no extra code required.
impl<F: FnMut() -> Result<T, Error>, T: Display> Getter for F {
    type Output = T;

    fn get_value(&mut self) -> Result<Self::Output, Error> {
        self()
    }
}

struct ValueV2<G: Getter, S: Fn(&G::Output) -> Status> {
    value: Option<G::Output>,
    getter: G,
    status: S,
}

// Why is this better?
// Cleaner public API – users see ValueV2<G, S> instead of a tangle of bounds.
//
// Focused abstraction – the Getter trait captures the concept of “something that can be called
// to produce a value”, hiding the exact callable type. This could later be extended with custom
// implementations (e.g., a remote data fetcher) without touching ValueV2.
//
// Easier maintenance – if you need to change the getter’s contract, you only modify the trait
// and its blanket impl; the struct’s signature stays stable.
//
// No loss of functionality – the value() and status() methods work exactly as before because
// they just use G::Output where T used to appear.


// USE CASES of blanket impl.
// Universal conversion traits (From / Into)

// If U knows how to construct itself from a T, then every T
// automatically get the ability to turn itself into a U via .into().

// You write one From implementation, and the compiler hands you the reverse direction for free.
// This is what the comment “Write half the code; the rest is derived for free” means.
/*
    impl<T, U> Into<U> for T // blanket impl Into<U> for any T that implement From<T>.
    where
        U: From<T>, // this bound tells your type implements From<T>.
    {
        fn into(self) -> U {
            U::from(self) // call your implementation of From<T> to convert self into U.
        }
    }
*/

fn blanket_impl_universal_conversion() {
    #[derive(Debug, PartialEq)]
    struct UserName(String);

    // From a string slice.
    impl From<&str> for UserName {
        fn from(name: &str) -> Self {
            UserName(name.trim().to_string())
        }
    }

    // From an owned String
    impl From<String> for UserName {
        fn from(name: String) -> Self {
            UserName(name.trim().to_string())
        }
    }

    // .into() the blanket impl eventually calls your from() of Trait From<T> to construct
    // UserName.
    let from_str: UserName = "  alice  ".into();
    let from_string: UserName = String::from(" bob ").into();

    assert_eq!(from_str, UserName("alice".into()));
    assert_eq!(from_string, UserName("bob".into()));
}

// Extension traits - adding methods to existing types
// You can add utility methods to all types implementing a standard trait, even types you don’t own.
// For example, adding .log_error() to every Result.
fn blanket_impl_feature_extension() {

    // define the extension trait for new feature.
    trait ResultExt<T, E> {
        fn log_error(self) -> Option<T>;
    }

    // implement the new trait for a specific T, here it's the Result<T, E>.
    impl<T, E: Display> ResultExt<T, E> for Result<T, E> {
        fn log_error(self) -> Option<T> {
            match self {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("Error: {e}");
                    None
                }
            }
        }
    }
}

// Bridging traits automatically.
// One trait can automatically imply another for any type that implements a given trait.
// For instance, the standard library’s ToString blanket impl:
/*
impl<T: Display + ?Sized> ToString for T {
    fn to_string(&self) -> String {
        // Uses the Display implementation to format into a String
        use std::fmt::Write;
        let mut buf = String::new();
        write!(&mut buf, "{}", self).expect("a Display implementation returned an error unexpectedly");
        buf
    }
}*/

fn blanket_impl_bridging_traits() {
    use std::fmt;

    struct Point {
        x: f64,
        y: f64,
    }

    // Minimal interface: just teach Point how to Display itself
    impl Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    impl From<Point> for String {
        fn from(p: Point) -> Self {
            format!("({}, {})", p.x, p.y)
        }
    }

    let p = Point { x: 3.14, y: 2.71 };

    // from Display:
    println!("Debug print: {}", p);                // uses Display

    // from the blanket ToString impl:
    let s: String = p.to_string();                 // ✅ works!
    assert_eq!(s, "(3.14, 2.71)");

    // also into String via .into() because From<Point> for String is implemented:
    let s2: String = p.into();                     // ✅ works!
    assert_eq!(s2, "(3.14, 2.71)");
}

// Marker and auto-traits (Send, Sync, Unpin)
// What are auto‑traits?
// Send, Sync, and Unpin are marker traits—they have no methods. The compiler doesn’t wait for
// you to implement them; it automatically derives them for your type if all its fields satisfy
// the trait. This is like a compile‑time blanket impl tailored to each type. If even one field
// is not Send, the whole type becomes !Send, and the compiler will prevent you from
// moving it to another thread.

// Concrete example: a database connection pool
// Imagine we’re building a simple connection pool that can be safely shared across threads.
// We want it to be Send + Sync without writing a single unsafe impl.
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;

// An inner pool state – all its fields are Send + Sync.
struct InnerPool {
    connections: Mutex<HashMap<String, Connection>>, // Mutex is both Send and Sync
    config:      PoolConfig,
}

// PoolConfig contains only Send + Sync data
struct PoolConfig {
    max_connections: usize,
    db_url:          String,   // String is Send + Sync
}

struct Connection {
    id:     u64,
    active: bool,
}

// The public pool handle – just a thread-safe reference-counted pointer
#[derive(Clone)]
struct Pool {
    inner: Arc<InnerPool>,
}

// Now observe: we never wrote unsafe impl Send for Pool {} or unsafe impl Sync for Pool {}.
// Yet the following compiles and runs perfectly:
fn auto_trait_via_connection_pool() {
    let pool = Pool {
        inner: Arc::new(InnerPool {
            connections: Mutex::new(HashMap::new()),
            config: PoolConfig {
                max_connections: 10,
                db_url: "postgres://localhost".to_string(),
            },
        }),
    };

    // Move a clone to another thread – Pool must be Send.
    let pool_clone = pool.clone();
    std::thread::spawn(move || {
        // Inside the thread we can lock the mutex – Pool must be Sync.
        let mut conns = pool_clone.inner.connections.lock().unwrap();
        conns.insert("conn1".into(), Connection { id: 1, active: true });
        println!("Thread inserted a connection");
    }).join().unwrap();

    // Main thread also accesses the pool
    let conns = pool.inner.connections.lock().unwrap();
    println!("Main thread sees {} connections", conns.len());
}

// Making closures and functions fit into your abstractions
// A blanket impl over FnOnce (or FnMut, Fn) turns any callable into a trait object or into a
// type that implements your own trait.
// For example, turning closures into parsers:
fn turn_closures_into_parser() {
    use std::error::Error;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    // Sized mean the object can be owned or consumed.
    // Here we defined a Parser, is anything that can consume itself and produce a Result of its
    // associated type Output. This is a very common pattern for parsing libraries, where you have
    // some input type (e.g., a string or a token stream) and you want to parse it into some
    // structured output (e.g., an AST or a configuration object). By defining a Parser trait with
    // an associated type Output, you can abstract over the specific input and output types, and
    // just require that any type that implements Parser can be consumed to produce a Result of
    // its Output type.
    pub trait Parser: Sized {
        type Output;
        fn parse(self) -> Result<Self::Output>;
    }


    // Now this block makes any callable (a closure or a function pointer) that takes nothing and
    // returns a Result<T> is a parser. As in the hierarchy of fn trait: Fn is a FnMut, FnMut is a
    // FnOnce, thus bound of F tells, for any callable takes nothing and return Result<T> is a
    // parser, they will automatically implement the fn parse(self) -> Result<T>.
    impl<F, T> Parser for F
    where
        F: FnOnce() -> Result<T>, // fn -> Fn -> FnMut -> FnOnce
    {
        type Output = T;

        fn parse(self) -> Result<Self::Output> {
            todo!()
        }
    }

    // A different trait: ParseMacroInput is for types that know how to create themselves from a
    // “macro input”.
    // The parse() method here takes no self – it’s an associated function (like a static method).
    // It returns Result<Self>, i.e., an instance of the type that implements the trait.
    pub trait ParseMacroInput: Sized {
        fn parse() -> Result<Self>;
    }

    // A blanket impl that makes every type T implement ParseMacroInput. This is usually too broad
    // for real code, but it’s used here to illustrate the pattern. The method body is again todo!()
    // – in reality you’d fill it with the actual parsing logic for that type.
    impl<T> ParseMacroInput for T {
        fn parse() -> Result<Self> {
            todo!()
        }
    }

    // This generic function ties the two traits together.
    //
    // T::parse gets the static method from ParseMacroInput. Because parse is a function without
    // arguments that returns Result<T>, the expression T::parse has the type fn() -> Result<T> (a function pointer).
    //
    // let x = T::parse; assigns that function pointer to x.
    //
    // x.parse() calls the parse method from the Parser trait. Since x is a function pointer that
    // matches FnOnce() -> Result<T>, the blanket impl for Parser applies. Inside that impl,
    // parse(self) calls self(), which is exactly T::parse().
    //
    // So parse::<T>() ends up calling T::parse() – the static parse method. The whole machinery
    // just routes the call through a function pointer and the Parser trait.
    pub fn parse<T: ParseMacroInput>() -> Result<T> {
        let x = T::parse; // fn parse<T>() -> Result<T>, x is a function pointer
        x.parse() // The Parser trait is implemented for all pointer of functions that return a Result
    }
}

// The pattern allows uniform treatment of closures and static constructors as “parsers”.
//
// If you have a type that implements ParseMacroInput (say, via a derive macro), its constructor
// can be used wherever a Parser is expected.
//
// Meanwhile, any ad‑hoc closure can also be a Parser.
//
// The generic parse<T>() function provides a convenient entry point: give it a type T, and it
// uses the type’s static parse method through the Parser interface.
//
// This is reminiscent of how parser combinator libraries (like nom or combine) allow you to pass
// either named functions or closures as sub‑parsers. The Rust trait system makes this seamless:
// every FnOnce() -> Result<T> is automatically a Parser, and the blanket impl takes care of the
// mapping. The ParseMacroInput trait then provides a hook for types that can generate themselves,
// bridging the gap between a type’s static constructor and the Parser trait world.


// Derive macros rely on blanket impls for primitives
// Serde’s Serialize and Deserialize derive macros work because there are blanket impls for all
// fundamental types (i32, &str, Vec<T>, HashMap<K, V>, etc.) where T is serializable.
// When you #[derive(Serialize)], the generated code calls these building‑block methods.
// Without blanket impls, every primitive would need manual serialization in every crate.
//
// Why it’s great – A single derive handles arbitrarily complex structures
// because base cases are covered generically.
fn blanket_impl_derive_macros() {
    // Simplified – the actual impls are in serde
    impl Serialize for i32 {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_i32(*self)
        }
    }

    impl Serialize for bool {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            todo!()
        } /* … */ }
    impl Serialize for f64 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            todo!()
        } /* … */ }
    impl Serialize for str {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            todo!()
        } /* … */ }
    impl Serialize for String {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            todo!()
        } /* … */ }

    // Blanket impl for Vec<T> – works for any T that is Serialize
    impl<T: Serialize> Serialize for Vec<T> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut seq = serializer.serialize_seq(Some(self.len()))?;
            for element in self {
                seq.serialize_element(element)?;
            }
            seq.end()
        }
    }

    // Blanket impl for Option<T>
    impl<T: Serialize> Serialize for Option<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            todo!()
        } /* … */ }

    // Blanket impl for HashMap<K, V> where K: Serialize, V: Serialize
    impl<K, V> Serialize for HashMap<K, V>
    where
        K: Serialize,
        V: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            todo!()
        } /* … */
    }
}

// Flexible generic APIs with minimal bounds.
// Let’s look at how blanket impls enable flexible APIs that accept many input types without
// forcing the caller to convert anything.
// The “magic” blanket impls
// The standard library defines a number of wide-reaching blanket impls for AsRef:
// If T itself can give a reference to U, then a shared reference to T can too.
/*
    impl<T: ?Sized, U: ?Sized> AsRef<U> for &T
    where
        T: AsRef<U>,
    {
        fn as_ref(&self) -> &U {
            (**self).as_ref()
        }
    }

    // Similarly for mutable references
    impl<T: ?Sized, U: ?Sized> AsRef<U> for &mut T
    where
        T: AsRef<U>,
    { /* … */ }

    impl AsRef<str> for str    { … }   // trivial
    impl AsRef<str> for String { … }   // returns self.as_str()
    impl AsRef<str> for Cow<'_, str> { … }
    // Taken together, these impls mean that any reference to a type that implements AsRef<str>
    also implements AsRef<str>. So &String, &Cow<str>, and of course &str all satisfy AsRef<str>.
     This is exactly what we need for a generic API.
*/

// a greeting function.
use std::borrow::Cow;

fn print_uppercase_greeting(name: impl AsRef<str>) {
    // .as_ref() returns a &str, no matter what type was passed
    let name_str: &str = name.as_ref();
    println!("HELLO, {}!", name_str.to_uppercase());
}

fn blanket_impl_flexible_api() {
    // 1. String slice
    print_uppercase_greeting("alice");

    // 2. Owned String
    let bob = String::from("bob");
    print_uppercase_greeting(bob);          // String moved, but AsRef used

    // 3. Reference to a String
    let carol = String::from("carol");
    print_uppercase_greeting(&carol);       // &String

    // 4. Cow – borrowed or owned
    let dave: Cow<str> = Cow::Borrowed("dave");
    print_uppercase_greeting(dave);
}

// filesystem paths
fn blanket_impl_as_ref_fs_path() {
    use std::path::Path;
    use std::fs;

    fn print_file_content(path: impl AsRef<Path>) {
        let content = fs::read_to_string(&path).unwrap();
        println!("{}", content);
    }

    print_file_content("config.toml");                 // &str
    print_file_content(String::from("config.toml"));   // String
    let path_buf = std::path::PathBuf::from("config.toml");
    print_file_content(&path_buf);                     // &PathBuf
    print_file_content(path_buf);                      // PathBuf
}