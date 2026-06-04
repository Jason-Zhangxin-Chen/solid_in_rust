// Some principles in mind when writing Rust code.

// 1. Do not fight against compiler, just think in Rust way. The compiler is your friend, it helps
// you to write safe and efficient code. If you find yourself fighting with the compiler, it's often
// a sign that you're trying to do something that is not idiomatic in Rust. Instead of trying to
// work around the compiler's rules, try to understand why the compiler is complaining and how you
// can achieve your goals in a way that is more natural in Rust.
// [Refine your data model with compile rules] An elegant way.

// 2. Low latency vs. High throughput.
// 2.1 Low latency: Require the request should be processed as soon as possible, you cannot wait
//     for resources on the entire hotpath, which means once the request comes, you should have
//     computing power, dependent data ready, fanout communication channel ready, and the underlying
//     persistent layer ready if you want to store corresponding state etc..., with such demand,
//     you should avoid context switching between kernel and user space, or threads in async
//     execution frameworks, such as tokio etc... That would ask you to have kernel by pass net I/O,
//     heap pre-allocation, lock-free data structure, CPU-thread binding, thread isolation from
//     OS task scheduler, memory mapped file for persistent layer, etc...
// 2.2 High throughput: It is in the other end against low latency, it is more about how many
//     requests you can process in a given time, and you can afford to have tokio context switching
//     between async futures to process as much as possible requests which are ready to be processed.
//
// Trade off in between this two is common in system design.
// In Rust, you can often achieve both by using the right data structures and algorithms, and by
// leveraging Rust's ownership and borrowing system to minimize unnecessary copying and allocations.
// For example, using references (&str) instead of owned Strings can help reduce latency by avoiding
// heap allocations, while still allowing for high throughput by efficiently processing large
// amounts of data.

// 3. Concurrency Model. Plan threads and the processing pipelines with lock-free structures or with
//     Mutex<T> or RwLock<T> if necessary.

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

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
// instead, use log::info; crate.
use log::info;
pub fn process_better(data: &str) {
    info!("processing data: {}", data);
}

// Do not compare floats with ==
// Floating point numbers have precision issues, Directly equality checks ofen fail in surprising
// way.
/*
    fn compare_floats(a: f64, b: f64) -> bool {
        a == b // bad, may fail due to precision issues.
    }
*/

fn compare_floats(a: f64, b: f64) -> bool {
    let eps = 1e-10;
    (a - b).abs() < eps
}

// Wrapping every element in Box adds an extra heap allocation per element and a pointer indirection.
// Usually unnecessary if T is Sized.
/*
    let items: Vec<Box<String>> = vec! [
        Box::new("apple".to_owned()),
        Box::new("orange".to_owned()),
        Box::new("banana".to_owned()),
    ];
*/

// Tip: Only use Box inside Vec when T is unsized (e.g. dyn Trait) or you explicitly need indirection.
fn wrapping_box() {
    let items: Vec<String> = vec![
        "apple".to_owned(),
        "orange".to_owned(),
        "banana".to_owned(),
    ];

    for item in &items {
        println!("{}", item);
    }
}

// Do not over using .clone(). In a common scenario, for example, a global config is parsed from
// a config file, then you feed different sub set of config into different models of your service.
// In this case, then pass references to different models, instead of cloning for each. If you
// carefully design the config with seb sets, wrap different set of config with RC/Arc, you can
// avoid cloning the whole config for each model, and just clone the reference counted pointer,
// which is much cheaper.
struct AppConfig {
    wallet: Rc<WalletConfig>, // Rc wraps the wallet config that can be shared from different model.
    web3: Web3RpcConfig,
    db: Arc<DBConfig>,  // Arc wraps the DB config that can be shared from different models across threads.
    msg_bus: MsgQueueConfig
}

struct WalletConfig {
    key: String,
}

struct Web3RpcConfig {
    host: String,
    port: u16,
}

struct DBConfig {
    host: String,
    port: u16,
    db: String,
    credential: String,
}

struct MsgQueueConfig {
    host: String,
    port: u16,
    credential: String,
}

// accepting &str as a function's input rather than &String. The deref of String returns &str.
// It would be flexible to accept &str, as it allows callers to pass either a String or a string
// literal without needing to clone or convert the input. This can improve performance and reduce
// unnecessary allocations.
// let data = "my data";
// process_string(data); // works with &str
// let data_string = String::from("my data");
// process_string(&data_string); // works with &String, deref to &str
fn process_string(input: &str) {
    println!("Processing: {}", input);
}

// Overusing Rc<RefCell<T>>, Using interior mutability and reference counting when simpler ownership
// or borrowing works, leading to runtime panic risk on already borrowed.
fn overusing_rc_refcell() {
    let shared = Rc::new(RefCell::new(1));
    // ... sharing everywhere...
    // Restructure to avoid shared mutation, e.g., store mutable data in a single owner and pass
    // &mut where needed.
}

// Using as for numeric conversions, "as" silently truncates or discard data on overflow/underflow,
// which can lead to bugs. Use TryFrom/TryInto for fallible conversions.
fn numeric_conversion() {
    /*
        let x: u32 = 500;
        let y: u8 = x as u8; //y = 244 (truncated)
    */
    let x: u32 = 500;
    let y = u8::try_from(x).expect("value too large");
}

// Using &mut self when &self is enough. Unnecessary restricting callers and preventing concurrent
// reads.

// Bad: fn length(&mut self) -> usize { self.data.len() }
// Good: fn length(&self) -> usize { self.data.len() }

// Definning Default manually when the default value is not meaningful. It can lead to confusion
// and bugs if the default value is not meaningful or valid for the type. If the default value does
// not make sense, it's better to not implement Default at all, and require callers to explicitly
// provide the necessary values when creating an instance of the type. Or just derive the Default.
// This can help prevent misuse and ensure that the type is always initialized with valid data.

// Do not match for single bool, option or result, use if let instead.
fn do_not_match_for_singles() {
    let op = Some(42);
    if let Some(v) = op {
        println!("{}", v);
    }

    let op: Option<i32> = None;
    if let None = op {
        return;
    }

    let result: Result<i32, &str> = Ok(32);
    if let Ok(v) = result {
        println!("{}", v);
    }

    if let Err(e) = result {
        println!("{}", e);
        return
    }

    let result: Result<i32, &str> = Err("value too large");
    if let Err(e) = result {
        println!("{}", e);
        return
    }

    if let Ok(v) = result {
        println!("{}", v);
    }
}

// do not collect intermediate vectors unnecessarily, it waits allocations when you could chain iterators.
fn collect_vectors() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    // do not collect with intermediate vectors.
    let filtered: Vec<_> = data.iter().filter(|&&i| i>5 ).collect();
    let mapped: Vec<_> = filtered.iter().map(|&i| i*2).collect();

    // do it like this:
    let result: Vec<_> = data.iter().filter(|&&i| i>5).map(|&i| i*2).collect();
}

// do not use panic! if the error is recoverable.
fn divide(a: f64, b: f64) -> Result<f64, &'static str> {
    if b == 0.0 { panic!("division by zero");}
    Ok(a / b)
}

// better one:
fn divide1(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {return None; } // exact zero: explicit guard

    let result = a / b;
    if result.is_finite() { // catch +- Inf from near-zero b.
        Some(result)
    } else {
        None
    }
}

// Dynamic dispatching vs. static dispatching
trait Animal {
    fn bark(&self) -> &'static str;
}

struct Cat {}
impl Animal for Cat {
    fn bark(&self) -> &'static str {
        "miao miao"
    }
}

struct Duck {}
impl Animal for Duck {
    fn bark(&self) -> &'static str {
        "gar gar"
    }
}

// dynamic, the fn takes a refence of the trait object, so the caller does not move it into
// the fn, a refence is a kind of fat pointer, thus it is sized with
// (2 words: data pointer + vtable pointer) on the stack as the Animal is an unknown type at
// compile phase, thus it need an extra vtable pointer.
fn barking0(animal: &dyn Animal) {
    println!("{}", animal.bark());
}

// dynamic, the fn takes a Box<T> smart pointer which wraps the trait object, so the caller moves
// the Box into the fn, and the Box is a 2word pointer on the stack as well since the type is not
// resolved at compile time two, so it requires an extra vtable pointer.
fn barking1(animal: Box<dyn Animal>) {
    println!("{}", animal.bark());
}

// Static dispatching is better, as it's resolved in compile phase. The caller don't want to move
// the ownership of T, instead it just want to pass a reference of T without a vtable pointer as
// T's type is known at compile phase.
fn barking2<T: Animal>(animal: &T) {
    println!("{}", animal.bark());
}

// Static dispatching, but with a moved value of T as the input. If T: Copy, the compiler copies
// instead of moves. If T is not Copy, the caller moves the ownership of T into the fn, and the fn
// can use it directly
fn barking3<T: Animal>(animal: T) {
    println!("{}", animal.bark());
} // animal dropped here.

/*
// You write one generic function...
fn barking2<T: Animal>(animal: &T) { ... }
// ...compiler silently generates:
fn barking2_dog(animal: &Dog) { ... }
fn barking2_cat(animal: &Cat) { ... }
*/

// Use slices instead of &Vec<T>. Slices accept vectors, arrays, and other sliceable types.
// Do not do this although it works.
fn sum(set: &Vec<i32>) -> i32 {
    set.iter().sum()
}

// Better to do it in this way, taking a slice is much better.
fn sum1(set: &[i32]) -> i32 {
    set.iter().sum()
}

fn string_conversion() {
    // Call the From<T> to convert &str to String, which is more efficient than to_string()
    // for string literals.
    let s: String = "hello world".into();
    // Create owned data from borrowed data by clone.
    let s: String = "hello world".to_owned();
    // Create owned data from Display trait to_string(), a little bit heavy.
    let s: String = "hello world".to_string();
}
