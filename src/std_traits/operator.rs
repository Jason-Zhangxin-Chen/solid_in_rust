// Operators — Deref is the most impactful (it powers the entire coercion system).
// Add/Sub etc. for arithmetic overloading, Index/IndexMut for [] syntax.

// Deref / DerefMut are the most impactful, as they power the entire coercion system.
// They allow you to treat a type as if it were a reference to another type, which is
// fundamental to how Rust's borrowing and ownership system works.

fn de_ref_de_ref_mut() {
    use std::ops::Deref;

    struct MyBox<T>(T);

    impl<T> Deref for MyBox<T> {
        type Target = T;
        fn deref(&self) -> &T { &self.0 }
    }

    let b = MyBox(String::from("hello"));
    println!("{}", *b);        // deref → String

    fn greet(s: &str) { println!("{s}"); }
    greet(&b);                 // MyBox → String → str (chain!)

    // std examples of deref coercion:
    let s = String::from("hi");
    let r: &str = &s;          // &String → &str
    let v = vec![1,2,3];
    let sl: &[i32] = &v;       // &Vec → &[T]
}

// Add/Sub/Mul/Div
// std::ops::Add etc...
// Operator overloading. Implement these to make + - * / work on your types.
// Each returns an Output associated type, allowing flexible result types.
fn op_overloading() {
    use std::ops::{Add, Neg};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Vec2 { x: f64, y: f64 }

    impl Add for Vec2 {
        type Output = Vec2;
        fn add(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
        }
    }

    impl Neg for Vec2 {
        type Output = Vec2;
        fn neg(self) -> Vec2 { Vec2 { x: -self.x, y: -self.y } }
    }

    let a = Vec2 { x: 1.0, y: 2.0 };
    let b = Vec2 { x: 3.0, y: 4.0 };
    let c = a + b;   // Vec2 { x: 4.0, y: 6.0 }
    let d = -a;      // Vec2 { x: -1.0, y: -2.0 }
}

// Index / IndexMut
// std::ops:Index / IndexMut
// Enables [] indexing on your types. Index for read access, IndexMut for write access.
// Output is the returned reference type.
fn index() {
    use std::ops::{Index, IndexMut};

    struct Grid { data: Vec<Vec<i32>>, cols: usize }

    impl Index<(usize, usize)> for Grid {
        type Output = i32;
        fn index(&self, (row, col): (usize, usize)) -> &i32 {
            &self.data[row][col]
        }
    }

    impl IndexMut<(usize, usize)> for Grid {
        fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut i32 {
            &mut self.data[row][col]
        }
    }

    let mut g = Grid { data: vec![vec![0;3];3], cols: 3 };
    g[(0, 1)] = 42;            // IndexMut
    println!("{}", g[(0, 1)]); // 42 — Index
}

