// Iterators — Iterator (implement next(), get 70+ methods free),
// IntoIterator (enables for loops),
// FromIterator (enables .collect()). These three form a complete system.


// Iterator
// The heart of Rust's iteration system. Implement next() — get map, filter, fold,
// collect, zip, chain, and 70+ other methods for free. Zero-cost abstractions, lazy evaluation.
fn iterator() {
    struct Counter { count: u32, max: u32 }

    impl Iterator for Counter {
        type Item = u32;
        fn next(&mut self) -> Option<u32> {
            if self.count < self.max {
                self.count += 1;
                Some(self.count)
            } else { None }
        }
    }

    let c = Counter { count: 0, max: 5 };
    let sum: u32 = c.sum();  // 15 — free from Iterator

    // Combinators — all lazy, zero intermediate allocation:
    let evens: Vec<_> = (1..=10)
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .collect();  // [4, 16, 36, 64, 100]
}

// IntoIterator.
// Enables a type to be used in for loops and iterator adapters.
// Implement this to make your collections feel native. Vec, HashMap, ranges all implement it.
fn into_iterator() {
    struct Grid { cells: Vec<Vec<i32>> }

    impl IntoIterator for Grid {
        type Item = i32;
        // Fix: add the type parameter <i32>
        type IntoIter = std::vec::IntoIter<i32>;

        fn into_iter(self) -> Self::IntoIter {
            // Flatten the nested vec, collect into a Vec<i32>, then iterate
            self.cells
                .into_iter()
                .flatten()
                .collect::<Vec<i32>>()
                .into_iter()
        }
    }

    let grid = Grid { cells: vec![vec![1, 2], vec![3, 4]] };
    for val in grid {         // `grid` is consumed (IntoIterator)
        println!("{val}");    // prints 1, 2, 3, 4
    }

    // Standard iterator modes still work
    let mut v = vec![1, 2, 3];
    for x in &v     { }   // &i32
    for x in &mut v { }   // &mut i32
    for x in v      { }   // i32 (consuming)
}

// FromIterator<T>
// Enables collect() to build your type from an iterator. Implement this to allow
// Iterator::collect() to produce your collection type.
fn from_iterator() {
    use std::iter::FromIterator;

    struct MySet<T: PartialEq> { items: Vec<T> }

    impl<T: PartialEq> FromIterator<T> for MySet<T> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut set = MySet { items: Vec::new() };
            for item in iter {
                if !set.items.contains(&item) {
                    set.items.push(item);
                }
            }
            set
        }
    }

    let my_set: MySet<i32> = vec![1, 2, 2, 3].into_iter().collect();
    // my_set.items == [1, 2, 3]
}