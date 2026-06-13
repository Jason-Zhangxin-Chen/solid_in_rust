

// std::rc::Weak<T> (single‑threaded) and std::sync::Weak<T> (thread‑safe) are non‑owning pointers
// to a value managed by Rc<T> or Arc<T> respectively. They do not keep the value alive; instead,
// you can attempt to “upgrade” a Weak back to a strong reference (Rc/Arc) if the value still exists.
// This makes them perfect for several architectural patterns.


// Breaking reference cycles (trees, graphs, doubly-linked lists)
// The classic use case: a child node holds a Weak pointer to its parent, while the parent holds an
// Rc/Arc (or Vec<Rc>) to its children. Without Weak, both directions would be strong, creating
// a cycle that prevents deallocation.

fn break_reference_cycle() {
    use std::rc::{Rc, Weak};
    use std::cell::RefCell;

    // When a subtree is dropped, the strong counts reach zero and the nodes are freed, while
    // the Weak parent pointers simply fail to upgrade.
    //
    // Arc::Weak is the same idea but for graphs shared across threads.
    struct Node {
        value: i32,
        parent: RefCell<Weak<Node>>,   // child → parent (weak)
        children: RefCell<Vec<Rc<Node>>>, // parent → children (strong)
    }
}

// Observer / event listener patterns.
// A subject maintains a list of Weak<dyn Observer> (or Weak<Listener>).
//
// Observers can be dropped independently without the subject needing to explicitly deregister them.
//
// The subject periodically cleans up dead weak pointers (those that fail to upgrade).

fn observer_pattern() {
    use std::rc::{Rc, Weak};
    use std::cell::RefCell;

    trait Observer {
        fn on_event(&self, event: &str);
    }

    struct Subject {
        observers: RefCell<Vec<Weak<dyn Observer>>>,
    }

    impl Subject {
        fn register_observer(&self, observer: Rc<dyn Observer>) {
            self.observers.borrow_mut().push(Rc::downgrade(&observer));
        }

        fn notify_observers(&self, event: &str) {
            let mut observers = self.observers.borrow_mut();

            // Clean up dead observers.
            observers.retain(|weak| weak.upgrade().is_some());
            for weak in observers.iter() {
                if let Some(observer) = weak.upgrade() {
                    observer.on_event(event);
                }
            }
        }
    }
}

// Caches that “don’t hold strong references”
// A cache stores Weak<T> to items that are otherwise owned by strong Arc<T> elsewhere in the program.
//
// When the last strong reference is dropped, the cache entry automatically becomes a dead weak pointer.
//
// The cache can then clean up or replace the entry lazily.
// shared cache / register across threads.
fn shared_cache() {
    use std::sync::{Arc, Weak, Mutex};
    use std::collections::HashMap;

    struct Data {
        id: u32,
    }

    impl Data {
        fn new(key: &str) -> Self {
            Data { id: key.len() as u32 }
        }
    }

    struct Cache {
        entries: Mutex<HashMap<String, Weak<Data>>>,
    }

    impl Cache {
        fn get_or_insert(&self, key: &str) -> Arc<Data> {
            let mut map = self.entries.lock().unwrap();

            // Prune dead entries (e.g. on every insert, or on a timer)
            map.retain(|_, weak| weak.upgrade().is_some());

            if let Some(weak) = map.get(key) {
                if let Some(strong) = weak.upgrade() {
                    return strong;
                }
            }
            let data = Arc::new(Data::new(key));
            map.insert(key.to_owned(), Arc::downgrade(&data));
            data
        }
    }
}

// Sentinels / checking if an object is still alive.
// You can hand out a Weak “token” to allow checking whether the original object still exists
// without extending its lifetime. Useful for:
//
// Progress tracking: a long‑running job holds a Weak<JobHandle>; external code polls
// upgrade().is_some() to see if the job is still running.
//
// Session validity: a client holds a Weak<Session> that it can upgrade only while the session
// is alive.


// Optional, non-owning references inside a structure.
// Sometimes you want a field that may or may not point to a shared object, but you don’t want that
// field to control the object’s lifetime. Weak provides a clean “maybe there” relationship without
// ownership.