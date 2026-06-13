// What if in some cases we want a type to behave similar to another type or enforce some behaviour
// at compile time when using only type aliases would not be enough?
//
// For example, if we want to create a custom Display implementation for String due to security
// considerations (e.g. passwords).
//
// For such cases we could use the Newtype pattern to provide type safety and encapsulation.

// Use a tuple struct with a single field to make an opaque wrapper for a type. This creates a
// new type, rather than an alias to a type (type items).
fn new_type() {
    use std::fmt::Display;

    // Create Newtype Password to override the Display trait for String
    struct Password(String);

    impl Display for Password {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "****************")
        }
    }

    fn main() {
        let unsecured_password: String = "ThisIsMyPassword".to_string();
        let secured_password: Password = Password(unsecured_password.clone());
        println!("unsecured_password: {unsecured_password}");
        println!("secured_password: {secured_password}");
    }
}