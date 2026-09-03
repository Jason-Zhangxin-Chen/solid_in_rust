// Fn / FnMut/ FnOnce
// The three closure traits, forming a hierarchy.
// FnOnce can be called once (consumes captured values).
// FnMut can be called multiple times with mutable capture.
// Fn can be called any number of times with shared access.
// Fn -> FnMut -> FnOnce
fn fn_traits() {
    // FnOnce — consumes captured variable, callable once only
    fn call_once(f: impl FnOnce()) { f(); }
    let name = String::from("Alice");
    call_once(move || println!("hello {name}"));
    // name moved into closure — cannot use name after

    // FnMut — mutates captured variable
    fn call_times(mut f: impl FnMut(), n: usize) {
        for _ in 0..n { f(); }
    }
    let mut count = 0;
    call_times(|| count += 1, 5);
    println!("{count}");  // 5

    // Fn — shared reference, callable any number of times
    fn apply_twice(f: impl Fn(i32) -> i32, x: i32) -> i32 {
        f(f(x))
    }
    let double = |x| x * 2;
    println!("{}", apply_twice(double, 3));  // 12
}