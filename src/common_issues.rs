

// returning data with a static lifetime bound.
// A Result<T, E> is an enum, it accepts either Ok(T) or Err(E) as the return value. In below case,
// the function save returns a Result<(), &'static str>, which means it can return either Ok(())
// or Err(&'static str). When we return Err("data is empty"), we are returning a string literal,
// which has a static lifetime. Therefore, the string literal "data is empty" can be used as the
// error value in the Result, satisfying the requirement of returning a &'static str.
fn save(data: &str) -> Result<(), &'static str> {
    if data.is_empty() {
        // The Result requires an Err wrapped reference of str with a static bound,
        // the string literal "data is empty" have a static lifetime.
        return Err("data is empty");
    }
    Ok(())
}

fn greeting() -> &'static str {
    "Hello, World!"
}

// Storing String in structs is sometimes correct, but if the struct is shorted-lived and the source
// data outlives it, a &str avoid allocation.
struct Config<'a> {
    name: &'a str,
}

// Do not misuse to_string(). Sometimes to_owned() is faster and meaningful.
fn to_string() {
    // works, but not the best. As it invokes the Display trait, which is slower for non-string types.
    //let s: String = "hello".to_string();
    let s: String = "hello".to_owned(); // is clearer in intent: make an owned copy.
    // or
    let s: String = String::from("hello"); // this is better as well.
}


// Common Deriving problems: too few or too many traits.
struct Point {x: f64, y: f64} // can't print or compare.

// When derive PartialEq for Coord {x:i32, y:i32}, does it compare the x and y fields for equality?
// Yes, it does. The derived implementation of PartialEq for Coord will compare the x fields of both
// instances and the y fields of both instances, and return true if both pairs of fields are equal,
// and false otherwise.
// Node: Only add copy for small, cheap types. Copy is critical for types that are used in large
// quantities, such as small numeric types (e.g., i32, f64) or simple structs that contain only
// Copy types. For larger or more complex types, implementing Copy can lead to unintended
// consequences, such as accidentally copying large amounts of data when you intended to move it.
// In such cases, it's often better to implement Clone instead, which allows for explicit copying
// when needed.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Coord { x: i32, y: i32 }

// DO NOT use println! in library functions:
// It takes over the callers' output. This makes them untestable and unusable in non-CLI context.
pub fn process(data: &str) {
    println!("processing data: {}", data); // bad in a lib.
}

// instead, use log::info; crate.
use log::info;
pub fn process_better(data: &str) {
    info!("processing data: {}", data);
}

