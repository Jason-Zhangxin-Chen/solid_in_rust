use std::collections::HashMap;
use std::time::Instant;

fn main() {
    let n = 20; // Change this value

    // Iterative
    let start = Instant::now();
    let iter_result = fib_iterative(n);
    let iter_dur = start.elapsed();

    // Recursive (caution: very slow for n > 30)
    let start = Instant::now();
    let rec_result = fib_recursive(n);
    let rec_dur = start.elapsed();

    // Memoized
    let start = Instant::now();
    let mut memo = HashMap::new();
    let mem_result = fib_memo(n, &mut memo);
    let mem_dur = start.elapsed();

    println!("Fibonacci({}) = {}", n, iter_result);
    println!("Iterative time: {:?}", iter_dur);
    println!("Recursive time: {:?}", rec_dur);
    println!("Memoized time:  {:?}", mem_dur);
}

fn fib_iterative(n: u32) -> u64 {
    if n == 0 { return 0; }
    let mut prev = 0;
    let mut curr = 1;
    for _ in 2..=n {
        let next = prev + curr;
        prev = curr;
        curr = next;
    }
    curr
}

fn fib_recursive(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_recursive(n - 1) + fib_recursive(n - 2),
    }
}

fn fib_memo(n: u32, memo: &mut HashMap<u32, u64>) -> u64 {
    if let Some(&result) = memo.get(&n) {
        return result;
    }
    let result = match n {
        0 => 0,
        1 => 1,
        _ => fib_memo(n - 1, memo) + fib_memo(n - 2, memo),
    };
    memo.insert(n, result);
    result
}

#[test]
fn test_fib() {
    main()
}