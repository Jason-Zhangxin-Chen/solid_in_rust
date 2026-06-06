
// share data by multiple independent objects.
// Imagine a texture cache in a game engine: many entities (sprites, UI elements) need to share
// the same texture. You don’t know which entity will be destroyed last – it depends on game logic
// that only unfolds at runtime. With Rc, every entity simply holds a clone of Rc<Texture>.
// The texture stays alive exactly as long as any entity still needs it.

use std::rc::Rc;

struct Texture { /* ... */ }

struct Sprite {
    texture: Rc<Texture>,  // shared ownership
    // other fields...
}

fn create_sprites() -> Vec<Sprite> {
    let shared_tex = Rc::new(Texture { /* ... */ });
    vec![
        Sprite { texture: shared_tex.clone() },
        Sprite { texture: shared_tex.clone() },
        // The texture will be dropped when all Sprites are gone.
    ]
}

// graph-like data structures with shared nodes. In a directed acyclic graph (DAG) representing a
// circuit, a node may have multiple parents. Because there is no single “owner” that outlives all
// others, you can’t use plain references. Rc makes each node independently owned yet shareable.

// We assume there is no circle in the DAG, thus there is no strong reference circle which leak mem.
struct Node {
    id: u32,
    edges: Vec<Rc<Node>>, // other nodes owned (shared) by this one
}

fn graph() {
    // Building a small DAG:
    let a = Rc::new(Node { id: 0, edges: vec![] });
    let b = Rc::new(Node { id: 1, edges: vec![] });
    // c is used by both a and b
    let c = Rc::new(Node { id: 2, edges: vec![a.clone(), b.clone()] });
}

// Callbacks that extend the lifetime of captured state.
// You might register an event handler that needs to keep some configuration alive, even after
// the original creator has been dropped.

fn extend_lifetime_with_callback() {
    use std::rc::Rc;
    use std::cell::RefCell;

    struct EventLoop {
        // stores callbacks that own an Rc<Config>
        handlers: Vec<Box<dyn Fn()>>,
    }

    impl EventLoop {
        // as the callback function (a closure) is stored in the event loop, it must be
        // 'static (no non-static references). The 'static is required because the closure is stored
        // in a trait object that has no explicit lifetime parameter. In Rust, when you write
        // Box<dyn Fn()> without any lifetime annotation, it’s actually shorthand for
        // Box<dyn Fn() + 'static>. This means the closure (and every value it captures) must be
        // valid for the entire remaining program execution — i.e., it cannot borrow any local
        // variables that would expire earlier. Why this default? The event loop could fire the
        // callback at any point in the future, potentially long after the register call and its
        // surrounding scope have disappeared. So the compiler must ensure that no captured
        // reference becomes dangling. The 'static bound guarantees that the closure is
        // self-contained: it either owns all the data it needs or holds only 'static references
        // (like string literals).
        // In the example, we satisfied this bound by using Rc
        fn register<F: Fn() + 'static>(&mut self, f: F) {
            self.handlers.push(Box::new(f));
        }

        fn new() -> Self {
            EventLoop { handlers: vec![] }
        }
    }

    struct Config { threshold: i32 }
    // config is a Rc<T>, be cloned and moved into the closure.
    let config = Rc::new(RefCell::new(Config { threshold: 10 }));
    let mut loop_ref = EventLoop::new();/* some event loop */;
    {
        let cfg = config.clone();
        loop_ref.register(move || {
            // This closure owns cfg, keeping the Config alive as long as
            // the closure is stored in the event loop.
            println!("threshold is {}", cfg.borrow().threshold);
        });
    }
    // Even if the original `config` goes out of scope, the event loop still
    // holds an Rc, so the Config remains valid.
}

// Mutual cyclic references with Rc + RefCell
// Sometimes you need a parent to own children, and children to refer back to their parent.
// The lifetimes are circular, so raw references would be impossible. Rc combined with RefCell
// (and Weak to avoid leaks) lets you model this.
fn cyclic_reference() {
    use std::rc::{Rc, Weak};
    use std::cell::RefCell;

    struct Node {
        value: i32,
        parent: RefCell<Weak<Node>>,   // non-owning backlink
        children: RefCell<Vec<Rc<Node>>>,
    }

    let leaf = Rc::new(Node {
        value: 3,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    let branch = Rc::new(Node {
        value: 5,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![leaf.clone()]),
    });
    *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
}