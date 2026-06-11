// =============================================================================
//  RUST WEEKLY REFRESHER  —  rust_weekly_refresher.rs
//  A single-file tour of every major Rust feature, with concise examples.
//  Run with:  rustc rust_weekly_refresher.rs && ./rust_weekly_refresher
//  Or:        cargo script rust_weekly_refresher.rs  (with cargo-script)
// =============================================================================
//
//  TABLE OF CONTENTS
//  ─────────────────────────────────────────────────────────────────────────
//   1.  Primitive Types & Variables
//   2.  Ownership, Move, Clone, Copy
//   3.  Borrowing & References
//   4.  Lifetimes
//   5.  Slices
//   6.  String Types  (&str vs String)
//   7.  Structs  (tuple / named / unit)
//   8.  Enums & Pattern Matching
//   9.  Option<T>
//  10.  Result<T, E> & Error Handling
//  11.  The ? Operator
//  12.  Traits
//  13.  Default Trait & derive Macros
//  14.  Generics
//  15.  Trait Objects  (dyn Trait)
//  16.  Associated Types
//  17.  Closures
//  18.  Iterators & Iterator Adapters
//  19.  Collections  (Vec, HashMap, HashSet, BTreeMap)
//  20.  Smart Pointers  (Box, Rc, Arc, RefCell, Cell, Weak, Cow)
//  21.  Interior Mutability
//  22.  Concurrency  (threads, Mutex, channels)
//  23.  Async / Await  (conceptual — no runtime needed to compile)
//  24.  Macros  (declarative)
//  25.  Modules & Visibility
//  26.  Type Aliases & Newtype Pattern
//  27.  Operator Overloading
//  28.  The Builder Pattern
//  29.  Iterating Custom Types  (impl Iterator)
//  30.  Raw Pointers & Unsafe
//  31.  Attributes & Lint Control
//  32.  Common Patterns Cheatsheet
// =============================================================================

#![allow(dead_code, unused_variables, unused_mut, unused_imports, clippy::all)]

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

fn main() {
    section_01_primitives();
    section_02_ownership();
    section_03_borrowing();
    section_04_lifetimes();
    section_05_slices();
    section_06_strings();
    section_07_structs();
    section_08_enums_and_matching();
    section_09_option();
    section_10_result();

    section_11_question_mark();
    section_12_traits();
    section_13_derive_and_default();
    section_14_generics();
    section_15_trait_objects();
    section_16_associated_types();
    section_17_closures();
    section_18_iterators();
    section_19_collections();
    section_20_smart_pointers();

    section_21_interior_mutability();
    section_22_concurrency();
    section_24_macros();
    section_26_type_aliases_newtype();
    section_27_operator_overloading();
    section_28_builder_pattern();
    section_29_custom_iterator();
    section_30_unsafe_raw_pointers();

    section_31_attributes();
    section_32_patterns_cheatsheet();

    println!("\n✓  All sections ran successfully.");
}

// =============================================================================
// §1  PRIMITIVE TYPES & VARIABLES
// =============================================================================
fn section_01_primitives() {
    // types' size.
    // Most type in Rust are sized, known at compile time.
    // Sized: String, i32, Dog.
    // Unsized: str,        just bytes, length unknown.
    // Unsized: [u8]        slice, length unknown.
    // Unsized: dyn Trait,  concrete type erased.

    // ── Integer types ────────────────────────────────────────────────────────
    let a: i8 = -128; // signed   8-bit
    let b: i16 = 32_000; // signed  16-bit  (underscores for readability)
    let c: i32 = -2_147_483_648; // signed  32-bit  (default integer)
    let d: i64 = 9_000_000_000_000;
    let e: i128 = 1;
    let f: isize = -1; // pointer-sized signed (usize on this platform)

    let u: u8 = 255;
    let v: u32 = 4_294_967_295;
    let w: usize = 42; // used for indexing / lengths

    // Integer literals: hex, octal, binary
    let hex = 0xFF_u8; // 255
    let oct = 0o17_u8; // 15
    let bin = 0b1111_0000_u8; // 240
    let byte = b'A'; // u8 = 65

    // ── Float types ──────────────────────────────────────────────────────────
    let fl32: f32 = 3.14;
    let fl64: f64 = std::f64::consts::PI; // default float

    // ── Bool & char ──────────────────────────────────────────────────────────
    let flag: bool = true;
    let ch: char = '🦀'; // char is a 4-byte Unicode scalar

    // ── Tuples ───────────────────────────────────────────────────────────────
    let tup: (i32, f64, bool) = (42, 3.14, true);
    let (x, y, z) = tup; // destructure
    let first = tup.0; // index access

    // ── Arrays (fixed-size, stack-allocated) ─────────────────────────────────
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    let zeros = [0i32; 100]; // 100 zeros
    let third = arr[2]; // 3
    let len = arr.len(); // 5

    // ── Variables & mutability ───────────────────────────────────────────────
    let immutable = 10; // immutable by default
    let mut mutable = 10;
    mutable += 1;

    // Shadowing — re-declare with same name (can change type)
    let shadow = "hello";
    let shadow = shadow.len(); // now shadow: usize = 5

    // Constants — must have explicit type, evaluated at compile time
    const MAX_POINTS: u32 = 100_000;
    static GREETING: &str = "hello"; // static lives for entire program

    // ── Unit type ────────────────────────────────────────────────────────────
    let unit: () = (); // functions that return nothing implicitly return ()

    println!("[§1] primitives OK — π ≈ {:.4}", fl64);
}

// =============================================================================
// §2  OWNERSHIP, MOVE, CLONE, COPY
// =============================================================================
fn section_02_ownership() {
    // ── Rule 1: each value has exactly ONE owner ──────────────────────────────
    let s1 = String::from("hello");
    let s2 = s1; // s1 is MOVED into s2; s1 is no longer valid
                 // println!("{}", s1);       // ← would not compile: value used after move

    // ── Clone: explicit deep copy ─────────────────────────────────────────────
    let s3 = String::from("world");
    let s4 = s3.clone(); // deep copy; both s3 and s4 are valid
    println!("[§2] clone: {} and {}", s3, s4);

    // ── Copy types: stack-only types implement Copy, so they are NOT moved ────
    let n1: i32 = 5;
    let n2 = n1; // n1 still valid — i32 is Copy
    println!("[§2] copy:  {} and {}", n1, n2);

    // Copy types: i8..i128, u8..u128, f32, f64, bool, char, (), &T, [T;N] if T:Copy

    // ── Ownership and functions ───────────────────────────────────────────────
    let s = String::from("drop me");
    takes_ownership(s); // s moved into function; dropped when fn returns
                        // println!("{}", s);        // ← would not compile

    let n = 42;
    makes_copy(n); // n copied; still valid here
    println!("[§2] n after makes_copy: {}", n);

    // ── Return values transfer ownership back ────────────────────────────────
    let s_out = gives_ownership();
    println!("[§2] got back: {}", s_out);
}

fn takes_ownership(s: String) { /* s is dropped here */
}
fn makes_copy(n: i32) {}
fn gives_ownership() -> String {
    String::from("owned")
}

// =============================================================================
// §3  BORROWING & REFERENCES
// =============================================================================
fn section_03_borrowing() {
    let s = String::from("hello");

    // ── Shared (immutable) reference: &T ─────────────────────────────────────
    let len = calculate_len(&s); // pass a reference — s is NOT moved
    println!("[§3] '{}' has length {}", s, len); // s still valid

    // RULE: many shared references are OK simultaneously
    let r1 = &s;
    let r2 = &s;
    println!("[§3] refs: {} and {}", r1, r2);

    // ── Mutable reference: &mut T ────────────────────────────────────────────
    let mut m = String::from("hello");
    change(&mut m); // lend mutably
    println!("[§3] after change: {}", m);

    // RULE: at most ONE mutable reference in scope at a time
    let r3 = &mut m;
    // let r4 = &mut m;          // ← would not compile: two &mut in same scope
    r3.push_str(" world");

    // RULE: cannot have &mut and & at the same time
    // (NLL — Non-Lexical Lifetimes — means scopes are precise)
    let mut v = vec![1, 2, 3];
    let first = &v[0]; // shared borrow
    println!("[§3] first = {}", first);
    // first reference ends here (last use), so we can mutate:
    v.push(4); // OK — first's lifetime already ended

    println!("[§3] borrowing OK");
}

fn calculate_len(s: &String) -> usize {
    s.len()
}
fn change(s: &mut String) {
    s.push_str(", world");
}

// =============================================================================
// §4  LIFETIMES
// =============================================================================
// Lifetimes tell the compiler how long references are valid.
// They are annotations — they don't change how long a value lives.

// ── Explicit lifetime annotation ─────────────────────────────────────────────
// 'a means: the returned reference lives at least as long as BOTH inputs
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// ── Lifetime in structs ───────────────────────────────────────────────────────
struct Important<'a> {
    excerpt: &'a str, // this struct cannot outlive the &str it holds
}

impl<'a> Important<'a> {
    // Lifetime elision: compiler infers 'a here
    fn level(&self) -> &str {
        self.excerpt
    }
}

// ── 'static lifetime: lives for the entire program ───────────────────────────
fn static_example() {
    let s: &'static str = "I live forever in the binary";
}

fn section_04_lifetimes() {
    let s1 = String::from("long string");
    let result;
    {
        let s2 = String::from("xy");
        result = longest(s1.as_str(), s2.as_str());
        println!("[§4] longest: '{}'", result);
    } // s2 dropped here — but result is not used after this, so OK

    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();
    let imp = Important {
        excerpt: first_sentence,
    };
    println!("[§4] excerpt: '{}'", imp.level());

    println!("[§4] lifetimes OK");
}

// =============================================================================
// §5  SLICES
// =============================================================================
fn section_05_slices() {
    // Slices are references to a contiguous sequence — no ownership, no copy.

    // ── String slices &str ────────────────────────────────────────────────────
    let s = String::from("hello world");
    let hello = &s[0..5]; // bytes 0–4
    let world = &s[6..11];
    let all = &s[..]; // entire string as slice

    // ── Array slices &[T] ─────────────────────────────────────────────────────
    let arr = [1, 2, 3, 4, 5];
    let mid = &arr[1..4]; // [2, 3, 4]
    println!("[§5] mid slice: {:?}", mid);

    let mut arr2 = [0, 1];
    let m_slice = &mut arr2[..];
    m_slice[0] = 1;

    // ── Slice in function signature ───────────────────────────────────────────
    fn sum(nums: &[i32]) -> i32 {
        nums.iter().sum()
    }
    println!("[§5] sum of arr: {}", sum(&arr));
    println!("[§5] sum of mid: {}", sum(mid));

    // ── first_word using slices ───────────────────────────────────────────────
    fn first_word(s: &str) -> &str {
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b' ' {
                return &s[0..i];
            }
        }
        &s[..]
    }
    println!("[§5] first word: '{}'", first_word("hello world"));
}

// =============================================================================
// §6  STRING TYPES
// =============================================================================
fn section_06_strings() {
    // &str  — immutable reference to UTF-8 bytes, stack-stored (or binary/heap)
    // String — owned, heap-allocated, growable UTF-8 string

    let literal: &str = "I am a string literal — &str";
    let owned: String = String::from("I am a heap String");
    let also: String = "also heap".to_string(); // come from the Display trait.
    let also2: String = "also heap".to_owned(); // come from the ToOwned trait of a borrowed value.

    // ── Conversion ────────────────────────────────────────────────────────────
    let slice: &str = &owned; // String → &str (deref coercion)
    let owned2 = slice.to_string(); // &str  → String

    // ── Building strings ──────────────────────────────────────────────────────
    let mut s = String::new();
    s.push_str("hello"); // append &str
    s.push(' '); // append char
    s.push_str("world");
    s.push_str(slice);

    let s2 = format!("{} — {}", s, literal); // format! never moves
    println!("[§6] {}", s2);

    // ── Concatenation with + ──────────────────────────────────────────────────
    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2; // s1 moved here; s2 borrowed (fn signature: add(self, &str))
                       // s1 no longer valid; s2 still valid

    // ── Iteration ─────────────────────────────────────────────────────────────
    let word = "café";
    for c in word.chars() { /* each Unicode scalar */ }
    for b in word.bytes() { /* each raw byte       */ }
    for d in word.as_bytes() { /*each &u8, reference to byte*/ }
    println!(
        "[§6] '{}' has {} chars, {} bytes",
        word,
        word.chars().count(),
        word.len()
    );

    // ── Useful methods ────────────────────────────────────────────────────────
    let s = "  Hello, Rust!  ";
    println!("[§6] trim:       '{}'", s.trim());
    println!("[§6] to_upper:   '{}'", s.trim().to_uppercase());
    println!("[§6] contains:   {}", s.contains("Rust"));
    println!("[§6] replace:    {}", s.trim().replace("Rust", "World"));
    let parts: Vec<&str> = "a,b,c".split(',').collect();
    println!("[§6] split:      {:?}", parts);
}

// =============================================================================
// §7  STRUCTS
// =============================================================================

// ── Named struct ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

impl User {
    // Associated function ("static method") — constructor by convention
    fn new(username: &str, email: &str) -> Self {
        User {
            username: username.to_string(),
            email: email.to_string(),
            sign_in_count: 0,
            active: true,
        }
    }
    // Method — first param is self / &self / &mut self
    fn greet(&self) -> String {
        format!("Hello, {}!", self.username)
    }

    fn deactivate(&mut self) {
        self.active = false;
    }
}

// ── Tuple struct ─────────────────────────────────────────────────────────────
#[derive(Debug)]
struct Point(f64, f64);

impl Point {
    fn distance_from_origin(&self) -> f64 {
        (self.0 * self.0 + self.1 * self.1).sqrt()
    }
}

#[derive(Debug)]
struct GenericPoint<T>
where
    T: Default + Add<Output = T> + Mul<Output = T> + Copy,
{
    x: T,
    y: T,
}

impl<T> GenericPoint<T>
where
    T: Default + Add<Output = T> + Mul<Output = T> + Copy,
{
    fn new(x: T, y: T) -> Self {
        GenericPoint { x, y }
    }

    fn distance_from_origin(&self) -> T {
        (self.x * self.x + self.y * self.y)
    }
}

// ── Unit struct (marker) ─────────────────────────────────────────────────────
#[derive(Debug)]
struct AlwaysEqual;

// ── Struct update syntax ─────────────────────────────────────────────────────
fn section_07_structs() {
    let mut u1 = User::new("alice", "alice@example.com");
    println!("[§7] {}", u1.greet());
    u1.deactivate();
    println!("[§7] active: {}", u1.active);

    // Struct update syntax — copy remaining fields from another instance
    let u2 = User {
        email: String::from("bob@example.com"),
        username: String::from("bob"),
        ..u1.clone() // remaining fields copied from u1
    };
    println!("[§7] u2: {:?}", u2);

    let p = Point(3.0, 4.0);
    println!("[§7] distance: {}", p.distance_from_origin()); // 5.0

    let ae = AlwaysEqual;
    println!("[§7] ae: {:?}", ae);
}

// =============================================================================
// §8  ENUMS & PATTERN MATCHING
// =============================================================================

#[derive(Debug)]
enum Shape {
    Circle(f64),                           // tuple variant
    Rectangle { width: f64, height: f64 }, // struct variant
    Triangle(f64, f64, f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle {
                width: w,
                height: h,
            } => w * h,
            Shape::Triangle(a, b, c) => {
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }
}

#[derive(Debug)]
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(String),
}

fn value_in_cents(coin: &Coin) -> u32 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println!("[§8] State quarter: {}", state);
            25
        }
    }
}

fn section_08_enums_and_matching() {
    let shapes = vec![
        Shape::Circle(5.0),
        Shape::Rectangle {
            width: 4.0,
            height: 6.0,
        },
        Shape::Triangle(3.0, 4.0, 5.0),
    ];
    for s in &shapes {
        println!("[§8] {:?} area = {:.2}", s, s.area());
    }

    let q = Coin::Quarter(String::from("Alaska"));
    println!("[§8] quarter = {} cents", value_in_cents(&q));

    // ── if let — match a single pattern, ignore the rest ─────────────────────
    let c = Shape::Circle(2.0);
    if let Shape::Circle(r) = c {
        println!("[§8] if let circle radius = {}", r);
    }

    // ── while let ────────────────────────────────────────────────────────────
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{} ", top);
    }
    println!();

    // ── Destructuring in match ────────────────────────────────────────────────
    let pair = (0, -2);
    let desc = match pair {
        (0, y) => format!("on y-axis at {}", y),
        (x, 0) => format!("on x-axis at {}", x),
        (x, y) if x == y => format!("on diagonal at {}", x),
        (x, y) => format!("at ({}, {})", x, y),
    };
    println!("[§8] {}", desc);

    // ── Range patterns ────────────────────────────────────────────────────────
    let n = 42u32;
    let grade = match n {
        0..=49 => "fail",
        50..=69 => "pass",
        70..=89 => "merit",
        _ => "distinction",
    };
    println!("[§8] grade: {}", grade);

    // ── @ bindings ────────────────────────────────────────────────────────────
    let msg = match n {
        low @ 1..=49 => format!("failing by {}", 50 - low),
        _ => String::from("passing"),
    };
    println!("[§8] {}", msg);

    // ── matches! macro ────────────────────────────────────────────────────────
    let is_nickel = matches!(Coin::Nickel, Coin::Nickel | Coin::Dime);
    println!("[§8] is_nickel: {}", is_nickel);
}

// =============================================================================
// §9  OPTION<T>
// =============================================================================
fn section_09_option() {
    // Option<T> = Some(T) | None — Rust's null safety

    fn divide(a: f64, b: f64) -> Option<f64> {
        if b == 0.0 {
            None
        } else {
            Some(a / b)
        }
    }

    // ── Unwrapping ────────────────────────────────────────────────────────────
    let result = divide(10.0, 2.0);
    println!("[§9] divide:      {:?}", result);
    println!("[§9] unwrap:      {}", result.unwrap()); // panics if None
    println!("[§9] unwrap_or:   {}", divide(10.0, 0.0).unwrap_or(0.0));
    println!(
        "[§9] unwrap_or_else: {}",
        divide(10.0, 0.0).unwrap_or_else(|| f64::INFINITY)
    );

    // ── map / and_then (Option as functor/monad) ──────────────────────────────
    let doubled = divide(10.0, 2.0).map(|v| v * 2.0);
    println!("[§9] map:         {:?}", doubled);

    let chained = divide(10.0, 2.0)
        .and_then(|v| divide(v, 2.0)) // Some(2.5)
        .map(|v| v.floor()); // Some(2.0)
    println!("[§9] and_then:    {:?}", chained);

    // ── or / or_else ──────────────────────────────────────────────────────────
    let fallback = divide(10.0, 0.0).or(Some(99.0));
    println!("[§9] or:          {:?}", fallback);
    let fallback2 = divide(1.0, 0.0).or_else(|| Some(f64::INFINITY));
    println!("[§9] or:          {:?}", fallback2);

    // ── if let / while let (see §8) ───────────────────────────────────────────
    if let Some(v) = divide(8.0, 4.0) {
        println!("[§9] if let Some: {}", v);
    }

    // ── ? in functions returning Option ──────────────────────────────────────
    fn sqrt_of_quotient(a: f64, b: f64) -> Option<f64> {
        let q = divide(a, b)?; // returns None immediately if divide returns None
        if q < 0.0 {
            None
        } else {
            Some(q.sqrt())
        }
    }
    println!(
        "[§9] sqrt_of_quotient(16,4) = {:?}",
        sqrt_of_quotient(16.0, 4.0)
    );
    println!(
        "[§9] sqrt_of_quotient(1,0)  = {:?}",
        sqrt_of_quotient(1.0, 0.0)
    );
}

// =============================================================================
// §10  RESULT<T, E> & ERROR HANDLING
// =============================================================================
use std::fmt::{Debug, Display};
use std::num::ParseIntError;

// Custom error type (idiomatic pattern)
#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    NegativeNumber(i64),
    Custom(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "parse error: {}", e),
            AppError::NegativeNumber(n) => write!(f, "negative number: {}", n),
            AppError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

// Implement std::error::Error for compatibility
impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Parse(e) => Some(e),
            _ => None,
        }
    }
}

// From<E> allows automatic conversion with ?
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

fn parse_positive(s: &str) -> Result<u64> {
    let n: i64 = s.trim().parse().map_err(AppError::Parse)?; // explicit map_err
    if n < 0 {
        Err(AppError::NegativeNumber(n))
    } else {
        Ok(n as u64)
    }
}

fn section_10_result() {
    // ── Basic usage ───────────────────────────────────────────────────────────
    println!("[§10] ok:      {:?}", parse_positive("42"));
    println!("[§10] neg:     {:?}", parse_positive("-5"));
    println!("[§10] bad:     {:?}", parse_positive("abc"));

    // ── Combinators ───────────────────────────────────────────────────────────
    let doubled: Result<u64> = parse_positive("21").map(|n| n * 2);
    println!("[§10] map:     {:?}", doubled);

    let chained = parse_positive("4").and_then(|n| parse_positive(&n.to_string()));
    println!("[§10] and_then:{:?}", chained);

    let fallback = parse_positive("bad").unwrap_or(0);
    println!("[§10] unwrap_or: {}", fallback);

    // ── Collecting Results ────────────────────────────────────────────────────
    let inputs = vec!["1", "2", "3"];
    let numbers: Result<Vec<u64>> = inputs.iter().map(|s| parse_positive(s)).collect();
    println!("[§10] collect ok: {:?}", numbers);

    let bad_inputs = vec!["1", "bad", "3"];
    let bad: Result<Vec<u64>> = bad_inputs.iter().map(|s| parse_positive(s)).collect();
    println!("[§10] collect err:{:?}", bad);

    // ── is_ok / is_err ────────────────────────────────────────────────────────
    println!("[§10] is_ok:   {}", parse_positive("5").is_ok());
    println!("[§10] is_err:  {}", parse_positive("-1").is_err());
}

// =============================================================================
// §11  THE ? OPERATOR
// =============================================================================
// ? on a Result:  if Err(e) → return Err(e.into())  else unwrap
// ? on an Option: if None   → return None           else unwrap

use std::io;

fn read_number_from_string(s: &str) -> Result<u64> {
    // Each ? either propagates the error or unwraps the Ok value
    let trimmed = s.trim();
    let n: i64 = trimmed.parse()?; // ParseIntError → AppError via From impl
    if n < 0 {
        return Err(AppError::NegativeNumber(n));
    }
    Ok(n as u64)
}

fn section_11_question_mark() {
    match read_number_from_string("  99  ") {
        Ok(n) => println!("[§11] parsed: {}", n),
        Err(e) => println!("[§11] error: {}", e),
    }
    match read_number_from_string("oops") {
        Ok(n) => println!("[§11] parsed: {}", n),
        Err(e) => println!("[§11] error: {}", e),
    }
}

// =============================================================================
// §12  TRAITS
// =============================================================================
/*
    Receiver type	Example signature	        Meaning
    self	        fn method(self)	            Consumes the value (ownership moves into the method).
    &self	        fn method(&self)	        Borrows immutably.
    &mut self	    fn method(&mut self)	    Borrows mutably.
    self: Box<Self>	fn method(self: Box<Self>)	Takes ownership via a Box pointer.
    self: Rc<Self>	fn method(self: Rc<Self>)	Takes ownership via a Rc reference‑counted pointer.
    self: Arc<Self>	fn method(self: Arc<Self>)	Takes ownership via an Arc (atomic reference count).
*/
// ── Define a trait ────────────────────────────────────────────────────────────
trait Drawable {
    fn draw(&self);
    // Default method — can be overridden, this is different when comparing with golang.
    fn bounding_box(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}

// ── Implement a trait ─────────────────────────────────────────────────────────
struct Circle {
    x: f64,
    y: f64,
    radius: f64,
}
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!(
            "[§12] Circle at ({:.0},{:.0}) r={:.0}",
            self.x, self.y, self.radius
        );
    }
    fn bounding_box(&self) -> (f64, f64) {
        (self.radius * 2.0, self.radius * 2.0)
    }
}

impl Drawable for Rect {
    fn draw(&self) {
        println!(
            "[§12] Rect at ({:.0},{:.0}) {}×{}",
            self.x, self.y, self.w, self.h
        );
    }
    fn bounding_box(&self) -> (f64, f64) {
        (self.w, self.h)
    }
}

// ── Trait bounds on functions ─────────────────────────────────────────────────
fn print_shape(shape: &impl Drawable) {
    // impl Trait syntax
    shape.draw();
}

fn print_shape_generic<T: Drawable>(shape: &T) {
    // generic with bound
    shape.draw();
    let (w, h) = shape.bounding_box();
    println!("[§12] bbox: {}×{}", w, h);
}

// Where clause — cleaner for complex bounds
fn print_both<T>(a: &T, b: &T)
where
    T: Drawable + fmt::Debug,
{
    a.draw();
    b.draw();
}

// Golang has similar thing, but it puts interfaces inside the body of trait.
// The way that rust do is C++ alike here.
// ── Trait inheritance ─────────────────────────────────────────────────────────
trait Resizable: Drawable {
    // must also impl Drawable
    fn resize(&mut self, factor: f64);
}

// ── Returning impl Trait ──────────────────────────────────────────────────────
fn make_circle() -> impl Drawable {
    Circle {
        x: 0.0,
        y: 0.0,
        radius: 5.0,
    }
}

fn section_12_traits() {
    let c = Circle {
        x: 1.0,
        y: 2.0,
        radius: 3.0,
    };
    let r = Rect {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 5.0,
    };
    print_shape(&c);
    print_shape_generic(&r);
    make_circle().draw();
}

// =============================================================================
// §13  DEFAULT TRAIT & DERIVE MACROS
// =============================================================================

// The most commonly derived traits:
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Point2D {
    x: i32,
    y: i32,
}

// Custom Default implementation
#[derive(Debug, Clone)]
struct Config {
    width: u32,
    height: u32,
    title: String,
    visible: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            width: 800,
            height: 600,
            title: String::from("App"),
            visible: true,
        }
    }
}

fn section_13_derive_and_default() {
    // Debug — {:?} and {:#?} formatting
    let p = Point2D { x: 3, y: 4 };
    println!("[§13] debug:   {:?}", p);
    println!("[§13] pretty:  {:#?}", p);

    // Clone
    let p2 = p.clone();

    // PartialEq / Eq
    println!("[§13] eq:      {}", p == p2);

    // PartialOrd / Ord — lexicographic on fields
    let mut points = vec![
        Point2D { x: 3, y: 4 },
        Point2D { x: 1, y: 9 },
        Point2D { x: 1, y: 2 },
    ];
    points.sort();
    println!("[§13] sorted:  {:?}", points);

    // Hash — enables use as HashMap key
    let mut map: HashMap<Point2D, &str> = HashMap::new();
    map.insert(Point2D { x: 0, y: 0 }, "origin");

    // Default
    let cfg = Config::default();
    println!("[§13] default: {:?}", cfg);

    // Struct update from default (common pattern)
    let custom = Config {
        title: String::from("Custom"),
        ..Config::default()
    };
    println!("[§13] custom:  {:?}", custom);
}

// =============================================================================
// §14  GENERICS
// =============================================================================

// ── Generic struct ────────────────────────────────────────────────────────────
#[derive(Debug)]
struct Stack<T>
where
    T: Debug,
{
    items: Vec<T>,
}

impl<T> Stack<T>
where
    T: Debug,
{
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, item: T) {
        self.items.push(item);
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}

// ── Generic function with multiple bounds ─────────────────────────────────────
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn larger<'a, T: PartialOrd>(a: &'a T, b: &'a T) -> &'a T {
    if a > b {
        a
    } else {
        b
    }
}

fn littlest<T: PartialOrd>(list: &[T]) -> &T {
    let mut littlest = &list[0];
    for item in list {
        if item < littlest {
            littlest = item;
        }
    }
    littlest
}

// ── Generic enum (like Option/Result) ────────────────────────────────────────
#[derive(Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

// ── Const generics (Rust 1.51+) ───────────────────────────────────────────────
#[derive(Debug)]
struct Matrix<T, const N: usize, const M: usize> {
    data: [[T; M]; N],
}

fn section_14_generics() {
    let mut stack: Stack<i32> = Stack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);
    println!("[§14] peek: {:?}", stack.peek());
    println!("[§14] pop:  {:?}", stack.pop());
    println!("[§14] len:  {}", stack.len());

    let nums = vec![34, 50, 25, 100, 65];
    let chars = vec!['y', 'm', 'a', 'q'];
    println!("[§14] largest int:  {}", largest(&nums));
    println!("[§14] largest char: {}", largest(&chars));

    let either: Either<i32, &str> = Either::Left(42);
    println!("[§14] either: {:?}", either);
}

// =============================================================================
// §15  TRAIT OBJECTS  (dyn Trait — runtime polymorphism)
// =============================================================================
// Box<dyn Trait> stores a fat pointer: (ptr to data, ptr to vtable)
// Use when: you don't know the concrete type at compile time

fn section_15_trait_objects() {
    // Heterogeneous collection of Drawables
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
        }),
        Box::new(Rect {
            x: 1.0,
            y: 1.0,
            w: 3.0,
            h: 2.0,
        }),
        Box::new(Circle {
            x: 5.0,
            y: 5.0,
            radius: 2.0,
        }),
    ];

    for shape in &shapes {
        shape.draw(); // dynamic dispatch — vtable lookup at runtime
    }

    // ── impl Trait vs dyn Trait ───────────────────────────────────────────────
    // impl Trait → static dispatch (monomorphized), faster, but single type
    // dyn Trait  → dynamic dispatch (vtable), flexible, slight overhead

    // Returning dyn Trait from function
    fn pick_shape(use_circle: bool) -> Box<dyn Drawable> {
        if use_circle {
            Box::new(Circle {
                x: 0.0,
                y: 0.0,
                radius: 1.0,
            })
        } else {
            Box::new(Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            })
        }
    }
    pick_shape(true).draw();
    pick_shape(false).draw();

    println!("[§15] trait objects OK");
}

// =============================================================================
// §16  ASSOCIATED TYPES
// =============================================================================
// Associated types let a trait say "I will produce some type T"
// without making T a type parameter of the trait itself.

trait Container {
    type Item; // associated type
    fn first(&self) -> Option<&Self::Item>;
    fn last(&self) -> Option<&Self::Item>;
    fn count(&self) -> usize;
}

struct Bag<T>(Vec<T>);

impl<T> Container for Bag<T> {
    type Item = T;
    fn first(&self) -> Option<&T> {
        self.0.first()
    }
    fn last(&self) -> Option<&T> {
        self.0.last()
    }
    fn count(&self) -> usize {
        self.0.len()
    }
}

fn section_16_associated_types() {
    let bag = Bag(vec![10, 20, 30]);
    println!(
        "[§16] first: {:?}, last: {:?}, count: {}",
        bag.first(),
        bag.last(),
        bag.count()
    );
    // Compare with generics: Container<Item=T> would require spelling out T every time.
    // Associated types give one-to-one relationship: one Bag<T> → one Item type.
    let bag2 = Bag(vec![0.0, 0.0, 0.2]);
    let bag3 = Bag(vec!["hi", "hello", "ciao"]);
}

// =============================================================================
// §17  CLOSURES
// =============================================================================
fn apply_twice<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(f(x))
}

fn fn_trait() {
    let multiplier = 2;
    let caller = |x: i32| x*multiplier;
    let v = 1i32;
    let applied = apply_twice(caller, v);
    println!("[§17] applied: {}", applied);
    assert_eq!(apply_twice(caller, 5), 20);
    assert_eq!(caller(10), 20);
}

// mut f: F tells the f closure can modify the captures.
fn for_each_index<F>(mut f: F, count: i32)
where F: FnMut(i32), {
    for i in 0..count {
        f(i);
    }
}

fn fnmut_trait() {
    let mut sum = 0; // captured and modified by the closure.
    let mut accumulator = |x: i32| sum += x; // accumulator should be mutable asked by the trait.
    for_each_index(accumulator, 3);
    assert_eq!(sum, 0 + 1 + 2);
}

fn run_once<F> (f: F)
where F: FnOnce() -> String {
    let s = f(); // the closure function should return a String value.
    println!("{}", s);
}

fn fnonce_trait() {
    let s = String::from("Hello, FnOnce!");
    let consumer = move || s; // moves s into closure, and return it.
    run_once(consumer); // consumer moved here, can only be called once
    // consumer is no long available from here, as it was consumed by run_once();
}

fn closures_traits() {
    // trait definitions in STD:
    /*
    pub trait FnOnce<Args> {
        type Output;
        // Takes ownership of self – can only be called once.
        // Consumes the closure, so it can move captured variables out.
        fn call_once(self, args: Args) -> Self::Output;
    }

    pub trait FnMut<Args>: FnOnce<Args> {
        // Takes &mut self – can be called multiple times, may mutate state.
        // Requires mutable access to the closure, so it can mutate captured variables.
        fn call_mut(&mut self, args: Args) -> Self::Output;
    }

    pub trait Fn<Args>: FnMut<Args> {
        // Takes &self – shared, non-mutating calls.
        // It cannot mutate the captured variables and can be called multiple times, even
        // concurrently if Sync.
        fn call(&self, args: Args) -> Self::Output;
    }
    */

    fn_trait();
    fnmut_trait();
    fnonce_trait();
}

fn section_17_closures() {
    // Closure syntax: |args| body  or  |args| { multi-line body }
    let square = |x: i32| x * x;
    println!("[§17] square(5): {}", square(5));

    // ── Capturing environment ─────────────────────────────────────────────────
    let offset = 10;
    let add_offset = |x| x + offset; // borrows offset
    println!("[§17] add_offset(5): {}", add_offset(5));

    // move closure — takes ownership of captured vars (needed for threads)
    let greeting = String::from("Hello");
    let greeter = move |name: &str| format!("{}, {}!", greeting, name);
    // `greeting` moved into closure; no longer accessible here
    println!("[§17] greeter: {}", greeter("Alice"));

    // ── Fn traits ─────────────────────────────────────────────────────────────
    // Fn     — can be called many times, borrows captured vars
    // FnMut  — can be called many times, mutably borrows captured vars
    // FnOnce — can be called ONCE (e.g. moves a value out)

    fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
        f(x)
    }
    fn apply_mut<F: FnMut() -> i32>(mut f: F) -> i32 {
        f()
    }
    fn apply_once<F: FnOnce() -> String>(f: F) -> String {
        f()
    }

    apply_mut(|| 3);
    println!("[§17] apply: {}", apply(|x| x * 3, 7));

    let mut count = 0;
    let mut counter = || {
        count += 1;
        count
    };
    println!("[§17] counter: {} {}", counter(), counter());

    // consumer is a FnOnce, it is moved into apply_once, thus it can be used only once.
    let s = String::from("consumed");
    let consumer = move || s; // FnOnce — moves s
    println!("[§17] once: {}", apply_once(consumer)); // consumer moved here, no longer available.

    // ── Closures as return values ─────────────────────────────────────────────
    fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
        move |x| x + n
    }
    let add5 = make_adder(5);
    println!("[§17] add5(10): {}", add5(10));

    // ── Storing closures in structs ───────────────────────────────────────────
    struct Memoize<F: Fn(i32) -> i32> {
        func: F,
        cache: Option<i32>,
    }
    impl<F: Fn(i32) -> i32> Memoize<F> {
        fn call(&mut self, arg: i32) -> i32 {
            match self.cache {
                Some(v) => v,
                None => {
                    let v = (self.func)(arg);
                    self.cache = Some(v);
                    v
                }
            }
        }
    }
}

// =============================================================================
// §18  ITERATORS & ITERATOR ADAPTERS
// =============================================================================
fn section_18_iterators() {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // for loop consumes iterators.
    for item in v.iter() {
        println!("{}", item)
    }

    // same but with index and &item wrapped in a tuple.
    for item in v.iter().enumerate() {
        println!("{}", item.1)
    }

    let mut v2 = vec![1, 2, 3];
    for item in v2.iter_mut() {
        *item += 1;
    }
    println!("{:?}", v2);

    // ── Creating iterators ────────────────────────────────────────────────────
    // iter()       → yields &T        (borrows)
    // iter_mut()   → yields &mut T    (mutable borrow)
    // into_iter()  → yields T         (consumes the collection)

    // ── Adapters (lazy — nothing runs until consumed) ─────────────────────────
    let result: Vec<i32> = v
        .iter()
        .filter(|&&x| x % 2 == 0) // keep evens: [2,4,6,8,10]
        .map(|&x| x * x) // square them: [4,16,36,64,100]
        .take(3) // first 3:   [4,16,36]
        .collect();
    println!("[§18] filter+map+take: {:?}", result);

    // ── Consumers ────────────────────────────────────────────────────────────
    let sum: i32 = v.iter().sum();
    let product: i32 = v.iter().product();
    let max = v.iter().max().unwrap();
    let count = v.iter().filter(|&&x| x > 5).count();
    println!(
        "[§18] sum={} product={} max={} count_gt5={}",
        sum, product, max, count
    );

    // ── fold / reduce ────────────────────────────────────────────────────────
    let factorial: u64 = (1..=10).fold(1, |acc, x| acc * x);
    println!("[§18] 10! = {}", factorial);

    // ── flat_map ──────────────────────────────────────────────────────────────
    let words = vec!["hello world", "foo bar"];
    let chars: Vec<&str> = words.iter().flat_map(|s| s.split_whitespace()).collect();
    println!("[§18] flat_map: {:?}", chars);

    // ── zip ───────────────────────────────────────────────────────────────────
    let names = vec!["Alice", "Bob", "Carol"];
    let scores = vec![95, 87, 92];
    let paired: Vec<(&&str, &i32)> = names.iter().zip(scores.iter()).collect();
    println!("[§18] zip: {:?}", paired);

    // ── enumerate ─────────────────────────────────────────────────────────────
    for (i, name) in names.iter().enumerate() {
        print!("[{}]={} ", i, name);
    }
    println!();

    // ── chain ─────────────────────────────────────────────────────────────────
    let a = [1, 2];
    let b = [3, 4];
    let chained: Vec<_> = a.iter().chain(b.iter()).collect();
    println!("[§18] chain: {:?}", chained);

    // ── any / all / find / position ───────────────────────────────────────────
    println!("[§18] any>5:  {}", v.iter().any(|&x| x > 5));
    println!("[§18] all>0:  {}", v.iter().all(|&x| x > 0));
    println!("[§18] find>5: {:?}", v.iter().find(|&&x| x > 5));
    println!("[§18] pos>5:  {:?}", v.iter().position(|&x| x > 5));

    // ── windows / chunks ──────────────────────────────────────────────────────
    let wins: Vec<_> = v[..5].windows(3).collect();
    println!("[§18] windows(3): {:?}", wins);

    // ── Ranges as iterators ───────────────────────────────────────────────────
    let squares: Vec<u32> = (1..=5).map(|x| x * x).collect();
    println!("[§18] squares: {:?}", squares);

    // ── peekable ─────────────────────────────────────────────────────────────
    let mut iter = v.iter().peekable();
    println!("[§18] peek: {:?}", iter.peek()); // forsee the future item.
    println!("[§18] next: {:?}", iter.next()); // iterate to the future item.

    // ── scan (stateful map) ───────────────────────────────────────────────────
    let running_sum: Vec<i32> = (1..=5)
        .scan(0, |acc, x| {
            *acc += x;
            Some(*acc)
        })
        .collect();
    println!("[§18] running sum: {:?}", running_sum);
}

// =============================================================================
// §19  COLLECTIONS
// =============================================================================
fn section_19_collections() {
    // ── Vec<T> ────────────────────────────────────────────────────────────────
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    v.insert(1, 99); // insert 99 at index 1
    v.remove(2); // remove index 2
    v.retain(|&x| x > 1); // keep elements > 1
    v.sort();
    v.dedup(); // remove consecutive duplicates
    v.extend([10, 11, 12]);
    println!("[§19] Vec: {:?}, len={}", v, v.len());
    println!("[§19] contains(99): {}", v.contains(&99));
    // Vec with capacity
    let mut cap_v: Vec<i32> = Vec::with_capacity(100);
    println!("[§19] capacity: {}", cap_v.capacity()); // no realloc up to 100 items

    // ── HashMap<K, V> ────────────────────────────────────────────────────────
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert(String::from("Alice"), 95);
    scores.insert(String::from("Bob"), 87);

    // entry API — insert-or-update
    scores.entry(String::from("Alice")).and_modify(|s| *s += 5);

    /*
    // or_insert – even if key "Alice" already exists, compute_expensive_score("Alice") runs
    scores.entry("Alice").or_insert(compute_expensive_score("Alice"));
    // or_insert_with – compute_expensive_score is only called if "Bob" is absent
    scores.entry("Bob").or_insert_with(|| compute_expensive_score("Bob"));
    */
    scores.entry(String::from("Carol")).or_insert(70); // insert only if absent
    scores.entry(String::from("Dave")).or_insert_with(|| 80); // lazy insert, the closure

    println!("[§19] scores: {:?}", scores);
    println!("[§19] Alice:  {:?}", scores.get("Alice"));

    // Iterating
    for (name, score) in &scores {
        // println!("  {} → {}", name, score);
    }

    // HashMap new
    let mut my_hash_map: HashMap<String, i32> = HashMap::new();
    my_hash_map.insert("key1".to_string(), 1);
    my_hash_map.remove("key1");

    // Collect into HashMap
    let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
    let map: HashMap<_, _> = pairs.into_iter().collect();
    println!("[§19] from pairs: {:?}", map);

    // ── HashSet<T> ────────────────────────────────────────────────────────────
    let mut set: HashSet<i32> = HashSet::from([1, 2, 3, 4, 5]);
    set.insert(6);
    set.remove(&1);
    let other = HashSet::from([3, 4, 5, 6, 7]);
    let union: HashSet<_> = set.union(&other).collect();
    let intersection: HashSet<_> = set.intersection(&other).collect();
    let difference: HashSet<_> = set.difference(&other).collect();
    println!("[§19] set: {:?}", set);
    println!("[§19] union: {:?}", union);
    println!("[§19] intersection: {:?}", intersection);

    // ── BTreeMap<K, V> — sorted map ──────────────────────────────────────────
    let mut btree: BTreeMap<&str, i32> = BTreeMap::new();
    btree.insert("cherry", 3);
    btree.insert("apple", 1);
    btree.insert("banana", 2);
    btree.remove("apple");

    // Iteration is in sorted key order
    for (k, v) in &btree {
        print!("{}:{} ", k, v);
    }
    println!();

    let mut iter = btree.iter().rev();
    while let Some(v) = iter.next() {
        print!("key:{}, value:{}", v.0, v.1);
    }

    // ── VecDeque<T> — double-ended queue ────────────────────────────────────
    let mut deque: VecDeque<i32> = VecDeque::new();
    deque.push_back(1);
    deque.push_back(2);
    deque.push_front(0);
    println!("[§19] deque: {:?}", deque);
    println!("[§19] pop_front: {:?}", deque.pop_front());
}

// =============================================================================
// §20  SMART POINTERS
// =============================================================================
fn section_20_smart_pointers() {
    // ── Box<T>: heap allocation, single owner ─────────────────────────────────
    let b: Box<i32> = Box::new(42);
    println!("[§20] Box: *b = {}", *b);

    // Recursive type impossible without Box (infinite size at compile time)
    #[derive(Debug)]
    enum List {
        Cons(i32, Box<List>),
        Nil,
    }
    let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
    println!("[§20] List: {:?}", list);

    // ── Rc<T>: reference-counted shared ownership (single-threaded) ───────────
    // the data inside a Rc is usually immutable, it provides read only shares. However,
    // with interior mutability, for example, the inside data are wrapped by lock-free
    // structures or synchronization primitives(Mutex<>, RwLock<>), Atomic types, they
    // are still mutable via their &self receivers without a mut &self receiver. Keep in mind,
    // the interior mutability is a design pattern, it is not a language feature.
    let a = Rc::new(String::from("shared"));
    let b_rc = Rc::clone(&a); // bump ref count to 2
    let mut c_rc = Rc::clone(&a); // bump ref count to 3

    println!("[§20] Rc strong count: {}", Rc::strong_count(&a)); // 3
    println!("[§20] Rc value: {}", a);
    drop(b_rc);

    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    let a = Rc::clone(&shared);
    let b = Rc::clone(&shared);
    a.borrow_mut().push(4); // mutate through `a`
    println!("{:?}", b.borrow()); // [1, 2, 3, 4] — `b` sees the change

    // ── Weak<T>: non-owning reference (breaks cycles) ─────────────────────────
    let strong = Rc::new(42);
    let weak: std::rc::Weak<i32> = Rc::downgrade(&strong);
    println!("[§20] Weak upgrade: {:?}", weak.upgrade()); // Some(42)
    drop(strong);
    println!("[§20] Weak upgrade after drop: {:?}", weak.upgrade()); // None

    // ── Arc<T>: atomic Rc — thread-safe ───────────────────────────────────────
    let arc = Arc::new(vec![1, 2, 3]);
    let arc2 = Arc::clone(&arc);
    println!("[§20] Arc: {:?}", arc2);

    // ── Cow<T>: clone-on-write ────────────────────────────────────────────────
    // Returning data in Cow:
    use std::borrow::Cow;
    fn ensure_no_spaces<'a>(s: &'a str) -> Cow<'a, str> {
        if s.contains(' ') {
            Cow::Owned(s.replace(' ', "_"))
        }
        // allocates
        else {
            Cow::Borrowed(s)
        } // zero-copy
    }
    println!("[§20] Cow no-op: {}", ensure_no_spaces("hello"));
    println!("[§20] Cow owned: {}", ensure_no_spaces("hello world"));

    // Use Cow data as input:
    fn process(data: Cow<str>) -> String {
        let processed = data.replace("foo", "bar"); // this implicitly calls into_owned if needed
        processed
    }
    let r = process(Cow::Borrowed("foo bar")); // no extra allocation
    let s = process(Cow::Owned(String::from("foo bar")));
}

// =============================================================================
// §21  INTERIOR MUTABILITY
// =============================================================================
fn section_21_interior_mutability() {
    // ── Cell<T>: Copy types, no borrow overhead ───────────────────────────────
    let c = Cell::new(5);
    let r = &c;
    c.set(11);
    r.set(12); // mutate through shared ref
    println!("[§21] Cell: {}", c.get());

    // ── RefCell<T>: any type, runtime borrow checking ─────────────────────────
    let rc = RefCell::new(vec![1, 2, 3]);
    rc.borrow_mut().push(4); // dynamic borrow check — panics on violation
    println!("[§21] RefCell: {:?}", rc.borrow());

    // ── Rc<RefCell<T>>: the shared-mutable-state pattern (single-threaded) ────
    let shared = Rc::new(RefCell::new(0i32));
    let c1 = Rc::clone(&shared);
    // let c2 = shared.clone(); same as below.
    let c2 = Rc::clone(&shared);
    *c1.borrow_mut() += 10;
    *c2.borrow_mut() += 20;
    println!("[§21] Rc<RefCell>: {}", shared.borrow()); // 30

    // ── Arc<Mutex<T>>: the shared-mutable-state pattern (multi-threaded) ─────
    // (See §22 for the threading example)
    let protected = Arc::new(Mutex::new(0i32));
    {
        let mut guard = protected.lock().unwrap(); // blocks until free
        *guard += 100;
    } // MutexGuard dropped here → unlocked automatically
    println!("[§21] Arc<Mutex>: {}", *protected.lock().unwrap());
}

// =============================================================================
// STD::SYNC::Channel
// =============================================================================

// multi-producer single-consumer channel, the standard, safe, "good enough" channel:
// * Muti-producer, single-consumer. Only one receiver is allowed.
// * Unbounded by default, but can be made synchronous/bounded with sync_channel.
// * Concurrency model, internally a linked list of nodes protected by a mutex + Condvar.
//     This makes it relatively slow under contention and subject to kernel scheduling noise.
// * Heap Allocation, allocation per message, unless you use a bounded sync channel with fixed
//     capacity, which still uses a similar queue.
// * No batching, each send/recv is a single operation with lock acquisition.
use std::sync::mpsc;
use std::thread;

fn std_mpsc_channel() {
    // ── mpsc channel: send data between threads ────────────────────────────────
    let (tx, rx) = mpsc::channel::<String>();

    // Multiple producers
    let tx2 = tx.clone();
    let h1 = thread::spawn(move || {
        tx.send(String::from("ping")).unwrap();
    });
    let h2 = thread::spawn(move || {
        tx2.send(String::from("pong")).unwrap();
    });
    h1.join().unwrap();
    h2.join().unwrap();

    //drop(tx2);   // all senders dropped → rx.recv() will return Err after queue empty

    for received in rx {
        println!("[§22] channel received: {}", received);
    }
}

// =============================================================================
// CROSSBEAM::Channels
// =============================================================================
// *Multiple flavours, unbounded, bounded(cap), zero (rendezvous), and specialized spsc/mpsc/mpmc
// *Lock-free algorithms (mostly), using atomic operations and careful memory ordering, wait-freedom
//     in some paths.
// *Much higher throughput and lower latency than std::mpsc, especially under contention.
// *Select-like operations supported via select! macro, allowing waiting on multiple channels simultaneously.
// *Allocation, unbounded channels still allocate per message; bounded channels pre-allocate a buffer,
//     avoiding per-message allocation.
// *No batching, still one send/recv per message, but much faster than std::mpsc due to lock-free design.
fn crossbeam_channels_bounded_spsc() {
    use crossbeam_channel::bounded;

    let (tx, rx) = bounded::<i32>(4); // capacity 4

    // Producer thread
    thread::spawn(move || {
        for i in 0..10 {
            tx.send(i).unwrap();
        }
    });

    // Consumer (main thread)
    for msg in rx {
        println!("Received: {}", msg);
    }
}

fn crossbeam_channels_unbounded_mpsc() {
    use crossbeam_channel::unbounded;
    use std::thread;

    let (tx, rx) = unbounded::<String>();

    // Clone the sender for multiple producers
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            for i in 0..5 {
                tx.send(format!("P{}-{}", id, i)).unwrap();
            }
        });
    }
    drop(tx); // original sender dropped so receiver can terminate

    for msg in rx {
        println!("Got: {}", msg);
    }
}

fn crossbeam_bounded_queue_spmc() {
    use crossbeam_queue::ArrayQueue;
    use std::sync::Arc;
    use std::thread;

    let queue = Arc::new(ArrayQueue::new(16));

    // Single producer
    let q_prod = Arc::clone(&queue);
    thread::spawn(move || {
        for i in 0..20 {
            while q_prod.push(i).is_err() {} // spin until slot free
        }
    });

    // Multiple consumers
    for id in 0..3 {
        let q_cons = Arc::clone(&queue);
        thread::spawn(move || {
            loop {
                match q_cons.pop() {
                    Some(val) => println!("C{} got {}", id, val),
                    None => break, // queue empty and producer finished? use a termination flag
                }
            }
        });
    }
}

// =============================================================================
// LMAX Disruptor::Channels
// =============================================================================
// * Allocation, a pre-allocated ring buffer with fixed-sized slots. Msg are written into those
//    slots, no allocation after initialization.
// * Zero-copy publishing, the producer claims a slot, writes into it, then publishes the sequence
//    number. Consumers read committed slots.
// * Batching, both producers and consumers can process multiple events at once, dramatically
//    reducing overhead.
// * Extremely low latency and very high throughput. Designed for the LMAX exchange to handle
//    millions of messages per second with sub-millisecond latency.
// * Complex dependency graph, you can set up multiple consumers that depend on each other using
//    a barrier.
// * Producer types, single-producer(No atomic CAS needed, just a store) or multi-producer (with
//    atomic CAS). Multiple consumers can be parallel or dependent.
// * API complexity, requires understanding Sequence, RingBuffer, EventHandler, Barrier. Not a simple
//    send/recv.
fn lmax_disruptor_channel_spsc() {
    // todo: write an example in LMAX disruptor pattern.
}

fn section_22_concurrency() {
    // ── Basic thread ──────────────────────────────────────────────────────────
    let handle = thread::spawn(|| {
        println!("[§22] hello from spawned thread");
    });
    handle.join().unwrap(); // wait for thread to finish

    // ── move closure captures by value ────────────────────────────────────────
    let msg = String::from("moved");
    let h = thread::spawn(move || println!("[§22] got: {}", msg));
    h.join().unwrap();

    // ── Arc<Mutex<T>>: shared mutable state across threads ────────────────────
    // Option of RwLock for read heavy scenarios.
    // let counter = Arc::new(RwLock::new(0i32));
    let counter = Arc::new(Mutex::new(0i32));
    let mut handles = vec![];
    for _ in 0..5 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut n = c.lock().unwrap();
            *n += 1;
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("[§22] Mutex counter: {}", *counter.lock().unwrap()); // 5

    std_mpsc_channel();
    crossbeam_bounded_queue_spmc();
    crossbeam_channels_unbounded_mpsc();
    crossbeam_channels_bounded_spsc();
}

// =============================================================================
// §23  ASYNC / AWAIT  (conceptual — compile-check only, no runtime here)
// =============================================================================
// async fn compiles to a state machine that implements Future.
// The runtime (tokio / async-std) drives it by calling .poll() repeatedly.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Manually implementing Future (what async fn does internally)
struct ReadyFuture(i32);
impl Future for ReadyFuture {
    type Output = i32;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<i32> {
        Poll::Ready(self.0) // immediately ready
    }
}

// async fn desugars to:  fn foo() -> impl Future<Output=i32>
async fn async_add(a: i32, b: i32) -> i32 {
    a + b
}
async fn async_pipeline() -> String {
    let sum = async_add(2, 3).await; // .await suspends until future is ready
    format!("sum = {}", sum)
}

// ── Returning Pin<Box<dyn Future>> from trait methods ─────────────────────────
trait AsyncGreeter {
    fn greet<'a>(&'a self, name: &'a str) -> Pin<Box<dyn Future<Output = String> + 'a>>;
}

struct Bot;
impl AsyncGreeter for Bot {
    fn greet<'a>(&'a self, name: &'a str) -> Pin<Box<dyn Future<Output = String> + 'a>> {
        Box::pin(async move { format!("Hello, {}!", name) })
    }
}

// Note: to actually run async code, add tokio to Cargo.toml:
//   [dependencies]
//   tokio = { version = "1", features = ["full"] }
//
// Then:
//   #[tokio::main]
//   async fn main() {
//       let result = async_pipeline().await;
//       println!("{}", result);
//   }

// =============================================================================
// §24  MACROS  (declarative)
// =============================================================================

// ── macro_rules! — match-based text substitution ──────────────────────────────
macro_rules! say_hello {
    () => { println!("Hello, world!"); };
    ($name:expr) => { println!("Hello, {}!", $name); };
    ($($name:expr),+) => {          // variadic: one or more names
        $( println!("Hello, {}!", $name); )+
    };
}

// ── A vec!-like macro ─────────────────────────────────────────────────────────
macro_rules! my_vec {
    ($($x:expr),* $(,)?) => {{
        let mut v = Vec::new();
        $( v.push($x); )*
        v
    }};
}

// ── map! convenience constructor ──────────────────────────────────────────────
macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m = HashMap::new();
        $( m.insert($k, $v); )*
        m
    }};
}

// ── assert_approx_eq for floats ───────────────────────────────────────────────
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr, $eps:expr) => {{
        let diff = ($a - $b).abs();
        assert!(diff < $eps, "{} ≈ {} failed (diff={})", $a, $b, diff);
    }};
}

fn section_24_macros() {
    say_hello!();
    say_hello!("Alice");
    say_hello!("Alice", "Bob", "Carol");

    let v = my_vec![1, 2, 3, 4];
    println!("[§24] my_vec: {:?}", v);

    let v2: Vec<&str> = my_vec![];
    println!("[§24] empty my_vec: {:?}", v2);

    let m = map! { "a" => 1, "b" => 2 };
    println!("[§24] map: {:?}", m);

    let m2: HashMap<&str, i32> = map! {};
    println!("[§24] empty map: {:?}", m2);

    assert_approx_eq!(3.14159f64, std::f64::consts::PI, 0.001);
    println!("[§24] macros OK");
}

// =============================================================================
// §25  MODULES & VISIBILITY
// =============================================================================
// In real projects these would be separate files / directories.
// Visibility modifiers:
//   (none)     — private to the current module
//   pub        — public everywhere
//   pub(super) — public to the parent module
//   pub(crate) — public within the crate

mod geometry {
    pub struct Rectangle {
        pub w: f64,
        pub h: f64,
    }
    impl Rectangle {
        pub fn new(w: f64, h: f64) -> Self {
            Rectangle { w, h }
        }
        pub fn area(&self) -> f64 {
            self.w * self.h
        }
        fn secret(&self) {} // private — only accessible inside this module
    }

    pub mod advanced {
        pub fn perimeter(r: &super::Rectangle) -> f64 {
            2.0 * (r.w + r.h)
        }
    }
}

// use to bring into scope
use geometry::advanced::perimeter;
use geometry::Rectangle;

// =============================================================================
// §26  TYPE ALIASES & NEWTYPE PATTERN
// =============================================================================

// ── Type alias: just a rename, no new type semantics ──────────────────────────
type Kilometers = i32;
type Result<T> = std::result::Result<T, AppError>; // scoped alias
type Thunk = Box<dyn Fn() -> String>; // name a complex type

// ── Newtype: a struct wrapper that IS a distinct type ─────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Feet(f64);

impl Meters {
    fn to_feet(self) -> Feet {
        Feet(self.0 * 3.28084)
    }
}

impl fmt::Display for Meters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}m", self.0)
    }
}

fn section_26_type_aliases_newtype() {
    let km: Kilometers = 5; // just i32 under the hood
    let m = Meters(1.8);
    let ft = m.to_feet();
    println!("[§26] {} = {:?}", m, ft);

    // Newtypes prevent accidental mixing of units at compile time:
    // let bad: Meters = ft;   // ← would not compile — different types
}

// =============================================================================
// §27  OPERATOR OVERLOADING
// =============================================================================
use std::ops::{Add, Index, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }
    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn dot(&self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}
impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

fn section_27_operator_overloading() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);
    println!("[§27] a+b = {}", a + b);
    println!("[§27] a-b = {}", a - b);
    println!("[§27] a*2 = {}", a * 2.0);
    println!("[§27] -a  = {}", -a);
    println!("[§27] |b| = {:.1}", b.length());
    println!("[§27] a·b = {:.1}", a.dot(b));
}

// =============================================================================
// §28  THE BUILDER PATTERN
// =============================================================================
#[derive(Debug)]
struct Request {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    timeout: u32,
}

#[derive(Default)]
struct RequestBuilder {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    timeout: u32,
}

impl RequestBuilder {
    fn new(url: &str) -> Self {
        RequestBuilder {
            url: url.to_string(),
            method: String::from("GET"),
            timeout: 30,
            ..Default::default()
        }
    }
    // note: every the "self" receiver consumes the object by returning it.
    fn method(mut self, m: &str) -> Self {
        self.method = m.to_string();
        self
    }
    fn header(mut self, k: &str, v: &str) -> Self {
        self.headers.insert(k.to_string(), v.to_string());
        self
    }
    fn body(mut self, b: &str) -> Self {
        self.body = Some(b.to_string());
        self
    }
    fn timeout(mut self, secs: u32) -> Self {
        self.timeout = secs;
        self
    }
    fn build(self) -> Request {
        Request {
            url: self.url,
            method: self.method,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout,
        }
    }
}

fn section_28_builder_pattern() {
    let req = RequestBuilder::new("https://api.example.com/data")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer token123")
        .body(r#"{"key":"value"}"#)
        .timeout(60)
        .build();
    println!("[§28] {:?}", req);
}

// =============================================================================
// §29  CUSTOM ITERATOR
// =============================================================================

struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let next = self.a;
        self.a = self.b;
        self.b = next + self.b;
        Some(next) // infinite — return None when you want to stop
    }
}

fn section_29_custom_iterator() {
    let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("[§29] Fibonacci: {:?}", fibs);

    // Because we implement Iterator, we get ALL iterator adapters for free:
    let sum: u64 = Fibonacci::new().take(10).sum();
    println!("[§29] sum of first 10: {}", sum);

    let first_over_100 = Fibonacci::new().find(|&x| x > 100);
    println!("[§29] first > 100: {:?}", first_over_100);
}

// =============================================================================
// §30  RAW POINTERS & UNSAFE
// =============================================================================
fn section_30_unsafe_raw_pointers() {
    // Raw pointers: *const T (immutable) and *mut T (mutable)
    // Creating them is safe. Dereferencing them requires unsafe.

    let mut x = 42i32;
    let r1: *const i32 = &x; // immutable raw pointer
    let r2: *mut i32 = &mut x; // mutable raw pointer

    unsafe {
        println!("[§30] *r1 = {}", *r1); // dereference
        *r2 = 100;
        println!("[§30] *r2 after write = {}", *r2);
    }

    // ── Null pointer check ────────────────────────────────────────────────────
    let null: *const i32 = std::ptr::null();
    println!("[§30] is_null: {}", null.is_null());

    // ── Box::into_raw / Box::from_raw (manual heap management) ────────────────
    let b = Box::new(String::from("manual heap"));
    let raw: *mut String = Box::into_raw(b); // Box leaked — no drop
    unsafe {
        println!("[§30] raw ptr: {}", *raw);
        let _ = Box::from_raw(raw); // re-box → dropped here
    }

    // ── ptr::copy_nonoverlapping (like memcpy) ────────────────────────────────
    let src = [1i32, 2, 3, 4, 5];
    let mut dst = [0i32; 5];
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
    println!("[§30] memcpy result: {:?}", dst);

    // ── unsafe fn ────────────────────────────────────────────────────────────
    unsafe fn dangerous() -> i32 {
        42
    }
    let v = unsafe { dangerous() };
    println!("[§30] unsafe fn: {}", v);

    // ── unsafe trait ─────────────────────────────────────────────────────────
    // unsafe trait Sendable { }
    // unsafe impl Sendable for MyType { }  // you are asserting safety

    // ── extern "C": calling C functions ──────────────────────────────────────
    unsafe extern "C" {
        fn abs(x: i32) -> i32;
    }
    let abs_val = unsafe { abs(-42) };
    println!("[§30] C abs(-42) = {}", abs_val);
}

// =============================================================================
// §31  ATTRIBUTES & LINT CONTROL
// =============================================================================
#[allow(dead_code)]
fn section_31_attributes() {
    // ── Code generation ───────────────────────────────────────────────────────
    // #[inline]           — suggest inlining
    // #[inline(always)]   — force inlining
    // #[inline(never)]    — prevent inlining
    // #[cold]             — hint: rarely called (branch prediction)
    // #[no_mangle]        — disable name mangling (for FFI), allowing rust api from C etc....

    // ── Conditional compilation ───────────────────────────────────────────────
    // #[cfg(target_os = "linux")]
    // #[cfg(debug_assertions)]
    // #[cfg(feature = "serde")]

    // ── Lints ─────────────────────────────────────────────────────────────────
    // #[allow(unused_variables)]
    // #[warn(missing_docs)]
    // #[deny(unsafe_code)]
    // #[forbid(unsafe_code)]       — cannot be overridden

    // ── Derive ────────────────────────────────────────────────────────────────
    // #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
    // #[derive(Serialize, Deserialize)]  — serde crate

    // ── Documentation ─────────────────────────────────────────────────────────
    // /// Triple-slash doc comment — appears in `cargo doc`
    // //! Inner doc comment — documents the containing item (module/crate)

    // ── Stability (std/compiler internals only) ────────────────────────────────
    // #[stable(feature = "rust1", since = "1.0.0")]
    // #[unstable(feature = "foo", issue = "12345")]

    // ── must_use ─────────────────────────────────────────────────────────────
    #[must_use]
    fn important_result() -> i32 {
        42
    }
    let _ = important_result(); // suppress the warning by binding to _

    // ── deprecated ────────────────────────────────────────────────────────────
    #[deprecated(since = "2.0.0", note = "use new_api() instead")]
    fn old_api() {}

    // ── repr: control memory layout ───────────────────────────────────────────
    #[repr(C)] // C-compatible layout (for FFI)
    struct CStruct {
        x: i32,
        y: i32,
    }

    #[repr(packed)] // no padding between fields
    struct Packed {
        a: u8,
        b: u32,
    }

    #[repr(align(16))] // 16-byte alignment
    struct Aligned {
        data: [u8; 16],
    }

    println!("[§31] attributes section — compile-check OK");
}

// =============================================================================
// §32  COMMON PATTERNS CHEATSHEET
// =============================================================================
fn section_32_patterns_cheatsheet() {
    println!("\n── §32 Patterns Cheatsheet ──────────────────────────────");

    // ── Destructuring everywhere ───────────────────────────────────────────────
    let (a, b, c) = (1, "two", 3.0);
    let Point2D { x, y } = Point2D { x: 3, y: 4 };
    let [first, second, ..] = [1, 2, 3, 4, 5]; // slice pattern
    println!(
        "destruct: {} {} {:.0} | pt: {},{} | arr: {} {}",
        a, b, c, x, y, first, second
    );

    // ── Tuple struct destructure ──────────────────────────────────────────────
    let Meters(val) = Meters(1.8);
    println!("meters: {}", val);

    // ── Nested patterns ───────────────────────────────────────────────────────
    let nested = Some(Point2D { x: 10, y: 20 });
    if let Some(Point2D { x, y }) = nested {
        println!("nested: ({}, {})", x, y);
    }

    // ── Guard in match ────────────────────────────────────────────────────────
    let num = 7;
    let desc = match num {
        n if n < 0 => "negative",
        0 => "zero",
        n if n % 2 == 0 => "positive even",
        _ => "positive odd",
    };
    println!("guard: {}", desc);

    // ── Multiple patterns with | ───────────────────────────────────────────────
    let n = 3;
    match n {
        1 | 2 | 3 => println!("one two or three"),
        _ => {}
    }

    // ── let else (Rust 1.65+): early return on mismatch ───────────────────────
    fn parse_u32(s: &str) -> Option<u32> {
        let Ok(n) = s.trim().parse::<u32>() else {
            return None;
        };
        Some(n)
    }
    println!("let-else: {:?}", parse_u32("42"));

    // ── RAII guard pattern ────────────────────────────────────────────────────
    // Resources tied to scope — drop at end of block automatically
    {
        let _guard = Mutex::new(0);
        let mut v = _guard.lock().unwrap(); // lock released when _guard drops
        *v = 1;
    }

    // ── Typestate pattern ────────────────────────────────────────────────────
    struct Locked;
    struct Unlocked;
    struct Door<State> {
        _state: std::marker::PhantomData<State>,
    }
    impl Door<Locked> {
        fn new() -> Self {
            Door {
                _state: std::marker::PhantomData,
            }
        }
        fn unlock(self) -> Door<Unlocked> {
            Door {
                _state: std::marker::PhantomData,
            }
        }
    }
    impl Door<Unlocked> {
        fn open(&self) {
            println!("door opened");
        }
        fn lock(self) -> Door<Locked> {
            Door {
                _state: std::marker::PhantomData,
            }
        }
    }
    Door::<Locked>::new().unlock().open(); // compile-time state machine

    // ── Extension trait pattern ───────────────────────────────────────────────
    trait StringExt {
        fn shout(&self) -> String;
    }
    impl StringExt for str {
        fn shout(&self) -> String {
            self.to_uppercase() + "!"
        }
    }
    println!("ext trait: {}", "hello".shout());

    // ── Deref coercion chain ──────────────────────────────────────────────────
    // Box<String> → String → str
    let boxed_string: Box<String> = Box::new(String::from("deref chain"));
    let s: &str = &boxed_string; // auto-deref coercion
    println!("deref coercion: {}", s);

    // ── Zero-sized types (ZSTs) ───────────────────────────────────────────────
    // PhantomData<T>, (), custom marker structs — zero runtime cost
    println!("ZST size: {} bytes", std::mem::size_of::<()>());
    println!(
        "PhantomData size: {} bytes",
        std::mem::size_of::<std::marker::PhantomData<String>>()
    );

    // ── std::mem utilities ────────────────────────────────────────────────────
    let mut x = 10i32;
    let mut y = 20i32;
    std::mem::swap(&mut x, &mut y);
    println!("after swap: x={} y={}", x, y);

    let old = std::mem::replace(&mut x, 99);
    println!("replace: old={} new={}", old, x);

    println!("size_of i64:    {} bytes", std::mem::size_of::<i64>());
    println!("size_of String: {} bytes", std::mem::size_of::<String>()); // 24
    println!("size_of &str:   {} bytes", std::mem::size_of::<&str>()); // 16 (fat ptr)
    println!("size_of Vec:    {} bytes", std::mem::size_of::<Vec<u8>>()); // 24

    // ── From / Into (conversion traits) ───────────────────────────────────────
    let s = String::from("from"); // From
    let n: i64 = i64::from(42i32); // widening int conversion
    let f: f64 = f64::from(42i32);
    let back: i32 = 42i64 as i32; // as for truncating/primitive casts
    let bits = f32::to_bits(3.14f32); // bit-level reinterpretation

    // Into is auto-derived when From is implemented:
    let s2: String = "into".into();
    println!("from/into: {} {}", s, s2);

    // ── TryFrom / TryInto (fallible conversions) ──────────────────────────────
    use std::convert::TryFrom;
    let big: i64 = 300;
    let small = i8::try_from(big); // Err — 300 doesn't fit in i8
    println!("TryFrom: {:?}", small);

    println!("── §32 done ─────────────────────────────────────────────");
}

// =============================================================================
//  SUPPLEMENTARY: fmt::Display for custom types
// =============================================================================
impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Circle(r) => write!(f, "Circle(r={:.1})", r),
            Shape::Rectangle {
                width: w,
                height: h,
            } => write!(f, "Rect({}×{})", w, h),
            Shape::Triangle(a, b, c) => write!(f, "Tri({},{},{})", a, b, c),
        }
    }
}

impl fmt::Debug for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Circle {{ x:{}, y:{}, r:{} }}",
            self.x, self.y, self.radius
        )
    }
}

impl fmt::Debug for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rect {{ x:{}, y:{}, w:{}, h:{} }}",
            self.x, self.y, self.w, self.h
        )
    }
}

fn process_text(text: &str) -> Cow<'_, str> {
    if text.contains("bad") {
        Cow::Owned(text.replace("bad", "good"))
    } else {
        Cow::Borrowed(text)
    }
}

fn cow_demo() {
    let hello = String::from("hello world");
    let result = process_text(&hello);
    println!("{:#?}", result);
    let bad = String::from("bad world");
    let result = process_text(&bad);
    println!("{:#?}", result);
}
