// collect some classic memory leak scenarios in Rust.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// Strong reference circle, and why derived Debug is dangerous for cycles.
fn mem_leak_and_recursive_debug_print() {

    // Node derive the default Debug print trait, which prints every single field of a type.
    #[derive(Debug)]
    enum Node {
        Empty,
        Link(Rc<RefCell<Node>>),
    }

    fn main() {
        // b is a Rc pointer, point to RefCell<Node::Empty>, strong = 1.
        let b = Rc::new(RefCell::new(Node::Empty));
        // c clones Rc pointer b, point to the same value, strong = 2.
        let c = Rc::clone(&b);
        println!("{}, {}", Rc::strong_count(&b), Rc::strong_count(&c));

        // *(*b).borrow_mut() unwrap the inner data -- Node::Empty, and replace it with
        // Node::Link(*) where the * is a Rc pointer point to itself which is no longer empty after
        // the replacement.
        *(*b).borrow_mut() = Node::Link(Rc::clone(&c));
        // derive Debug, generate Debug::fmt for each field, it executes until hitting the Node::Empty.
        // Therefore, it will cause a stack overflow when trying to print the node.
        println!("{:?}", b)
    }
}

fn fix_the_mem_leak() {
    #[derive(Debug)]
    enum Node {
        Empty,
        Link(Weak<RefCell<Node>>),
    }

    fn main() {
        let b = Rc::new(RefCell::new(Node::Empty));
        let c = Rc::clone(&b);
        println!("{}, {}", Rc::strong_count(&b), Rc::strong_count(&c));
        *(*b).borrow_mut() = Node::Link(Rc::downgrade(&c));
    }
}