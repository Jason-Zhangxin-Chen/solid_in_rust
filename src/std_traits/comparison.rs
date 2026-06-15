// Comparison — PartialEq/Eq for ==, PartialOrd/Ord for sorting, Hash for HashMap keys.
// Always derive these together.

// 1. PartialEq / Eq.
// PartialEq enables == and !=. Eq is a marker saying equality is total (reflexive: a == a always).
// f64 implements PartialEq but not Eq because NaN != NaN.
fn partial_eq() {
    #[derive(Debug, PartialEq)]
    struct Color { r: u8, g: u8, b: u8 }

    let red   = Color { r: 255, g: 0, b: 0 };
    let red2  = Color { r: 255, g: 0, b: 0 };
    let blue  = Color { r: 0,   g: 0, b: 255 };

    assert_eq!(red, red2);    // ✅
    assert_ne!(red, blue);    // ✅

    // Custom impl — e.g. case-insensitive string wrapper:
    struct CiStr(String);
    impl PartialEq for CiStr {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq_ignore_ascii_case(&other.0)
        }
    }

    // f64 is PartialEq but NOT Eq — NaN breaks reflexivity:
    assert!(f64::NAN != f64::NAN);  // true — NaN is not equal to itself
}


// 2. PartialOrd / Ord
// PartialOrd enables <, >, <=, >=. Ord is total ordering — required for sort(), min(), max().
// Derive both together. f64 only implements PartialOrd (NaN breaks total order).
fn partial_ord() {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Version { major: u32, minor: u32, patch: u32 }

    let mut versions = vec![
        Version { major: 1, minor: 2, patch: 0 },
        Version { major: 0, minor: 9, patch: 5 },
        Version { major: 1, minor: 0, patch: 0 },
    ];
    versions.sort();  // requires Ord — lexicographic by fields
    // [0.9.5, 1.0.0, 1.2.0]

    #[derive(PartialEq, Eq, PartialOrd)]
    struct Task {
        priority: u32,
        name: String,
    }

    // Custom Ord — e.g. sort by priority, then name:
    impl Ord for Task {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.priority.cmp(&self.priority)
                .then(self.name.cmp(&other.name))
        }
    }
}


// 3. Hash
// Enables a type to be used as a HashMap or HashSet key. Derive it for most types.
// Golden rule: if a == b then hash(a) == hash(b) — always derive Hash and Eq together.
fn hash() {
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct Point { x: i32, y: i32 }  // now usable as HashMap key

    let mut grid = HashMap::new();
    grid.insert(Point { x: 0, y: 0 }, "origin");
    grid.insert(Point { x: 1, y: 0 }, "right");

    let val = grid.get(&Point { x: 0, y: 0 });
    // Some("origin")

    // CRITICAL RULE: if you impl PartialEq manually, impl Hash manually too.
    // They must agree: a == b → hash(a) == hash(b)
    // Violating this causes silent HashMap bugs (wrong bucket lookups)
}