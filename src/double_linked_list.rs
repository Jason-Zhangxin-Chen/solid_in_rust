use std::cell::{Ref, RefCell};
use std::rc::{Rc, Weak};

// =============================================================================
// Node — private implementation detail
// =============================================================================
struct Node<T> {
    elem: RefCell<T>,
    next: RefCell<Option<Rc<Node<T>>>>,
    prev: RefCell<Option<Weak<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<Self> {                   // return Rc directly — avoids
        Rc::new(Self {                              // a redundant Rc::new at call sites
            elem: RefCell::new(elem),
            next: RefCell::new(None),
            prev: RefCell::new(None),
        })
    }
}

// =============================================================================
// LinkedList
// =============================================================================
pub struct LinkedList<T> {
    size: usize,
    head: Option<Rc<Node<T>>>,
    tail: Option<Rc<Node<T>>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self { size: 0, head: None, tail: None }
    }

    pub fn len(&self) -> usize { self.size }        // renamed: len() is the Rust convention
    pub fn is_empty(&self) -> bool { self.size == 0 }

    // -------------------------------------------------------------------------
    // Push
    // -------------------------------------------------------------------------

    pub fn push_back(&mut self, elem: T) {
        let new_node = Node::new(elem);

        match self.tail.take() {
            Some(old_tail) => {
                // old_tail.next → new_node
                *old_tail.next.borrow_mut() = Some(Rc::clone(&new_node));
                // new_node.prev → old_tail (Weak to avoid cycle)
                *new_node.prev.borrow_mut() = Some(Rc::downgrade(&old_tail));
                self.tail = Some(new_node);
            }
            None => {
                // Empty list — both head and tail point at the single node
                self.head = Some(Rc::clone(&new_node));
                self.tail = Some(new_node);
            }
        }
        self.size += 1;
    }

    pub fn push_front(&mut self, elem: T) {
        let new_node = Node::new(elem);

        match self.head.take() {
            Some(old_head) => {
                // new_node.next → old_head
                *new_node.next.borrow_mut() = Some(Rc::clone(&old_head));
                // old_head.prev → new_node (Weak)
                *old_head.prev.borrow_mut() = Some(Rc::downgrade(&new_node));
                self.head = Some(new_node);
            }
            None => {
                self.head = Some(Rc::clone(&new_node));
                self.tail = Some(new_node);
            }
        }
        self.size += 1;
    }

    // -------------------------------------------------------------------------
    // Pop
    // -------------------------------------------------------------------------

    pub fn pop_front(&mut self) -> Option<T> {
        let old_head = self.head.take()?;

        match old_head.next.borrow_mut().take() {
            Some(next) => {
                // Sever the back-pointer so old_head has no references left
                *next.prev.borrow_mut() = None;
                self.head = Some(next);
            }
            None => {
                // List is now empty — tail still holds an Rc to old_head;
                // take() it so old_head's strong count drops to 1 (only us)
                self.tail = None;
            }
        }
        self.size -= 1;
        // try_unwrap succeeds only when we hold the sole strong reference.
        // If it fails, there is a bug elsewhere (an iterator is still alive, etc.)
        Some(Rc::try_unwrap(old_head).ok().expect("pop_front: node still shared").elem.into_inner())
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let old_tail = self.tail.take()?;

        // Walk the Weak back-pointer to find the new tail
        let prev = old_tail.prev
            .borrow_mut()
            .take()                                // clear the Weak from old_tail
            .and_then(|w| w.upgrade());             // None if it was already dead

        match prev {
            Some(prev_node) => {
                // Disconnect old_tail from the list
                *prev_node.next.borrow_mut() = None;
                self.tail = Some(prev_node);
            }
            None => {
                // old_tail was the only node
                self.head = None;
            }
        }
        self.size -= 1;
        Some(Rc::try_unwrap(old_tail).ok().expect("pop_back: node still shared").elem.into_inner())
    }

    // -------------------------------------------------------------------------
    // Peek  (new — was missing)
    // -------------------------------------------------------------------------

    pub fn peek_front(&self) -> Option<Ref<'_, T>> {
        self.head.as_ref().map(|node| node.as_ref().elem.borrow())
    }

    pub fn peek_back(&self) -> Option<Ref<'_, T>> {
        self.tail.as_ref().map(|node| node.as_ref().elem.borrow())
    }

    // Note: returning &T from inside Rc<RefCell<>> normally requires unsafe.
    // The safe alternative is to return a clone (add T: Clone bound).
    // A raw-pointer implementation (NonNull) avoids this entirely.

    // -------------------------------------------------------------------------
    // Iter helpers
    // -------------------------------------------------------------------------

    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.head.clone(),
            back:  self.tail.clone(),
            len:   self.size,
            _marker: Default::default(),
        }
    }
}

// =============================================================================
// Drop — iterative, avoids stack overflow on long lists
// =============================================================================
impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        // pop_front breaks the Rc chain one node at a time — no recursion
        while self.pop_front().is_some() {}
    }
}

// =============================================================================
// Default
// =============================================================================
impl<T> Default for LinkedList<T> {
    fn default() -> Self { Self::new() }
}

// =============================================================================
// Owned iterator  (consumes the list)
// =============================================================================
pub struct IntoIter<T>(LinkedList<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.0.pop_front() }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.size, Some(self.0.size))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> { self.0.pop_back() }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> IntoIterator for LinkedList<T> {
    type Item     = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter { IntoIter(self) }
}

// =============================================================================
// Borrowed iterator — clones Rc handles, returns &'a T via raw pointer
// =============================================================================
// See the explanation in the previous session: Rc<RefCell<>> + &'a T requires
// either unsafe (raw pointer) or T: Clone. We use the raw pointer approach
// with a PhantomData lifetime anchor so the borrow checker tracks 'a correctly.

use std::marker::PhantomData;

pub struct Iter<'a, T> {
    front: Option<Rc<Node<T>>>,
    back:  Option<Rc<Node<T>>>,
    len:   usize,
    // without this PhantomData marker, the compiler does not know how long this Iter
    // live, as the front/back does not contain the reference at all. The PhantomData
    // marker here just let the compiler know the object of Iter should bind with the
    // underlying T as it refer to T with such lifetime 'a, as <&'a T> specified.
    _marker: PhantomData<&'a T>,
}

// Override the derived impl so PhantomData doesn't add spurious bounds
impl<'a, T> LinkedList<T> {
    // already defined above — re-stated here for visibility
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.front.take().map(|node| {
            // Extend the reference lifetime from the Rc's scope to 'a.
            // Sound because the list is borrowed for 'a, keeping every node alive.
            let elem: &'a T = unsafe { &*node.elem.as_ptr() };

            // Advance — clone the next Rc out before the guard drops
            self.front = node.next.borrow().clone();
            self.len  -= 1;
            elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.back.take().map(|node| {
            let elem: &'a T = unsafe { &*node.elem.as_ptr() };

            // Walk backwards via Weak → upgrade → Rc
            self.back = node.prev
                .borrow()
                .as_ref()
                .and_then(|w| w.upgrade());   // Option<Rc<Node<T>>> — owned, fine
            self.len -= 1;
            elem
        })
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item     = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_front() {
        let mut l = LinkedList::new();
        l.push_front(3); l.push_front(2); l.push_front(1);
        assert_eq!(l.len(), 3);
        assert_eq!(l.pop_front(), Some(1));
        assert_eq!(l.pop_front(), Some(2));
        assert_eq!(l.pop_front(), Some(3));
        assert_eq!(l.pop_front(), None);
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn push_pop_back() {
        let mut l = LinkedList::new();
        l.push_back(1); l.push_back(2); l.push_back(3);
        assert_eq!(l.pop_back(), Some(3));
        assert_eq!(l.pop_back(), Some(2));
        assert_eq!(l.pop_back(), Some(1));
        assert_eq!(l.pop_back(), None);
    }

    #[test]
    fn mixed_push_pop() {
        let mut l = LinkedList::new();
        l.push_back(2);
        l.push_front(1);
        l.push_back(3);
        assert_eq!(l.pop_front(), Some(1));
        assert_eq!(l.pop_back(),  Some(3));
        assert_eq!(l.pop_front(), Some(2));
        assert!(l.is_empty());
    }

    #[test]
    fn peek() {
        let mut l = LinkedList::new();
        assert!(l.peek_front().is_none());
        l.push_back(10);
        l.push_back(20);
        assert_eq!(*l.peek_front().unwrap(), 10);
        assert_eq!(*l.peek_back().unwrap(),  20);
    }

    #[test]
    fn iter_forward() {
        let mut l = LinkedList::new();
        for i in 1..=4 { l.push_back(i); }
        let v: Vec<_> = l.iter().copied().collect();
        assert_eq!(v, vec![1, 2, 3, 4]);
    }

    #[test]
    fn iter_backward() {
        let mut l = LinkedList::new();
        for i in 1..=4 { l.push_back(i); }
        let v: Vec<_> = l.iter().rev().copied().collect();
        assert_eq!(v, vec![4, 3, 2, 1]);
    }

    #[test]
    fn into_iter() {
        let mut l = LinkedList::new();
        for i in 1..=3 { l.push_back(i); }
        let v: Vec<_> = l.into_iter().collect();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn into_iter_rev() {
        let mut l = LinkedList::new();
        for i in 1..=3 { l.push_back(i); }
        let v: Vec<_> = l.into_iter().rev().collect();
        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn exact_size() {
        let mut l = LinkedList::new();
        l.push_back(1); l.push_back(2);
        assert_eq!(l.iter().len(), 2);
    }

    #[test]
    fn drop_no_leak() {
        // If Drop is broken, Miri or valgrind will catch a leak here
        let mut l = LinkedList::new();
        for i in 0..10_000 { l.push_back(i); }
        drop(l); // must complete without stack overflow
    }
}