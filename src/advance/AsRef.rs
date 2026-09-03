// AsRef<T> is a trait for cheap, non‑ownership‑consuming, explicit reference‑to‑reference
// conversions.
// It lets you turn &self into &T in a generic way, so a function can accept many different types
// that can all be “viewed as” a reference to T

// Use cases:

// 1. Flexible function parameters (the main use‑case)
// When you write a function that needs a &str, you could just take &str, but that forces the
// caller to borrow or explicitly convert. Using AsRef<str> lets callers pass a &str, a String,
// a Cow<str>, or anything that implements AsRef<str>.

fn as_ref_fn_parameter() {
    fn greet(name: impl AsRef<str>) {
        println!("Hello, {}!", name.as_ref());
    }

    greet("Alice");                       // &str
    greet(String::from("Bob"));              // String
    greet(std::borrow::Cow::from("Charlie"));// Cow<str>


    fn write_all(data: impl AsRef<[u8]>) {
        let bytes: &[u8] = data.as_ref();
        // write bytes somewhere
    }

    write_all(b"hello");        // &[u8; 5] derefs to &[u8] via AsRef<[u8]>
    write_all(vec![1, 2, 3]);   // Vec<u8>
}


// 2. std::fs::File::open and paths.
// File::open accepts P: AsRef<Path>. That’s why you can pass:
//
// &str
//
// String
//
// Path
//
// PathBuf
//
// OsStr
//
// OsString
//
// Component/PrefixComponent

fn as_ref_file_path() {
    use std::path::PathBuf;

    let _ = std::fs::File::open("config.toml");          // &str → AsRef<Path>
    let _ = std::fs::File::open(String::from("data.txt")); // String → AsRef<Path>
    let _ = std::fs::File::open(PathBuf::from("log"));     // PathBuf → AsRef<Path>
}

// 3. Exposing inner data from a newtype wrapper
// If you have a wrapper around a String (e.g., UserId(String)) and you want to access its
// inner &str without consuming it, implement AsRef<str>:
fn as_ref_polymorphism() {
    struct UserId(String);

    // impl AsRef<str> for UserId.
    impl AsRef<str> for UserId {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    // the parameter of validate should implement AsRef<str>.
    fn validate(id: impl AsRef<str>) { /* ... */ }

    let uid = UserId("abc123".into()); // as &str implements From<String>, thus into convert it to String.
    validate(&uid);       // works via our AsRef<str> impl
    validate(uid);        // also works if we also implement AsRef<str> for UserId (consuming is not allowed)
}

// 4. Converting between string types
// AsRef<Path> is implemented for OsStr, OsString, str, String, etc.
// Similarly, CStr implements AsRef<CStr> (trivially), and CString implements AsRef<CStr>.
// This allows you to write functions that work with either owned or borrowed foreign‑string types.
fn as_ref_converting_string_types() {
    use std::ffi::{CStr, CString};

    fn print_cstr(s: impl AsRef<CStr>) {
        println!("{:?}", s.as_ref());
    }

    print_cstr(CStr::from_bytes_with_nul(b"hello\0").unwrap());
    print_cstr(CString::new("world").unwrap());
}

// 5. AsRef vs Deref (and why not just use Deref)
// Deref is implicit and can only have one target. It’s meant for “smart pointer” semantics
// (e.g., Box<T> → T).
//
// AsRef is explicit and can be implemented multiple times for different target types. For example,
// String implements AsRef<str>, AsRef<[u8]>, AsRef<Path>, etc.
// Deref cannot provide all of those.
//
// So AsRef is the right tool when you want to accept any type that can cheaply give you a &T
// and you don’t want to own the value. It’s a cornerstone of idiomatic, flexible Rust APIs.