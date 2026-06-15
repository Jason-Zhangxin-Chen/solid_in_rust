// Formatting — always derive Debug.
// Implement Display for user-facing output. Display also powers .to_string().

// 1. Display, std::fmt::Display.
// Human-readable formatting. Drives to_string(), println!("{}"), format!().
// Implement this for any type you want printable as a user-facing string.
fn display() {
    use std::fmt;

    struct User {
        name: String,
        age: u8,
    }

    impl fmt::Display for User {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "User {{ name: {}, age: {} }}", self.name, self.age)
        }
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    println!("{}", user); // Output: User { name: Alice, age: 30 }
}

// 2. Debug, std::fmt::Debug.
// Developer-facing formatting. Drives {:?} and {:#?} (pretty-print). Almost always derived.
// Required by assert_eq!, dbg!, and most test harnesses.
fn debug() {
    // Derive it — almost always the right call:
    #[derive(Debug)]
    struct Config { host: String, port: u16 }

    let c = Config { host: "localhost".into(), port: 8080 };
    println!("{c:?}");   // Config { host: "localhost", port: 8080 }
    println!("{c:#?}");  // pretty-printed multiline
    dbg!(&c);           // [src/main.rs:10] &c = Config { ... }

    struct Secret{}
    // Custom impl when you want to hide sensitive fields:
    impl std::fmt::Debug for Secret {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_struct("Secret").field("value", &"***").finish()
        }
    }
}