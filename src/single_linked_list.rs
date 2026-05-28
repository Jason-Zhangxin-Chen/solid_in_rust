// Singly linked list, implements a generic singly linked list via Box<T>.

struct Node<T> {
    elem: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(elem: T, next: Option<Box<Node<T>>>) -> Self {
        Self { elem, next }
    }
}

struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    tail: Option<Box<Node<T>>>,
    size: usize,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self{
            head: None,
            tail: None,
            size: 0,
        }
    }
    
    pub fn push_back(&mut self, elem: T) {
        // tail's next -> new node.
        let new_node = Box::new(Node::new(elem, None));
        if let Some(ref mut tail) = self.tail {
            tail.next = Some(new_node);
        } else {
            self.tail = Some(new_node);
        }
        self.size += 1;
    }
    
    pub fn push_front(&mut self, elem: T) {
        let new_node = Box::new(Node::new(elem, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            self.head = old_head.next;
            self.size -= 1;
            old_head.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
            // boxed node dropped from here.
        }
    }
}

// ---------Iterators --------------
// owned iterator which consumes the ownership of underlying linked list.
struct LinkedListIntoIter<T> {
    list: LinkedList<T>,
}

impl <T> Iterator for LinkedListIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // let the underlying pop function consumes the data.
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.size, Some(self.list.size))
    }
}

impl <T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = LinkedListIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        LinkedListIntoIter{ list: self }
    }
}

// borrowed iterator which immutably borrowed the item from LinkedList<T>
struct LinkedListIter<'a, T> {
    next: Option<&'a Node<T>>,
    size: usize,
}

impl <'a, T> Iterator for LinkedListIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|head| {
            self.size -= 1;
            self.next = head.next.as_deref();
            &head.elem
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl <'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = LinkedListIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        LinkedListIter{ next: self.head.as_deref(), size: self.size }
    }
}

// mutable borrowed iterator
struct LinkedListIterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
    size: usize,
}

impl <'a, T> Iterator for LinkedListIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|head| {
            self.size -= 1;
            self.next = head.next.as_deref_mut();
            &mut head.elem
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl <'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = LinkedListIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        LinkedListIterMut{ next: self.head.as_deref_mut(), size: self.size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {}
}