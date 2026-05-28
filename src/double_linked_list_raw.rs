use std::ptr::NonNull;
use std::marker::PhantomData;

struct Node<T> {
    elem: T,
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    // Box::new allocates on the heap, then we immediately take the raw pointer.
    // Box::into_raw transfers ownership OUT of Box — nothing is dropped here.
    // We are now responsible for freeing this memory manually.
    fn allocate(elem: T) -> NonNull<Self> {
        let boxed = Box::new(Node {
            elem,
            next: None,
            prev: None,
        });
        // SAFETY: Box::into_raw never returns null
        unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) }
    }

    // Reclaim ownership back into a Box, so Rust drops it normally.
    // Caller must guarantee the pointer is valid and exclusively owned.
    unsafe fn deallocate(ptr: NonNull<Self>) -> Box<Self> {
        Box::from_raw(ptr.as_ptr())
    }
}

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    // Tells the compiler: LinkedList<T> logically OWNS values of T
    // Without this, the compiler doesn't know how to handle drop order,
    // variance, or lifetime constraints over T
    _marker: PhantomData<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let mut new_node = Node::allocate(elem);

        match self.head {
            None => {
                // Empty list — new node is both head and tail
                self.tail = Some(new_node);
            }
            Some(mut old_head) => {
                unsafe {
                    // new_node.next → old_head
                    new_node.as_mut().next = Some(old_head);
                    // old_head.prev → new_node
                    old_head.as_mut().prev = Some(new_node);
                }
            }
        }

        self.head = Some(new_node);
        self.len += 1;
    }

    pub fn push_back(&mut self, elem: T) {
        let mut new_node = Node::allocate(elem);

        match self.tail {
            None => {
                // Empty list
                self.head = Some(new_node);
            }
            Some(mut old_tail) => {
                unsafe {
                    // old_tail.next → new_node
                    old_tail.as_mut().next = Some(new_node);
                    // new_node.prev → old_tail
                    new_node.as_mut().prev = Some(old_tail);
                }
            }
        }

        self.tail = Some(new_node);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|old_head| unsafe {
            // Re-box to take ownership — this will be dropped at end of scope
            let old_head = Node::deallocate(old_head);

            match old_head.next {
                None => {
                    // Was the only node
                    self.tail = None;
                    self.head = None;
                }
                Some(mut new_head) => {
                    // Sever the backward link — new head has no predecessor
                    new_head.as_mut().prev = None;
                    self.head = Some(new_head);
                }
            }

            self.len -= 1;
            old_head.elem  // Box drops the node here, elem is moved out
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.map(|old_tail| unsafe {
            let old_tail = Node::deallocate(old_tail);

            match old_tail.prev {
                None => {
                    // Was the only node
                    self.head = None;
                    self.tail = None;
                }
                Some(mut new_tail) => {
                    // Sever the forward link
                    new_tail.as_mut().next = None;
                    self.tail = Some(new_tail);
                }
            }

            self.len -= 1;
            old_tail.elem
        })
    }
}


impl<T> LinkedList<T> {
    pub fn peek_front(&self) -> Option<&T> {
        // SAFETY: head is valid as long as &self is alive
        self.head.map(|node| unsafe { &node.as_ref().elem })
    }

    pub fn peek_back(&self) -> Option<&T> {
        self.tail.map(|node| unsafe { &node.as_ref().elem })
    }

    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        // SAFETY: &mut self guarantees exclusive access
        self.head.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn peek_back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}

// --- Immutable iterator ---

pub struct Iter<'a, T> {
    next: Option<NonNull<Node<T>>>,
    _marker: PhantomData<&'a T>,  // ties iterator lifetime to the list
}

impl<T> LinkedList<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head, _marker: PhantomData }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| unsafe {
            let node = node.as_ref();
            self.next = node.next;
            &node.elem
        })
    }
}

// --- Mutable iterator ---

pub struct IterMut<'a, T> {
    next: Option<NonNull<Node<T>>>,
    _marker: PhantomData<&'a mut T>,
}

impl<T> LinkedList<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut { next: self.head, _marker: PhantomData }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|mut node| unsafe {
            let node = node.as_mut();
            self.next = node.next;
            &mut node.elem
        })
    }
}

// --- Consuming iterator ---

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

pub struct IntoIter<T>(LinkedList<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.0.pop_front() }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        // Pop everything — each pop re-boxes the node and drops it cleanly.
        // Without this, the raw pointers just vanish and all nodes leak.
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_back(),  Some(3));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn peek() {
        let mut list = LinkedList::new();
        list.push_back(10);
        list.push_back(20);

        assert_eq!(list.peek_front(), Some(&10));
        assert_eq!(list.peek_back(),  Some(&20));

        *list.peek_front_mut().unwrap() = 99;
        assert_eq!(list.peek_front(), Some(&99));
    }

    #[test]
    fn iter() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let v: Vec<_> = list.iter().collect();
        assert_eq!(v, [&1, &2, &3]);
    }

    #[test]
    fn into_iter() {
        let mut list = LinkedList::new();
        list.push_back('a');
        list.push_back('b');
        list.push_back('c');

        let v: Vec<_> = list.into_iter().collect();
        assert_eq!(v, ['a', 'b', 'c']);
    }
}