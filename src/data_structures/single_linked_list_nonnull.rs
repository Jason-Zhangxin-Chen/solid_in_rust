use std::ptr::NonNull;
use std::marker::PhantomData;
use std::alloc::{alloc, dealloc, Layout};

// ========== Node ==========
struct Node<T> {
    elem: T,
    next: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    // Allocate a Node on the heap and return a NonNull pointer to it.
    // We use the global allocator directly — no Box involved.
    fn new(elem: T, next: Option<NonNull<Node<T>>>) -> NonNull<Self> {
        let layout = Layout::new::<Node<T>>();
        unsafe {
            // alloc() returns *mut u8; cast to *mut Node<T>
            let raw = alloc(layout) as *mut Node<T>;
            // Write the value into the allocation without reading
            // the (uninitialised) memory that was there before.
            raw.write(Node { elem, next });
            // NonNull::new_unchecked: we trust the allocator to never
            // return null (it aborts instead on OOM in std).
            NonNull::new_unchecked(raw)
        }
    }

    // Deallocate a node WITHOUT running its destructor.
    // The caller is responsible for having already moved `elem` out.
    unsafe fn dealloc(ptr: NonNull<Self>) {
        let layout = Layout::new::<Node<T>>();
        dealloc(ptr.as_ptr() as *mut u8, layout);
    }
}
/*
// ========== LinkedList ==========
// Option<NonNull<T>> is the idiomatic "nullable NonNull" — the compiler
// optimises it to a single pointer-sized word (null == None).
struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    size: usize,
    // NonNull<T> is already covariant over T, but we still need
    // PhantomData<T> so the compiler knows this struct *owns* T values
    // (affects drop-check and Send/Sync derivation).
    _marker: PhantomData<T>,
}

// SAFETY: We have exclusive ownership of every node — no aliasing.
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,   // None == null, zero cost at runtime
            size: 0,
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, elem: T) {
        // Allocate the new node, pointing at the current head.
        let new_node = Node::new(elem, self.head);
        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        // Option::map runs only when head is Some(ptr).
        self.head.map(|ptr| unsafe {
            // as_ptr() gives *mut Node<T> from NonNull — always non-null.
            let node = ptr.as_ptr();

            // Advance head before we destroy the node.
            self.head = (*node).next;
            self.size -= 1;

            // Move `elem` out of the node without copying the whole Node.
            // ptr::read performs a bitwise copy, leaving the source
            // memory as "logically uninitialised" (we must not use it again).
            let elem = (*node).elem_read();

            // Free the allocation. elem's Drop will run normally when
            // the caller's binding goes out of scope.
            Node::dealloc(ptr);
            elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        // as_ref() on NonNull gives &Node<T> tied to the lifetime of &self.
        self.head.map(|ptr| unsafe {
            // SAFETY: ptr is valid and we hold &self so no mutation can occur.
            &ptr.as_ref().elem
        })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.map(|mut ptr| unsafe {
            // SAFETY: ptr is valid; &mut self guarantees exclusive access.
            &mut ptr.as_mut().elem
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

// Helper: read elem without going through a full Box
impl<T> Node<T> {
    // Reads `elem` out of the node via pointer arithmetic.
    // SAFETY: caller must ensure the node is valid and elem not yet moved.
    unsafe fn elem_read(self: *mut Self) -> T {
        // addr_of!((*self).elem) gives a *const T to the field.
        // ptr::read bitwise-copies the T without dropping the source.
        std::ptr::addr_of!((*self).elem).read()
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut cur = self.head;
        while let Some(ptr) = cur {
            unsafe {
                let node = ptr.as_ptr();
                // Save next BEFORE we destroy the node.
                cur = (*node).next;
                // Drop elem in place — runs T's destructor properly.
                std::ptr::drop_in_place(std::ptr::addr_of_mut!((*node).elem));
                // Deallocate the node's memory.
                Node::dealloc(ptr);
            }
        }
    }
}

// ========== Owned iterator ==========
struct LinkedListIntoIter<T> {
    list: LinkedList<T>,
}

impl<T> Iterator for LinkedListIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.size, Some(self.list.size))
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = LinkedListIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        LinkedListIntoIter { list: self }
    }
}

// ========== Shared borrowed iterator ==========
struct LinkedListIter<'a, T> {
    // *const: we never mutate through this pointer.
    // Option wraps it so None == end-of-list.
    next: Option<NonNull<Node<T>>>,
    size: usize,
    // Ties this iterator to the list's shared borrow lifetime 'a.
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for LinkedListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|ptr| unsafe {
            // as_ref() gives &Node<T> with lifetime 'a — correct because
            // the list is borrowed for 'a and we hold PhantomData<&'a T>.
            let node = ptr.as_ref();
            self.next = node.next;
            self.size -= 1;
            &node.elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = LinkedListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        LinkedListIter {
            next: self.head,
            size: self.size,
            _marker: PhantomData,
        }
    }
}

// ========== Mutable borrowed iterator ==========
struct LinkedListIterMut<'a, T> {
    next: Option<NonNull<Node<T>>>,
    size: usize,
    // PhantomData<&'a mut T>: this iterator exclusively borrows T for 'a.
    // Prevents a second mutable iterator from being created concurrently.
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for LinkedListIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|mut ptr| unsafe {
            // as_mut() gives &mut Node<T> with lifetime 'a.
            // Exclusive access is guaranteed by &mut self and PhantomData.
            let node = ptr.as_mut();
            self.next = node.next;
            self.size -= 1;
            &mut node.elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = LinkedListIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        LinkedListIterMut {
            next: self.head,
            size: self.size,
            _marker: PhantomData,
        }
    }
}

// ========== Tests ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut list: LinkedList<i32> = LinkedList::new();
        assert_eq!(list.pop(), None);
        list.push(1); list.push(2); list.push(3);
        assert_eq!(list.size(), 3);
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
        assert_eq!(list.size(), 0);
    }

    #[test]
    fn peek() {
        let mut list: LinkedList<i32> = LinkedList::new();
        assert_eq!(list.peek(), None);
        list.push(42);
        assert_eq!(list.peek(), Some(&42));
        *list.peek_mut().unwrap() = 99;
        assert_eq!(list.peek(), Some(&99));
    }

    #[test]
    fn into_iter() {
        let mut list = LinkedList::new();
        list.push(1); list.push(2); list.push(3);
        let vals: Vec<_> = list.into_iter().collect();
        assert_eq!(vals, vec![3, 2, 1]);
    }

    #[test]
    fn iter() {
        let mut list = LinkedList::new();
        list.push(1); list.push(2); list.push(3);
        let vals: Vec<i32> = (&list).into_iter().copied().collect();
        assert_eq!(vals, vec![3, 2, 1]);
        // list still usable after shared borrow
        assert_eq!(list.size(), 3);
    }

    #[test]
    fn iter_mut() {
        let mut list = LinkedList::new();
        list.push(1); list.push(2); list.push(3);
        for v in &mut list { *v *= 10; }
        let vals: Vec<i32> = (&list).into_iter().copied().collect();
        assert_eq!(vals, vec![30, 20, 10]);
    }

    #[test]
    fn drop_with_heap_type() {
        // String has a non-trivial Drop — verifies drop_in_place is correct.
        let mut list = LinkedList::new();
        list.push(String::from("hello"));
        list.push(String::from("world"));
        // Drops here — Miri would catch any leak or double-free.
    }

    #[test]
    fn large_list_no_stack_overflow() {
        let mut list = LinkedList::new();
        for i in 0..100_000 { list.push(i); }
        // Iterative Drop — no recursion, no stack overflow.
    }
}*/