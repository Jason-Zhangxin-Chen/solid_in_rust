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