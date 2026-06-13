
// RefCell<T> provides interior mutability with runtime borrow checking. Unlike Cell<T>, it allows
// you to get a mutable reference (&mut T) to the inner value through a shared &RefCell<T>, at the
// cost of a runtime borrow counter that panics if Rust’s aliasing rules are violated.
//
// You use RefCell<T> when you need to mutate data behind an immutable reference, and the mutation
// pattern is more complex than just getting or setting a whole value – you need actual references
// into the data, or the data is large and you want to avoid copying it.

// Mutating a field inside a large struct via &self.
// Suppose you have a struct with a method that must be &self (e.g., to conform to a trait), but
// internally you need to modify a field. If the field is a simple Copy type, Cell<T> works. But if
// the field is a Vec, HashMap, or any large type that you need to call push, insert, or other
// mutating methods on, you need a &mut reference into it. RefCell<T> gives you that.

fn interior_mutability_struct_field() {
    use std::cell::RefCell;

    struct Cache {
        // Large, non-Copy data
        entries: RefCell<Vec<String>>,
    }

    impl Cache {
        // todo: refine the println!
        fn add_entry(&self, entry: String) {
            // self is &self, but we can borrow_mut the RefCell
            if let Ok(mut v) = self.entries.try_borrow_mut() {
                v.push(entry);
                return;
            }
            println!("already borrowed");
        }

        fn get_entries(&self) -> Vec<String> {
            if let Ok(v) = self.entries.try_borrow() {
                return v.clone()
            }
            println!("already borrowed");
            Vec::new()
        }
    }

    let cache = Cache {
        entries: RefCell::new(vec![]),
    };

    cache.add_entry("hello".into());
    cache.add_entry("world".into());
    println!("{:?}", cache.get_entries()); // ["hello", "world"]
}

// Rc<RefCell<T>> - shared ownership with interior mutability. (The backbone of graphs, trees,
// UI widgets).
// RefCell<T> is often used together with Rc<T> to allow multiple owners of the same data, where
// the data can be mutated. This is common in graph and tree data structures, where nodes may
// have multiple parents or children, and you need to mutate the structure (e.g., add/remove edges)
// while sharing ownership of the nodes.
// This is the most classic pattern: combine Rc for shared ownership and RefCell for mutability.
// It lets you build mutable, shared data structures that can be manipulated from multiple owners.

fn share_ownership_with_interior_mutability() {
    use std::rc::Rc;
    use std::cell::RefCell;

    // todo: check if there is a strong refence circle in this example.
    struct Node {
        value: i32,
        children: RefCell<Vec<Rc<Node>>>,
        parent: RefCell<Option<Rc<Node>>>,
    }

    impl Node {
        fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
            // TODO: how to write the try_borrow_mut() in an elegant way?
            // Borrow mutably to modify the children list
            parent.children.borrow_mut().push(child.clone());
            // Set child's parent (also needs mutable access)
            *child.parent.borrow_mut() = Some(parent.clone());
        }
    }

    let root = Rc::new(Node {
        value: 0,
        children: RefCell::new(vec![]),
        parent: RefCell::new(None),
    });

    let leaf = Rc::new(Node {
        value: 1,
        children: RefCell::new(vec![]),
        parent: RefCell::new(None),
    });

    Node::add_child(&root, leaf);

    // Both root and leaf can be accessed and mutated from anywhere that holds an Rc.
}

// Mock objects in unit tests.
// When testing code that depends on a trait, you often create a mock that records calls or
// returns canned values. The mock needs to mutate its internal state (e.g., a call counter)
// while the code under test calls methods through a &self reference. RefCell lets you keep
// the mock simple and single‑threaded.
fn interior_mutate_mock() {
    use std::cell::RefCell;

    trait Logger {
        fn log(&self, msg: &str);
    }

    struct MockLogger {
        // We need to record messages behind an &self method
        recorded: RefCell<Vec<String>>,
    }

    impl Logger for MockLogger {
        fn log(&self, msg: &str) {
            // todo: use try_borrow_mut()
            self.recorded.borrow_mut().push(msg.to_owned());
        }
    }

    #[test]
    fn test_logging() {
        let mock = MockLogger {
            recorded: RefCell::new(vec![]),
        };

        // Code under test uses the Logger trait (with &self)
        do_something(&mock);

        assert_eq!(*mock.recorded.borrow(), vec!["start", "end"]);
    }

    fn do_something(logger: &dyn Logger) {
        logger.log("start");
        // ...
        logger.log("end");
    }
}

// Adapter / Wrapper that needs to call &mut self on an inner object from &self.
// You’re wrapping a standard library type, but the outer type’s public API requires &self
// (perhaps for ergonomics, or because it implements a trait). The inner type needs &mut self
// to perform I/O or update state. RefCell bridges the gap.

fn interior_mutable_wrapper() {
    use std::cell::RefCell;
    use std::io::{self, Write};

    struct BufWriter<W: Write> {
        inner: RefCell<W>,   // We'll need &mut W to write
    }

    impl<W: Write> BufWriter<W> {
        fn new(inner: W) -> Self {
            BufWriter {
                inner: RefCell::new(inner),
            }
        }

        fn write_all(&self, buf: &[u8]) -> io::Result<()> {
            // self is shared, but we need a mutable reference to the writer
            self.inner.borrow_mut().write_all(buf)
        }
    }

    let mut buf = Vec::new();
    let writer = BufWriter::new(&mut buf);
    writer.write_all(b"hello").unwrap();
    // buf now contains the data
}

// Lazy initialization of a field on an immutable struct.
// Sometimes you create a struct in a shared context and only later fill in some fields.
// Using Option<T> and RefCell<Option<T>> lets you set the value once via a &self method.

fn lazy_initialization() {

    struct Connection {
        con: u32, // placeholder for a real connection object
    }
    impl Connection {
        fn new() -> Connection {
            Connection { con: 1 }
        }
    }

    impl Connection {
        fn execute(&self, sql: &str) {}
    }

    use std::cell::RefCell;
    struct Database {
        // Connection might be established after construction
        conn: RefCell<Option<Connection>>,
    }

    impl Database {
        // Because ensure_connected is &self, callers can hold a simple &Database and still
        // trigger initialization. RefCell permits the one‑time mutation.
        fn ensure_connected(&self) {
            if self.conn.borrow().is_none() {
                let conn = Connection::new(); // hypothetical
                *self.conn.borrow_mut() = Some(conn);
            }
        }

        fn query(&self, sql: &str) {
            self.ensure_connected();
            let conn = self.conn.borrow();
            let conn = conn.as_ref().unwrap();
            conn.execute(sql);
        }
    }
}