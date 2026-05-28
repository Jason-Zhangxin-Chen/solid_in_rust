#[derive(Default)]
struct MyContainer<T> {
    slots: Vec<T>,
}

impl<T> MyContainer<T> {
    fn new() -> Self {
        Self { slots: Vec::new() }
    }

    // Owned iterator – consumes the container
    fn into_iter(self) -> MyContainerIntoIter<T> {
        MyContainerIntoIter::new(self.slots)
    }

    // Shared iterator
    fn iter(&self) -> MyContainerIter<'_, T> {
        MyContainerIter::new(&self.slots)
    }

    // Mutable iterator
    fn iter_mut(&mut self) -> MyContainerIterMut<'_, T> {
        MyContainerIterMut::new(&mut self.slots)
    }
}

// ========== 1. Owned iterator (safe, no unsafe) ==========
pub struct MyContainerIntoIter<T> {
    // We keep the Vec and drain it from the front
    vec: Vec<T>,
    // Optional: we could store an index, but remove(0) already tracks position.
    // For simplicity we just remove from front each time.
}

impl<T> MyContainerIntoIter<T> {
    fn new(vec: Vec<T>) -> Self {
        Self { vec }
    }
}

impl<T> Iterator for MyContainerIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.vec.is_empty() {
            None
        } else {
            // Remove the first element – this moves ownership out of the Vec.
            // Complexity O(n) per call, but it's safe and easy to understand.
            Some(self.vec.remove(0))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len();
        (len, Some(len))
    }
}

// When the iterator is dropped before exhaustion, the remaining elements
// are automatically dropped because `vec` goes out of scope.
// No manual Drop implementation needed.

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice.len();
        (len-self.pos, Some(len-self.pos))
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

// ========== Convenience IntoIterator impls ==========
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
fn container_v2() {
    let mut c = MyContainer::new();
    c.slots.push(100);
    c.slots.push(200);
    c.slots.push(300);

    // Shared iteration
    for val in &c {
        println!("shared: {}", val);
    }

    // Mutable iteration
    for val in &mut c {
        *val += 5;
    }
    println!("After mutation: {:?}", c.slots); // [105, 205, 305]

    // Owned iteration (consumes c)
    let sum: i32 = c.into_iter().sum();
    println!("Sum: {}", sum); // 615
    // c is now moved and cannot be used
}

#[test]
fn test_demo() {
    container_v2();
}
