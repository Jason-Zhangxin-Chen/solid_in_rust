use std::collections::VecDeque;
use std::marker::PhantomData;
use std::ptr::NonNull;

// ==================== Link type ============================================
type Link<T> = Option<NonNull<Node<T>>>;

// ==================== Node =================================================
struct Node<T> {
    elem:  T,
    left:  Link<T>,
    right: Link<T>,
}

impl<T> Node<T> {

    fn allocate(elem: T) -> NonNull<Self> {
        let boxed = Box::new(Node { elem, left: None, right: None });
        unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) }
    }

    fn deallocate(ptr: NonNull<Self>) -> Box<Self> {
        unsafe { Box::from_raw(ptr.as_ptr()) }
    }
}

// ==================== free_subtree helper ==================================
/// Frees an entire subtree using iterative post-order traversal.
/// Safety: `root` must be a valid, exclusively-owned pointer.
unsafe fn free_subtree<T>(root: NonNull<Node<T>>) {
    let mut stack = vec![root];
    while let Some(&top) = stack.last() {
        let node = &mut *top.as_ptr();
        if let Some(left) = node.left.take() {
            stack.push(left);
            continue;
        }
        if let Some(right) = node.right.take() {
            stack.push(right);
            continue;
        }
        stack.pop();
        let _ = Node::deallocate(top);
    }
}

// ==================== BinaryTree ===========================================
pub struct BinaryTree<T> {
    root:    Link<T>,
    len:     usize,
    _marker: PhantomData<T>,
}

// --- Drop ------------------------------------------------------------------
impl<T> Drop for BinaryTree<T> {
    fn drop(&mut self) {
        if let Some(root) = self.root.take() {
            // Safety: root is valid and exclusively owned by self.
            unsafe { free_subtree(root) };
        }
    }
}

// --- Core methods (no trait bounds needed) ---------------------------------
impl<T> BinaryTree<T> {
    pub fn new() -> Self {
        Self { root: None, len: 0, _marker: PhantomData }
    }

    pub fn len(&self)      -> usize { self.len }
    pub fn is_empty(&self) -> bool  { self.len == 0 }

    // -------- depth (recursive) --------
    pub fn depth(&self) -> usize {
        Self::depth_recursive(&self.root)
    }

    fn depth_recursive(link: &Link<T>) -> usize {
        match link {
            None       => 0,
            Some(ptr)  => {
                // Safety: ptr is valid.
                let node = unsafe { &*ptr.as_ptr() };
                1 + Self::depth_recursive(&node.left)
                    .max(Self::depth_recursive(&node.right))
            }
        }
    }

    // -------- depth (iterative) --------
    pub fn depth_iter(&self) -> usize {
        let mut max_depth = 0;
        let mut stack: Vec<(NonNull<Node<T>>, usize)> = Vec::new();
        if let Some(root) = self.root {
            stack.push((root, 1));
        }
        while let Some((ptr, height)) = stack.pop() {
            max_depth = max_depth.max(height);
            // Safety: ptr is valid.
            let node = unsafe { &*ptr.as_ptr() };
            if let Some(left)  = node.left  { stack.push((left,  height + 1)); }
            if let Some(right) = node.right { stack.push((right, height + 1)); }
        }
        max_depth
    }

    // -------- is_balanced (recursive) --------
    pub fn is_balanced(&self) -> bool {
        fn check<T>(link: &Link<T>) -> (bool, usize) {
            match link {
                None      => (true, 0),
                Some(ptr) => {
                    // Safety: ptr is valid.
                    let node = unsafe { &*ptr.as_ptr() };
                    let (l_ok, l_h) = check(&node.left);
                    let (r_ok, r_h) = check(&node.right);
                    (l_ok && r_ok && l_h.abs_diff(r_h) <= 1, 1 + l_h.max(r_h))
                }
            }
        }
        check(&self.root).0
    }

    // -------- is_balanced (iterative) --------
    pub fn is_balanced_iter(&self) -> bool {
        use std::collections::HashMap;
        let mut stack: Vec<NonNull<Node<T>>> = Vec::new();
        let mut current = self.root;
        let mut last_visited: Option<NonNull<Node<T>>> = None;
        let mut heights: HashMap<NonNull<Node<T>>, usize> = HashMap::new();

        while current.is_some() || !stack.is_empty() {
            while let Some(node) = current {
                stack.push(node);
                current = unsafe { (*node.as_ptr()).left };
            }
            if let Some(&top) = stack.last() {
                let right = unsafe { (*top.as_ptr()).right };
                if right.is_some() && right != last_visited {
                    current = right;
                } else {
                    let left_h  = unsafe { (*top.as_ptr()).left }
                        .and_then(|l| heights.get(&l)).copied().unwrap_or(0);
                    let right_h = unsafe { (*top.as_ptr()).right }
                        .and_then(|r| heights.get(&r)).copied().unwrap_or(0);
                    if left_h.abs_diff(right_h) > 1 { return false; }
                    heights.insert(top, 1 + left_h.max(right_h));
                    last_visited = Some(stack.pop().unwrap());
                }
            }
        }
        true
    }

    // -------- dfs (mutable visit) --------
    // FIX: store NonNull in the stack; create &mut only at the use site
    //      to avoid holding multiple simultaneous &mut references.
    pub fn dfs<F>(&mut self, mut f: F)
    where F: FnMut(&mut T) {
        let mut stack: Vec<NonNull<Node<T>>> = Vec::new();
        if let Some(root) = self.root { stack.push(root); }
        while let Some(ptr) = stack.pop() {
            // Safety: ptr is valid; &mut self ensures exclusive access.
            // &mut node lives only for this iteration — no aliasing.
            let node = unsafe { &mut *ptr.as_ptr() };
            f(&mut node.elem);
            if let Some(right) = node.right { stack.push(right); }
            if let Some(left)  = node.left  { stack.push(left);  }
        }
    }

    // -------- bfs (mutable visit) --------
    pub fn bfs<F>(&mut self, mut f: F)
    where F: FnMut(&mut T) {
        let mut queue: VecDeque<NonNull<Node<T>>> = VecDeque::new();
        if let Some(root) = self.root { queue.push_back(root); }
        while let Some(ptr) = queue.pop_front() {
            // Safety: same as dfs above.
            let node = unsafe { &mut *ptr.as_ptr() };
            f(&mut node.elem);
            if let Some(left)  = node.left  { queue.push_back(left);  }
            if let Some(right) = node.right { queue.push_back(right); }
        }
    }
}

// --- Default ---------------------------------------------------------------
impl<T: Ord> Default for BinaryTree<T> {
    fn default() -> Self { Self::new() }
}

// --- Ord-only methods (insert / search, no Clone needed) -------------------
impl<T: Ord> BinaryTree<T> {

    // -------- insert (recursive) --------
    // Returns true if the element was newly inserted.
    // Duplicates are silently ignored and len is NOT incremented.
    pub fn insert(&mut self, elem: T) -> bool {
        let inserted = Self::insert_recursive(&mut self.root, elem);
        if inserted { self.len += 1; }
        inserted
    }

    fn insert_recursive(link: &mut Link<T>, elem: T) -> bool {
        match link {
            None => { *link = Some(Node::allocate(elem)); true }
            Some(ptr) => {
                // Safety: ptr is valid.
                let node = unsafe { &mut *ptr.as_ptr() };
                match elem.cmp(&node.elem) {
                    std::cmp::Ordering::Less    => Self::insert_recursive(&mut node.left,  elem),
                    std::cmp::Ordering::Greater => Self::insert_recursive(&mut node.right, elem),
                    std::cmp::Ordering::Equal   => false,  // duplicate — do nothing
                }
            }
        }
    }

    // -------- insert (iterative) --------
    pub fn insert_iter(&mut self, elem: T) -> bool {
        let mut current = &mut self.root;
        while let Some(ptr) = current {
            // Safety: ptr is valid.
            let node = unsafe { &mut *ptr.as_ptr() };
            match elem.cmp(&node.elem) {
                std::cmp::Ordering::Less    => current = &mut node.left,
                std::cmp::Ordering::Greater => current = &mut node.right,
                std::cmp::Ordering::Equal   => return false,
            }
        }
        *current = Some(Node::allocate(elem));
        self.len += 1;
        true
    }

    // -------- search (recursive) --------
    pub fn search(&self, elem: &T) -> bool {
        Self::search_recursive(&self.root, elem)
    }

    fn search_recursive(link: &Link<T>, elem: &T) -> bool {
        match link {
            None      => false,
            Some(ptr) => {
                // Safety: ptr is valid.
                let node = unsafe { &*ptr.as_ptr() };
                match node.elem.cmp(elem) {
                    std::cmp::Ordering::Equal   => true,
                    std::cmp::Ordering::Less    => Self::search_recursive(&node.right, elem),
                    std::cmp::Ordering::Greater => Self::search_recursive(&node.left,  elem),
                }
            }
        }
    }

    // -------- search (iterative) --------
    pub fn search_iter(&self, elem: &T) -> bool {
        let mut current = &self.root;
        while let Some(ptr) = current {
            // Safety: ptr is valid.
            let node = unsafe { &*ptr.as_ptr() };
            match node.elem.cmp(elem) {
                std::cmp::Ordering::Equal   => return true,
                std::cmp::Ordering::Less    => current = &node.right,
                std::cmp::Ordering::Greater => current = &node.left,
            }
        }
        false
    }
}

// --- Ord + Clone methods (traversals returning Vec) ------------------------
impl<T: Ord + Clone> BinaryTree<T> {

    // -------- inorder --------
    pub fn inorder(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive(link: &Link<T>, result: &mut Vec<T>) {
        if let Some(ptr) = link {
            let node = unsafe { &*ptr.as_ptr() };
            Self::inorder_recursive(&node.left, result);
            result.push(node.elem.clone());
            Self::inorder_recursive(&node.right, result);
        }
    }

    pub fn inorder_iter(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        let mut stack  = Vec::new();
        let mut current = self.root;
        while current.is_some() || !stack.is_empty() {
            while let Some(node) = current {
                stack.push(node);
                current = unsafe { (*node.as_ptr()).left };
            }
            if let Some(node) = stack.pop() {
                let node_ref = unsafe { &*node.as_ptr() };
                result.push(node_ref.elem.clone());
                current = node_ref.right;
            }
        }
        result
    }

    // -------- preorder --------
    pub fn preorder(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        Self::preorder_recursive(&self.root, &mut result);
        result
    }

    fn preorder_recursive(link: &Link<T>, result: &mut Vec<T>) {
        if let Some(ptr) = link {
            let node = unsafe { &*ptr.as_ptr() };
            result.push(node.elem.clone());
            Self::preorder_recursive(&node.left,  result);
            Self::preorder_recursive(&node.right, result);
        }
    }

    pub fn preorder_iter(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        let mut stack  = Vec::new();
        if let Some(node) = self.root { stack.push(node); }
        while let Some(ptr) = stack.pop() {
            let node = unsafe { &*ptr.as_ptr() };
            result.push(node.elem.clone());
            if let Some(right) = node.right { stack.push(right); }
            if let Some(left)  = node.left  { stack.push(left);  }
        }
        result
    }

    // -------- postorder --------
    pub fn postorder(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        Self::postorder_recursive(&self.root, &mut result);
        result
    }

    fn postorder_recursive(link: &Link<T>, result: &mut Vec<T>) {
        if let Some(ptr) = link {
            let node = unsafe { &*ptr.as_ptr() };
            Self::postorder_recursive(&node.left,  result);
            Self::postorder_recursive(&node.right, result);
            result.push(node.elem.clone());
        }
    }

    pub fn postorder_iter(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len);
        let mut stack: Vec<NonNull<Node<T>>> = Vec::new();
        let mut current = self.root;
        let mut last_visited: Option<NonNull<Node<T>>> = None;
        while current.is_some() || !stack.is_empty() {
            while let Some(node) = current {
                stack.push(node);
                current = unsafe { (*node.as_ptr()).left };
            }
            if let Some(&top) = stack.last() {
                let right = unsafe { (*top.as_ptr()).right };
                if right.is_some() && right != last_visited {
                    current = right;
                } else {
                    let top_ref = unsafe { &*top.as_ptr() };
                    result.push(top_ref.elem.clone());
                    last_visited = Some(stack.pop().unwrap());
                }
            }
        }
        result
    }

    pub fn iter(&self) -> Iter<T> {
        let mut stack = Vec::new();
        let mut cur = self.root;
        while let Some(ptr) = cur {
            let node_ref = unsafe { &*ptr.as_ptr() };
            stack.push(node_ref);
            cur = node_ref.left;
        }
        Iter { stack }
    }
}

// ==================== Iter (borrowed, in-order) ============================
pub struct Iter<'a, T> {
    stack: Vec<&'a Node<T>>,
}

impl<'a, T: Ord + Clone> IntoIterator for &'a BinaryTree<T> {
    type Item     = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, T: Ord + Clone> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let mut cur = node.right;
        while let Some(ptr) = cur {
            let right_ref = unsafe { &*ptr.as_ptr() };
            self.stack.push(right_ref);
            cur = right_ref.left;
        }
        Some(&node.elem)
    }
}

// ==================== IntoIter (consuming, in-order) =======================
pub struct IntoIter<T> {
    stack: Vec<NonNull<Node<T>>>,
}

impl<T: Ord + Clone> IntoIterator for BinaryTree<T> {
    type Item     = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let mut stack = Vec::new();
        let mut cur = self.root.take();  // root = None so BinaryTree::drop is a no-op
        while let Some(ptr) = cur {
            // Take left so the stack's left links are cleared;
            // right links remain intact and are handled in next() / Drop.
            let left = unsafe { (*ptr.as_ptr()).left.take() };
            cur = left;
            stack.push(ptr);
        }
        IntoIter { stack }
    }
}

impl<T: Ord + Clone> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.stack.pop()?;
        // Take the right child and push its left spine.
        let mut cur = unsafe { (*ptr.as_ptr()).right.take() };
        while let Some(n) = cur {
            let left = unsafe { (*n.as_ptr()).left.take() };
            cur = left;
            self.stack.push(n);
        }
        // Clone the element, then free this node.
        let elem = unsafe { &*ptr.as_ptr() }.elem.clone();
        // Safety: ptr was allocated via Box::new; we now own it exclusively.
        let _ = unsafe { Box::from_raw(ptr.as_ptr()) };
        Some(elem)
    }
}

// FIX: the original Drop only freed nodes in the stack but NOT their right
// subtrees, causing a memory leak when iteration was abandoned early.
impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for ptr in self.stack.drain(..) {
            // Each ptr's left link was already taken (set to None) during
            // into_iter() or a previous next() call.
            // Its right subtree is still fully intact and must be freed.
            unsafe { free_subtree(ptr) };
        }
    }
}

// ==================== Tests ================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn build() -> BinaryTree<i32> {
        //        4
        //       / \
        //      2   6
        //     / \ / \
        //    1  3 5  7
        let mut t = BinaryTree::new();
        for v in [4, 2, 6, 1, 3, 5, 7] { t.insert(v); }
        t
    }

    #[test]
    fn len_and_search() {
        let t = build();
        assert_eq!(t.len(), 7);
        assert!(t.search(&4));
        assert!(t.search_iter(&1));
        assert!(!t.search(&99));
    }

    #[test]
    fn no_duplicate_insert() {
        let mut t = BinaryTree::new();
        assert!(t.insert(5));
        assert!(!t.insert(5));  // duplicate — returns false
        assert_eq!(t.len(), 1); // len stays at 1
    }

    #[test]
    fn depth_both_agree() {
        let t = build();
        assert_eq!(t.depth(),      3);
        assert_eq!(t.depth_iter(), 3);
    }

    #[test]
    fn balanced() {
        let t = build();
        assert!(t.is_balanced());
        assert!(t.is_balanced_iter());
        let mut skewed = BinaryTree::new();
        for v in [1, 2, 3, 4, 5] { skewed.insert(v); }
        assert!(!skewed.is_balanced());
        assert!(!skewed.is_balanced_iter());
    }

    #[test]
    fn inorder_sorted() {
        let t = build();
        assert_eq!(t.inorder(),      vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(t.inorder_iter(), vec![1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn preorder_both_agree() {
        let t = build();
        assert_eq!(t.preorder(), t.preorder_iter());
    }

    #[test]
    fn postorder_both_agree() {
        let t = build();
        assert_eq!(t.postorder(),      vec![1, 3, 2, 5, 7, 6, 4]);
        assert_eq!(t.postorder_iter(), vec![1, 3, 2, 5, 7, 6, 4]);
    }

    #[test]
    fn dfs_doubles_values() {
        let mut t = build();
        t.dfs(|v| *v *= 2);
        assert_eq!(t.inorder(), vec![2, 4, 6, 8, 10, 12, 14]);
    }

    #[test]
    fn bfs_doubles_values() {
        let mut t = build();
        t.bfs(|v| *v *= 2);
        assert_eq!(t.inorder(), vec![2, 4, 6, 8, 10, 12, 14]);
    }

    #[test]
    fn iter_borrowed() {
        let t = build();
        let vals: Vec<_> = t.iter().copied().collect();
        assert_eq!(vals, vec![1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(t.len(), 7); // still usable
    }

    #[test]
    fn into_iter_consumes() {
        let t = build();
        let vals: Vec<_> = t.into_iter().collect();
        assert_eq!(vals, vec![1, 2, 3, 4, 5, 6, 7]);
    }

    // Verifies IntoIter::Drop doesn't leak when iteration stops early.
    // Run with `cargo test -- --nocapture` under valgrind or miri to confirm.
    #[test]
    fn into_iter_partial_no_leak() {
        let t = build();
        let mut iter = t.into_iter();
        let _ = iter.next(); // consume only one element, then drop
    }

    #[test]
    fn drop_no_stack_overflow() {
        let mut t = BinaryTree::new();
        for v in 0..100_000 { t.insert(v); }
        drop(t);
    }

    #[test]
    fn into_iter_drop_no_stack_overflow() {
        let mut t = BinaryTree::new();
        for v in 0..100_000 { t.insert(v); }
        let mut iter = t.into_iter();
        let _ = iter.next();
        drop(iter); // must not overflow or leak
    }
}