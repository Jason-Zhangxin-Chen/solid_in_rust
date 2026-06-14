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