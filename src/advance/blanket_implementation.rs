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

// Another blanket impl as an example.
// The pattern demonstrates that Rust can treat plain functions as first‑class trait objects
// via blanket impls over Fn traits, and that you can layer a second trait (ParseMacroInput)
// to allow types to be constructed via a generic function that feeds into that same system.
// It’s a clean separation of concerns: “how to parse” (the callable) vs. “what type to produce”
// (the associated type / trait).
fn blanket_impl() {
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