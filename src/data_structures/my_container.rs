
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Default, Debug)]
struct MyContainer<T> {
    slots: Vec<T>,
}

impl<T> MyContainer<T> {
    fn new() -> Self {
        Self { slots: Vec::new() }
    }

    // Creates an owning iterator (consumes the container)
    fn into_iter(self) -> MyContainerIntoIter<T> {
        MyContainerIntoIter::new(self)
    }

    // Creates a shared borrowed iterator
    fn iter(&self) -> MyContainerIter<'_, T> {
        MyContainerIter::new(&self.slots)
    }

    // Creates a mutable borrowed iterator
    fn iter_mut(&mut self) -> MyContainerIterMut<'_, T> {
        MyContainerIterMut::new(&mut self.slots)
    }
}

// ========== 1. Owned iterator (moves elements out) ==========
pub struct MyContainerIntoIter<T> {
    // Raw pointer to the start of the heap‑allocated buffer
    ptr: NonNull<T>,
    // Current read position (element index)
    idx: usize,
    // Total number of elements remaining
    len: usize,
    // Needed to correctly drop the buffer when the iterator is dropped
    _marker: PhantomData<Vec<T>>,
}

impl<T> MyContainerIntoIter<T> {
    fn new(container: MyContainer<T>) -> Self {
        let mut vec = container.slots;
        // Obtain raw parts of the Vec
        let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };
        let len = vec.len();
        // Prevent Vec from deallocating when it goes out of scope
        std::mem::forget(vec);
        Self {
            ptr,
            idx: 0,
            len,
            _marker: PhantomData,
        }
    }
}

impl<T> Iterator for MyContainerIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            None
        } else {
            // Read the value from the current position and advance
            let result = unsafe {
                // SAFETY: idx is within bounds, and we hold exclusive ownership of the buffer.
                self.ptr.as_ptr().add(self.idx).read()
            };
            self.idx += 1;
            Some(result) // todo: QQ: ownership moved here, how does it release the memory later?
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.idx, Some(self.len - self.idx))
    }
}

// Drop the remaining elements when the iterator is dropped before exhausting it.
impl<T> Drop for MyContainerIntoIter<T> {
    fn drop(&mut self) {
        // Drop all remaining elements // todo: QQ: as the scope end, items dropped?
        while let Some(_) = self.next() {}
        // Reconstruct the Vec to deallocate the buffer // todo: QQ: why shall we need this?
        if self.len > 0 {
            unsafe {
                let _ = Vec::from_raw_parts(self.ptr.as_ptr(), 0, self.len);
            }
        }
    }
}

// ========== 2. Shared borrowed iterator ==========
pub struct MyContainerIter<'a, T> {
    slice: &'a [T],
    pos: usize,
}

impl<'a, T> MyContainerIter<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        Self { slice, pos: 0 }
    }
}

impl<'a, T> Iterator for MyContainerIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.slice.len() {
            None
        } else {
            let item = &self.slice[self.pos];
            self.pos += 1;
            Some(item)
        }
    }
}

// ========== 3. Mutable borrowed iterator ==========
pub struct MyContainerIterMut<'a, T> {
    // No more `pos` — we shrink the slice from the front instead
    slice: &'a mut [T],
}

impl<'a, T> MyContainerIterMut<'a, T> {
    fn new(slice: &'a mut [T]) -> Self {
        Self { slice }
    }
}

impl<'a, T> Iterator for MyContainerIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // Take the slice out of self so we're not holding a borrow on self
        // while constructing the return value.
        let slice = std::mem::take(&mut self.slice);
        if slice.is_empty() {
            return None;
        }
        // split_at_mut gives us (&mut [first], &mut [rest]), both with lifetime 'a
        let (head, tail) = slice.split_at_mut(1);
        self.slice = tail;          // store the remainder back
        Some(&mut head[0])          // return the front element — lifetime is 'a ✓
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.slice.len(), Some(self.slice.len()))
    }
}

// ========== Convenience IntoIterator implementations ==========
impl<T> IntoIterator for MyContainer<T> {
    type Item = T;
    type IntoIter = MyContainerIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a MyContainer<T> {
    type Item = &'a T;
    type IntoIter = MyContainerIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut MyContainer<T> {
    type Item = &'a mut T;
    type IntoIter = MyContainerIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// ========== Example usage ==========
fn container_v1() {
    let mut c = MyContainer::new();
    c.slots.push(10);
    c.slots.push(20);
    c.slots.push(30);

    // 1. Shared borrow iteration
    for &val in &c {
        println!("shared: {}", val);
    }

    // 2. Mutable borrow iteration
    for val in &mut c {
        *val *= 2;
    }
    println!("After mutation: {:?}", c.slots); // [20, 40, 60]

    // 3. Owned iteration (moves out)
    let mut sum = 0;
    for val in c {
        sum += val;
    }
    // c is no longer usable here
    println!("Sum: {}", sum);
}

#[test]
fn test_demo() {
    container_v1();
}

/*
#[derive(Default)]
struct MyContainer<T> {
    slots: Vec<T>,
}

impl<T> MyContainer<T> {
    fn new() -> MyContainer<T> {
        MyContainer { slots: vec![] }
    }

    fn into_iter(self) -> MyContainerIntoIter<T> {

    }
}

// todo: implement owned iterator which move the ownership from
//  the underlying container.
struct MyContainerIntoIter<T> {

}

impl<T>


// todo: implement the borrowed iterator which borrow the
//  underlying container, the lifetime 'a should bind with the
//  underlying container.
struct MyContainerIter<T> {

}


// todo: implement the mutable iterator which do a mutable
//  borrow from the underlying container, the lifetime 'a should
//  bind with the underlying container.
struct MyContainerIterMut<T> {

}*/
