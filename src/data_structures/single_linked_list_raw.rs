use std::ptr;
use std::marker::PhantomData;

// ========== Node ==========
struct Node<T> {
    elem: T,
    next: *mut Node<T>,   // null ptr replaces Option::None
}

impl<T> Node<T> {
    // Allocate on heap and return raw pointer — caller owns the memory
    fn new(elem: T, next: *mut Node<T>) -> *mut Self {
        Box::into_raw(Box::new(Node { elem, next }))
    }
}

// ========== LinkedList ==========
struct LinkedList<T> {
    head: *mut Node<T>,
    size: usize,
    // Raw pointers are neither Send nor Sync by default, and they carry
    // no lifetime. PhantomData<T> tells the compiler this struct logically
    // owns T values, restoring correct drop-check and variance behaviour.
    _marker: PhantomData<T>,
}

// Raw pointers strip Send/Sync auto-traits, so we restore them manually.
// SAFETY: LinkedList exclusively owns its nodes — no aliasing across threads.
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: ptr::null_mut(),  // null == empty
            size: 0,
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, elem: T) {
        // Node::new boxes the value and hands back the raw pointer.
        let new_node = Node::new(elem, self.head);
        self.head = new_node;
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }
        unsafe {
            // Re-box the raw pointer so Rust will free it when `old_head`
            // drops at the end of this block.
            let old_head = Box::from_raw(self.head);
            self.head = old_head.next;
            self.size -= 1;
            Some(old_head.elem)
            // `old_head` (Box) drops here, freeing the node's memory.
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.head.is_null() {
            return None;
        }
        // SAFETY: head is non-null and valid; we produce a shared borrow
        // tied to the lifetime of &self, so no mutation can occur.
        unsafe { Some(&(*self.head).elem) }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.head.is_null() {
            return None;
        }
        // SAFETY: head is non-null and valid; exclusive borrow of &mut self
        // guarantees no other reference to this node exists.
        unsafe { Some(&mut (*self.head).elem) }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut cur = self.head;
        while !cur.is_null() {
            unsafe {
                // Re-box each node so Box::drop runs its elem's Drop and
                // then frees the allocation.  Capture next BEFORE the Box
                // is dropped, since the memory is gone after.
                let node = Box::from_raw(cur);
                cur = node.next;
                // `node` drops here — elem's Drop runs, memory freed.
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
    // *const: we promise not to mutate through this pointer.
    next: *const Node<T>,
    size: usize,
    // Ties the iterator's lifetime to the list's borrow —
    // without this, the compiler forgets the lifetime relationship.
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for LinkedListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }
        unsafe {
            // SAFETY: non-null, valid for 'a (list is borrowed for 'a),
            // and we only produce shared references.
            let node = &*self.next;
            self.next = node.next; // raw *mut coerces to *const automatically
            self.size -= 1;
            Some(&node.elem)
        }
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
            next: self.head as *const Node<T>,
            size: self.size,
            _marker: PhantomData,
        }
    }
}

// ========== Mutable borrowed iterator ==========
struct LinkedListIterMut<'a, T> {
    next: *mut Node<T>,
    size: usize,
    // PhantomData<&'a mut T> encodes: this iterator exclusively borrows T
    // values for lifetime 'a — prevents any other &/&mut alias.
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for LinkedListIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }
        unsafe {
            // SAFETY: non-null, valid for 'a, and &mut self guarantees
            // exclusive access — no two calls can alias the same node.
            let node = &mut *self.next;
            self.next = node.next;
            self.size -= 1;
            Some(&mut node.elem)
        }
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
    }

    #[test]
    fn peek() {
        let mut list: LinkedList<i32> = LinkedList::new();
        assert_eq!(list.peek(), None);
        list.push(10);
        assert_eq!(list.peek(), Some(&10));
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
        let vals: Vec<_> = (&list).into_iter().copied().collect();
        assert_eq!(vals, vec![3, 2, 1]);
    }

    #[test]
    fn iter_mut() {
        let mut list = LinkedList::new();
        list.push(1); list.push(2); list.push(3);
        for v in &mut list { *v *= 10; }
        let vals: Vec<_> = (&list).into_iter().copied().collect();
        assert_eq!(vals, vec![30, 20, 10]);
    }

    #[test]
    fn drop_doesnt_leak() {
        // If Drop is broken, Miri or valgrind will catch it.
        let mut list = LinkedList::new();
        for i in 0..1000 { list.push(i); }
        // list drops here — all 1000 nodes must be freed without stack overflow
    }
}