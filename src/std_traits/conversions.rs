// Conversion — From/Into, TryFrom/TryInto, AsRef/AsMut, FromStr are the four pillars.
// From for infallible, TryFrom for fallible, AsRef for flexible function params, FromStr for .parse().


// 1. From<T> / Into<T>
// Infallible type conversion. Implement From on the target type — Into is provided
// automatically via blanket impl. The workhorse of idiomatic Rust conversions.
fn from_to() {
    struct Celsius(f64);
    struct Fahrenheit(f64);

    impl From<Celsius> for Fahrenheit {
        fn from(c: Celsius) -> Self {
            Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
        }
    }

    impl From<Fahrenheit> for Celsius {
        fn from(fahrenheit: Fahrenheit) -> Self {
            Celsius((fahrenheit.0 / 5.0) + 32.0)
        }
    }

    let boiling = Celsius(100.0);
    let f = Fahrenheit::from(boiling); // explicit
    let f: Fahrenheit = Celsius(0.0).into(); // via blanket Into

    // std examples:
    let s = String::from("hello");    // &str → String
    let n: i64 = i64::from(42i32);   // i32 → i64 (widening)
}

// 2. TryFrom<T> / TryInto<T>
// Fallible type conversion. Returns Result. Use when the conversion can legitimately fail
// (overflow, invalid input, out of range).
fn try_from_to() {
    use std::convert::TryFrom;

    struct EvenNumber(i32);

    impl TryFrom<i32> for EvenNumber {
        type Error = String;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            if value % 2 == 0 {
                Ok(EvenNumber(value))
            } else {
                Err(format!("{} is not an even number", value))
            }
        }
    }

    let even = EvenNumber::try_from(4); // Ok(EvenNumber(4))
    let odd = EvenNumber::try_from(3);  // Err("3 is not an even number")
}

// 3. AsRef<T> / AsMut<T>
// Cheap reference-to-reference conversion. Use in function parameters to accept multiple types
// that all borrow as T. The idiomatic way to write flexible APIs.
fn as_ref_as_mut() {
    // Accept &str, String, Box, Cow — anything that borrows as str
    fn print(s: impl AsRef<str>) {
        println!("{}", s.as_ref());
    }
    print("literal");                  // ✅
    print(String::from("owned"));      // ✅
    print(Box::from("boxed"));         // ✅

    // fs::File::open uses AsRef — pass &str, String, PathBuf, &Path
    use std::fs;
    fn open_file(p: impl AsRef<std::path::Path>) -> std::io::Result<fs::File> {
        fs::File::open(p)
    }
    open_file("foo.txt");              // ✅ &str
    open_file(std::path::PathBuf::from("foo.txt")); // ✅ PathBuf
}

// 4. FromStr
// Enables .parse() on string slices. Returns Result. The idiomatic way to parse a type from text.
// Implement this to make your type parseable.
fn from_str() {
    use std::str::FromStr;

    #[derive(Debug)]
    struct Color { r: u8, g: u8, b: u8 }

    #[derive(Debug)]
    struct ParseColorError;

    impl FromStr for Color {
        type Err = ParseColorError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() != 3 { return Err(ParseColorError); }
            Ok(Color {
                r: parts[0].trim().parse().map_err(|_| ParseColorError)?,
                g: parts[1].trim().parse().map_err(|_| ParseColorError)?,
                b: parts[2].trim().parse().map_err(|_| ParseColorError)?,
            })
        }
    }

    if let Ok(Color {r, g, b}) = "255, 0, 0".parse() {
        println!("Parsed color: r={}, g={}, b={}", r, g, b);
    }
}
