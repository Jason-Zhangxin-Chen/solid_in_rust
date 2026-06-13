
// share configuration / immutable data across many threads.
// You want to spin up several worker threads, each needing read‑only access to the same large
// configuration. Arc lets each thread hold an owning handle to the config, guaranteeing it stays
// alive until all threads are done.

fn readonly_share_between_workers() {
    use std::sync::Arc;
    use std::thread;

    struct Config {
        // lots of data…
        db_url: String,
        pool_size: u32,
    }

    let config = Arc::new(Config {
        db_url: "postgres://...".into(),
        pool_size: 10,
    });

    let mut handles = vec![];

    for _ in 0..4 {
        let cfg = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            // cfg is moved into the thread, giving it an owned Arc
            println!("Worker using {}", cfg.db_url);
            // … do work with cfg …
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    // config stays alive until the last thread finishes
}

// Mutable data share via Arc<Mutex<T>>, Arc<RwLock<T>>
// Often you need multiple threads to both read and mutate the same data. Wrapping a Mutex
// (or RwLock) inside an Arc gives you a thread‑safe, reference‑counted handle to a piece of
// shared mutable state.
fn rw_data_share_between_workers() {
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread;

    let counter = Arc::new(Mutex::new(0u64));
    let rw_counter = Arc::new(RwLock::new(0u64));
    let mut handles = vec![];

    for _ in 0..10 {
        let cnt = Arc::clone(&counter);
        let rw_cnt = Arc::clone(&rw_counter);
        handles.push(thread::spawn(move || {
            {
                let mut val = cnt.lock().unwrap();
                *val += 1;
                // mutex is released here
            }
            {
                let mut val = rw_cnt.write().unwrap();
                *val += 1;
            }   // rwlock is release here
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap()); // 10
    println!("Final count: {}", *rw_counter.read().unwrap()); // 10
}

fn read_heavy_workers() {
    use std::sync::{Arc, RwLock};
    use std::thread;

    let data = Arc::new(RwLock::new(vec![1, 2, 3]));

    // Many readers
    let readers: Vec<_> = (0..3).map(|_| {
        let d = Arc::clone(&data);
        thread::spawn(move || {
            let v = d.read().unwrap();
            println!("Reader sees: {:?}", *v);
        })
    }).collect();

    // One writer
    let writer = {
        let d = Arc::clone(&data);
        thread::spawn(move || {
            let mut v = d.write().unwrap();
            v.push(4);
        })
    };

    writer.join().unwrap();
    for r in readers { r.join().unwrap(); }
}

// share cache or connection pool in a muti-threaded server
// In a web server, you might want a single connection pool or cache shared by all request‑handling
// threads. The pool is initialized once and then given to every handler via Arc.
fn cache_share_workers() {
    use std::sync::Arc;
    use std::collections::HashMap;
    use std::sync::RwLock;

    struct Cache {
        map: RwLock<HashMap<String, String>>,
    }

    fn handle_request(cache: Arc<Cache>, key: &str) {
        if let Some(value) = cache.map.read().unwrap().get(key) {
            println!("Cache hit: {value}");
        } else {
            // fetch from DB, then write into cache
            let mut map = cache.map.write().unwrap();
            map.insert(key.to_owned(), "computed_value".to_owned());
        }
    }

    let cache = Arc::new(Cache {
        map: RwLock::new(HashMap::new()),
    });

    // Imagine spawning a thread per request:
    let c = cache.clone();
    std::thread::spawn(move || handle_request(c, "user_1"));
}

// Callbacks / event handlers across threads (e.g., GUI or async runtimes)
// When a closure must be sent to another thread and it needs to keep some state alive, you move an
// Arc into the closure. The closure then owns a share of the state, which remains valid for as
// long as the callback exists.
fn extend_lifetime_by_closure_between_workers() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    struct EventBus {
        // stores callbacks (here simplified as a single callback)
        callback: Mutex<Option<Box<dyn Fn() + Send + 'static>>>,
    }

    impl EventBus {
        fn set_callback(&self, f: Box<dyn Fn() + Send + 'static>) {
            *self.callback.lock().unwrap() = Some(f);
        }
        fn emit(&self) {
            if let Some(ref f) = *self.callback.lock().unwrap() {
                f();
            }
        }
    }

    let state = Arc::new(Mutex::new(42));
    let bus = Arc::new(EventBus { callback: Mutex::new(None) });

    {
        let st = Arc::clone(&state);
        // The closure captures `st` (an Arc), so it can be sent to another thread.
        bus.set_callback(Box::new(move || {
            println!("State is: {}", *st.lock().unwrap());
        }));
    }

    // Emit from another thread
    let bus2 = Arc::clone(&bus);
    thread::spawn(move || {
        bus2.emit();
    }).join().unwrap();
}

// breaking cycles with Weak in concurrent data structures.
// Just like with Rc, Arc can leak memory if strong reference cycles exist. Use std::sync::Weak
// (the atomic version) to break cycles in thread‑safe graphs.
fn break_circle() {
    use std::sync::{Arc, Weak, Mutex};

    struct Node {
        value: i32,
        parent: Mutex<Weak<Node>>,   // non‑owning backreference
        children: Mutex<Vec<Arc<Node>>>,
    }

    let leaf = Arc::new(Node {
        value: 3,
        parent: Mutex::new(Weak::new()),
        children: Mutex::new(vec![]),
    });
    let branch = Arc::new(Node {
        value: 5,
        parent: Mutex::new(Weak::new()),
        children: Mutex::new(vec![leaf.clone()]),
    });
    *leaf.parent.lock().unwrap() = Arc::downgrade(&branch);
}