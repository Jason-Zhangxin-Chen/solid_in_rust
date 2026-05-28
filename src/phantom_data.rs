use std::marker::PhantomData;

// ── 1. Unused type parameter ──────────────────────────────────────────────────
//
// Id<User> and Id<Post> both wrap a u64, but they are distinct types.
// The compiler will reject passing a PostId where a UserId is expected.

struct Id<T> {
    value: u64,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    fn new(value: u64) -> Self {
        Id { value, _marker: PhantomData }
    }
}

struct User;
struct Post;

type UserId = Id<User>;
type PostId = Id<Post>;

fn lookup_user(id: UserId) {
    println!("  Looking up user #{}", id.value);
}

fn example_unused_type_param() {
    println!("\n── 1. Unused type parameter ──");

    let uid: UserId = Id::new(42);
    let _pid: PostId = Id::new(42); // same inner value, different type

    lookup_user(uid);
    // lookup_user(_pid); // ← uncomment to see the compile error
    println!("  UserId and PostId are distinct types despite both wrapping u64");
}

// ── 2. Raw pointers — ownership & Drop ───────────────────────────────────────
//
// PhantomData<T> tells the compiler "this struct owns a T", so it
// runs T's destructor on drop and infers correct variance.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

struct MyBox<T> {
    ptr: *mut T,
    _marker: PhantomData<T>, // "I own a T"
}

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        let layout = Layout::new::<T>();
        let ptr = unsafe {
            let raw = alloc(layout) as *mut T;
            ptr::write(raw, value);
            raw
        };
        MyBox { ptr, _marker: PhantomData }
    }

    fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr);           // runs T's destructor
            dealloc(self.ptr as *mut u8, Layout::new::<T>());
        }
    }
}

fn example_raw_pointers() {
    println!("\n── 2. Raw pointers — ownership & Drop ──");

    let b = MyBox::new(String::from("hello from MyBox"));
    println!("  MyBox contains: {}", b.get());
    // The String inside is properly dropped when `b` goes out of scope.
}

// ── 3. Lifetime relationship ──────────────────────────────────────────────────
//
// PhantomData<&'a T> tells the compiler this struct borrows a &'a T,
// so the iterator cannot outlive the slice it points into.

struct RawIter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>, // "I borrow a &'a T"
}

impl<'a, T> RawIter<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        RawIter {
            ptr: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for RawIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            return None;
        }
        unsafe {
            let val = &*self.ptr;
            self.ptr = self.ptr.add(1);
            Some(val)
        }
    }
}

fn example_lifetime() {
    println!("\n── 3. Lifetime relationship ──");

    let data = vec![10, 20, 30];
    let iter = RawIter::new(&data);
    let collected: Vec<_> = iter.collect();
    println!("  Iterated via raw pointer: {:?}", collected);
    // `iter` cannot outlive `data` — the borrow checker sees 'a.
}

// ── 4. Variance ───────────────────────────────────────────────────────────────
//
// PhantomData controls whether Foo<Dog> can substitute for Foo<Animal>.
//
//   PhantomData<T>      → covariant  (like &T)     — Foo<Dog> usable as Foo<Animal>
//   PhantomData<*mut T> → invariant  (like &mut T)  — Foo<Dog> unrelated to Foo<Animal>
//   PhantomData<fn(T)>  → contravariant             — Foo<Animal> usable as Foo<Dog>
//
// TypedCell must be invariant: exposing &mut T while being covariant
// would let you smuggle a short-lived reference into a long-lived slot.

use std::cell::UnsafeCell;

struct TypedCell<T> {
    value: UnsafeCell<T>,
    _marker: PhantomData<*mut T>, // forces invariance over T
}

impl<T: Copy + std::fmt::Debug> TypedCell<T> {
    fn new(v: T) -> Self {
        TypedCell { value: UnsafeCell::new(v), _marker: PhantomData }
    }

    fn get(&self) -> T {
        unsafe { *self.value.get() }
    }

    fn set(&self, v: T) {
        unsafe { *self.value.get() = v; }
    }
}

fn example_variance() {
    println!("\n── 4. Variance ──");

    let cell = TypedCell::new(100_i32);
    println!("  Initial value : {:?}", cell.get());
    cell.set(999);
    println!("  After set(999): {:?}", cell.get());
    println!("  TypedCell is invariant over T via PhantomData<*mut T>");
}

// ── 5. !Send / !Sync ─────────────────────────────────────────────────────────
//
// PhantomData<*mut ()> opts out of both Send and Sync.
// PhantomData<Rc<()>>  opts out of Send only (Rc is !Send but Sync is fine).

use std::rc::Rc;

struct SingleThreaded {
    handle: u32,
    _not_send_sync: PhantomData<*mut ()>, // !Send + !Sync
}

struct NotSend {
    data: u32,
    _not_send: PhantomData<Rc<()>>, // !Send, Sync is fine
}

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn example_send_sync() {
    println!("\n── 5. !Send / !Sync ──");

    // A plain struct auto-implements both:
    struct Normal { _x: u32 }
    assert_send::<Normal>();
    assert_sync::<Normal>();
    println!("  Normal         :  Send ✓   Sync ✓");

    // Uncommenting either line below is a compile error:
    // assert_send::<SingleThreaded>(); // error: SingleThreaded is not Send
    // assert_sync::<SingleThreaded>(); // error: SingleThreaded is not Sync
    // assert_send::<NotSend>();        // error: NotSend is not Send
    println!("  SingleThreaded : !Send    !Sync   (PhantomData<*mut ()>)");
    println!("  NotSend        : !Send     Sync   (PhantomData<Rc<()>>)");

    // Constructing them works fine on the same thread:
    let _a = SingleThreaded { handle: 1, _not_send_sync: PhantomData };
    let _b = NotSend { data: 42, _not_send: PhantomData };
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    example_unused_type_param();
    example_raw_pointers();
    example_lifetime();
    example_variance();
    example_send_sync();
}