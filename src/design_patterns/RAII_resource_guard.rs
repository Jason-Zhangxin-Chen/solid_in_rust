// RAII stands for “Resource Acquisition is Initialisation” which is a terrible name.
// The essence of the pattern is that resource initialisation is done in the constructor of
// an object and finalisation in the destructor. This pattern is extended in Rust by using a
// RAII object as a guard of some resource and relying on the type system to ensure that access
// is always mediated by the guard object.

// Mutex guards are the classic example of this pattern from the std library
// (this is a simplified version of the real implementation):
fn mutex_guard_simplified() {
    use std::ops::Deref;

    struct Foo {}

    struct Mutex<T> {
        // We keep a reference to our data: T here.
        //..
    }

    struct MutexGuard<'a, T: 'a> {
        data: &'a T,
        //..
    }

    // Locking the mutex is explicit.
    impl<T> Mutex<T> {
        fn lock(&self) -> MutexGuard<T> {
            // Lock the underlying OS mutex.
            //..

            // MutexGuard keeps a reference to self
            MutexGuard {
                data: self,
                //..
            }
        }
    }

    // Destructor for unlocking the mutex.
    impl<'a, T> Drop for MutexGuard<'a, T> {
        fn drop(&mut self) {
            // Unlock the underlying OS mutex.
            //..
        }
    }

    // Implementing Deref means we can treat MutexGuard like a pointer to T.
    impl<'a, T> Deref for MutexGuard<'a, T> {
        type Target = T;

        fn deref(&self) -> &T {
            self.data
        }
    }

    fn baz(x: Mutex<Foo>) {
        let xx = x.lock();
        xx.foo(); // foo is a method on Foo.
        // The borrow checker ensures we can't store a reference to the underlying
        // Foo which will outlive the guard xx.

        // x is unlocked when we exit this function and xx's destructor is executed.
    }
}

// Example2. Memory pool with guard:
// A simple generic memory pool implementation that allows for efficient reuse of memory blocks.
// The core design.
// Pool<T>
//   │
//   ├── storage: Vec<UnsafeCell<T>>   ← pre-allocated slab on heap
//   └── free: Vec<usize>              ← indices of available slots
//
// PoolGuard<T>          ← RAII handle, auto-returns slot on drop
//   ├── pool: *mut Pool<T>
//   └── index: usize

use std::cell::UnsafeCell;

pub struct Pool<T> {
    // UnsafeCell<T> vs. Cell<T> and RefCell<T>.

    // Cell<T>
    //   ├── single-threaded only
    //   ├── T: Copy for get()
    //   ├── swaps whole values in/out
    //   └── NEVER gives out &T or &mut T
    //       → designed for small Copy types: bool, i32, Option<usize>
    //
    // RefCell<T>
    //   ├── single-threaded only
    //   ├── dynamic borrow checking at runtime (panics on violation)
    //   ├── gives out Ref<T> and RefMut<T> (smart reference wrappers)
    //   └── overhead: runtime borrow counter per cell
    //       → designed for single-threaded shared ownership with checked borrows
    //
    // UnsafeCell<T>
    //   ├── the PRIMITIVE — all other cells are built on this
    //   ├── no T bounds whatsoever
    //   ├── gives out *mut T — raw pointer, no checks
    //   ├── caller is responsible for safety invariants
    //   └── zero runtime overhead
    //       → designed for building custom synchronisation / data structures

    // Cell<T> requires T: Copy, which would prevent storing types like Vec<u8>, String, or any
    // non-Copy type in the pool — defeating the purpose entirely. Cell::get() returns T by value,
    // which requires a copy. Our pool needs to return &mut T so the caller can mutate the object
    // in place — Cell fundamentally cannot do that. Cell<T> deliberately never hands out a
    // reference to its interior — that is its entire safety guarantee. No &T or &mut T can ever
    // escape from a Cell. It only moves values in and out. Our PoolGuard needs to give the caller
    // a &mut T through DerefMut — which is exactly what Cell refuses to allow.

    // RefCell<T> could give out &mut T via borrow_mut(), and doesn't require T: Copy. So why not?
    // It technically works but has two problems:
    // Runtime borrow checking overhead — every acquire() and drop() would increment/decrement a
    // counter and check for conflicts. For a pool meant to avoid allocation overhead, adding
    // per-slot runtime checks is wasteful.
    // Panic risk — if you accidentally call borrow_mut() on an already-borrowed slot, it panics
    // at runtime. With UnsafeCell we control the invariant structurally — a slot is either in the

    storage: Vec<UnsafeCell<T>>,
    free:    Vec<usize>,
}

impl<T> Pool<T> {
    /// Pre-allocate `capacity` objects using `init` to construct each one.
    pub fn new(capacity: usize, mut init: impl FnMut() -> T) -> Self {
        let storage: Vec<UnsafeCell<T>> = (0..capacity)
            .map(|_| UnsafeCell::new(init()))
            .collect();

        let free = (0..capacity).rev().collect(); // stack: pop gives index 0 first

        Pool { storage, free }
    }

    /// Acquire a free slot. Returns None if the pool is exhausted.
    pub fn acquire(&mut self) -> Option<PoolGuard<T>> {
        let index = self.free.pop()?;
        Some(PoolGuard { pool: self as *mut Pool<T>, index })
    }

    /// Called by PoolGuard::drop — do not call directly.
    fn release(&mut self, index: usize) {
        self.free.push(index);
    }

    pub fn capacity(&self) -> usize { self.storage.len() }
    pub fn available(&self) -> usize { self.free.len() }
    pub fn in_use(&self) -> usize { self.capacity() - self.available() }
}

use std::ops::{Deref, DerefMut};

pub struct PoolGuard<T> {
    pool:  *mut Pool<T>,
    index: usize,
}

impl<T> PoolGuard<T> {
    pub fn index(&self) -> usize { self.index }
}

// Transparent read access — use guard just like &T
impl<T> Deref for PoolGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*(*self.pool).storage[self.index].get() }
    }
}

// Transparent write access — use guard just like &mut T
impl<T> DerefMut for PoolGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(*self.pool).storage[self.index].get() }
    }
}

// The magic — slot is returned automatically when guard goes out of scope
impl<T> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        unsafe { (*self.pool).release(self.index) }
    }
}

fn memory_pool_usage() {
    // Pre-allocate 4 buffers of 1024 bytes each
    let mut pool: Pool<Vec<u8>> = Pool::new(4, || Vec::with_capacity(1024));

    println!("capacity : {}", pool.capacity());   // 4
    println!("available: {}", pool.available());  // 4

    {
        let mut buf1 = pool.acquire().expect("pool exhausted");
        let mut buf2 = pool.acquire().expect("pool exhausted");

        println!("available: {}", pool.available()); // 2

        // Use guards exactly like &mut Vec<u8> — DerefMut kicks in
        buf1.extend_from_slice(b"hello");
        buf2.extend_from_slice(b"world");

        println!("buf1 index={} data={:?}", buf1.index(), &*buf1);
        println!("buf2 index={} data={:?}", buf2.index(), &*buf2);

    } // buf1 and buf2 dropped here — both slots returned automatically

    println!("available: {}", pool.available()); // 4 again
}