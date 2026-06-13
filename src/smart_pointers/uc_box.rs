
// Recursive types, indirection to get a known size.
// In Rust, every type must have a compile-time-known size. A struct that directly contains itself
// has infinite size, which is impossible. Box provides indirection: the field stores a fixed-size
// pointer to a heap-allocated value, making the struct’s size finite.
//
// Classic example: a cons list.
fn cons_list() {
    enum List<T> {
        Cons(T, Box<List<T>>),  // Box gives the tail a known size
        Nil,
    }

    use List::*;

    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
}

// Trait objects (dynamic dispatch).
// The trait object need to be boxed to have fixed sized pointer in the heap with a data
// pointer + vtable.
fn trait_object() {
    trait Draw {
        fn draw(&self);
    }

    struct Circle { radius: f64 }
    struct Square { side: f64 }

    impl Draw for Circle { fn draw(&self) { /* ... */ } }
    impl Draw for Square { fn draw(&self) { /* ... */ } }

    fn get_shapes(which: bool) -> Box<dyn Draw> {
        if which {
            Box::new(Circle { radius: 10.0 })
        } else {
            Box::new(Square { side: 5.0 })
        }
    }

    let shape = get_shapes(true);
    shape.draw(); // dynamic dispatch
}

// Large data to avoid stack overflow, cheap moves.
// The stack has limited space (often 8MB on typical systems). Allocating a huge struct or array on
// the stack can cause a stack overflow. Moving such a large value also means copying all bytes.
// Box<T> moves the data to the heap; moving the Box itself only copies the pointer.
fn cheap_move() {
    struct LargeData([u8; 10_000_000]);

    fn process(data: Box<LargeData>) {
        // data is on the heap, only a pointer is passed in registers
        // ...
    }

    let huge = Box::new(LargeData([0; 10_000_000]));
    process(huge); // cheap: moves just a pointer
}

// Owned dynamically sized types (DSTs)
// Slice types [T] and str are dynamically sized – their size isn’t known at compile time.
// You can’t put them directly on the stack, but you can own them via Box<[T]> or Box<str>.
fn dst() {
    // Turn a Vec into a boxed slice (no extra capacity)
    let v: Vec<i32> = vec![1, 2, 3];
    let boxed_slice: Box<[i32]> = v.into_boxed_slice();
    // boxed_slice owns the elements on the heap, exactly 3 elements.

    // Box<str> from a String
    let s = String::from("hello");
    let boxed_str: Box<str> = s.into_boxed_str();
}

// Reducing enum size for variants with wildly different sizes.
// If an enum has one variant that is much larger than the others, the whole enum is as large as
// that largest variant. You can Box the large variant to shrink the enum’s footprint on the stack.
// This is a common optimization in async state machines and parser ASTs.
fn shrink_enum_size() {
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        BigChunk(Box<[u8; 1024 * 1024]>), // 1 MiB on the heap, not the stack
    }
    // Now Message's size is much smaller (pointer size) instead of 1 MiB.
}

// Returning closures or opaque types.
// When you need to return a closure from a function, its type is unnameable. You can box it into a
// trait object.
fn returning_closure() {
    fn make_adder(x: i32) -> Box<dyn Fn(i32) -> i32> {
        Box::new(move |y| x + y)
    }

    let add5 = make_adder(5);
    println!("{}", add5(10)); // 15
}

