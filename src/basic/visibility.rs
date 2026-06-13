
// In Rust, field visibility defaults to private (no pub keyword) and can be widened with pub or
// restricted pub(...) forms. The two versions of Publish illustrate this:
//
// Summary of Rust visibility modifiers
// Syntax	    Meaning
// (no pub)	    Private – accessible only within the current module and its descendants (child modules).
// pub	        Public – accessible everywhere.
// pub(crate)	Accessible within the current crate.
// pub(super)	Accessible within the parent module.
// pub(in some::path)	Accessible within the given ancestor module (e.g., pub(in crate::outer::inner)).
// pub(self)	Equivalent to no pub (private).

// How to think about visibility
// Private by default: Fields are hidden from anything outside the module.
//
// pub opens fully: use sparingly to maintain encapsulation.
//
// Restricted pub(...): fine-grained control – common in library crates to share implementation
// details with parent modules or the whole crate without exposing them publicly.
//
// So in the first Publish, all fields are completely private; in the second, ticket is fully
// public, exchange is crate‑internal, routing_key is meant to be parent‑module‑internal, and
// bits remains private.