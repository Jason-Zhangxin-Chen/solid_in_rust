use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashSet;
use std::sync::Mutex;

pub struct LeakTracker {
    // Delegate to the real system allocator
    inner: System,
    // Record every live allocation (pointer as usize for thread-safety)
    live: Mutex<HashSet<usize>>,
}

impl LeakTracker {
    // const constructor allows a static initializer
    pub const fn new() -> Self {
        LeakTracker {
            inner: System,
            // todo: fix this.
            live: Mutex::new(HashSet::new()),
        }
    }

    /// Call this before `main` exits to see what hasn't been deallocated.
    pub fn report_leaks(&self) {
        let live = self.live.lock().unwrap();
        if live.is_empty() {
            println!("✅ No memory leaks detected.");
        } else {
            println!("❌ Leaked {} allocation(s):", live.len());
            for ptr in live.iter() {
                println!("   ⚠️ 0x{:x}", ptr);
            }
        }
    }
}

// ---- Implement the GlobalAlloc trait ----
unsafe impl GlobalAlloc for LeakTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            self.live.lock().unwrap().insert(ptr as usize);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let removed = self.live.lock().unwrap().remove(&(ptr as usize));
        if !removed {
            panic!("Double-free or invalid free of {:?}", ptr);
        }
        self.inner.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc_zeroed(layout);
        if !ptr.is_null() {
            self.live.lock().unwrap().insert(ptr as usize);
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Remove old pointer, insert the new one (if different)
        let mut live = self.live.lock().unwrap();
        live.remove(&(ptr as usize));
        let new_ptr = self.inner.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            live.insert(new_ptr as usize);
        }
        new_ptr
    }
}

// ---- Install as the global allocator ----
#[global_allocator]
static GLOBAL: LeakTracker = LeakTracker::new();

// ---- Example usage ----
fn main() {
    let _normal = Box::new(42);   // will be dropped automatically ➜ not leaked
    let leaked = Box::new(3.14);  // we'll intentionally forget it
    std::mem::forget(leaked);

    // Some Vec allocation that will be freed
    let v = vec![1, 2, 3];
    drop(v);

    // Check for leaks before exiting
    GLOBAL.report_leaks();
}