use std::collections::VecDeque;

type Link<T> = Option<Box<Node<T>>>;

// ==================== Node (private implementation detail) ====================

struct Node<T> {
    elem: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Box<Self> {
        Box::new(Node { elem, left: None, right: None })
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        let mut stack: Vec<Box<Node<T>>> = Vec::new();
        if let Some(left)  = self.left.take()  { stack.push(left); }
        if let Some(right) = self.right.take() { stack.push(right); }
        while let Some(mut node) = stack.pop() {
            if let Some(left)  = node.left.take()  { stack.push(left);  }
            if let Some(right) = node.right.take() { stack.push(right); }
        }
    }
}

// ==================== BinaryTree (public API) ====================

pub struct BinaryTree<T> {
    root: Link<T>,
    len: usize,
}

impl<T: Ord + Clone> BinaryTree<T> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    // -------- insert (recursive) --------
    pub fn insert(&mut self, elem: T) {
        Self::insert_recursive(&mut self.root, elem);
        self.len += 1;
    }

    fn insert_recursive(link: &mut Link<T>, elem: T) {
        match link {
            None => *link = Some(Node::new(elem)),
            Some(node) => {
                if elem.lt(&node.elem) {
                    Self::insert_recursive(&mut node.left, elem);
                } else {
                    Self::insert_recursive(&mut node.right, elem);
                }
            }
        }
    }

    // -------- insert (iterative) --------
    pub fn insert_iter(&mut self, elem: T) {
        let mut current = &mut self.root;
        while let Some(node) = current {
            if elem.lt(&node.elem) {
                current = &mut node.left;
            } else {
                current = &mut node.right;
            }
        }
        *current = Some(Node::new(elem));
        self.len += 1;
    }

    // -------- search (recursive) --------
    pub fn search(&self, elem: &T) -> bool {
        Self::search_recursive(&self.root, elem)
    }

    fn search_recursive(link: &Link<T>, elem: &T) -> bool {
        match link {
            None => false,
            Some(node) => match node.elem.cmp(elem) {
                std::cmp::Ordering::Equal   => true,
                std::cmp::Ordering::Less    => Self::search_recursive(&node.right, elem),
                std::cmp::Ordering::Greater => Self::search_recursive(&node.left,  elem),
            }
        }
    }

    // -------- search (iterative) --------
    pub fn search_iter(&self, elem: &T) -> bool {
        let mut current = &self.root;
        while let Some(node) = current {
            match node.elem.cmp(elem) {
                std::cmp::Ordering::Equal   => return true,
                std::cmp::Ordering::Less    => current = &node.right,
                std::cmp::Ordering::Greater => current = &node.left,
            }
        }
        false
    }

    // -------- depth (recursive) --------
    pub fn depth(&self) -> usize {
        Self::depth_recursive(&self.root)
    }

    fn depth_recursive(link: &Link<T>) -> usize {
        match link {
            None => 0,
            Some(node) => 1 + Self::depth_recursive(&node.left)
                .max(Self::depth_recursive(&node.right)),
        }
    }

    // -------- depth (iterative) --------
    pub fn depth_iter(&self) -> usize {
        let mut max_depth = 0;
        let mut stack = Vec::new();
        if let Some(node) = &self.root {
            stack.push((node.as_ref(), 1usize));
        }
        while let Some((node, height)) = stack.pop() {
            max_depth = max_depth.max(height);
            if let Some(left)  = &node.left  { stack.push((left.as_ref(),  height + 1)); }
            if let Some(right) = &node.right { stack.push((right.as_ref(), height + 1)); }
        }
        max_depth
    }

    // -------- is_balanced --------
    pub fn is_balanced(&self) -> bool {
        fn check<T>(link: &Link<T>) -> (bool, usize) {
            match link {
                None => (true, 0),
                Some(node) => {
                    let (l_ok, l_depth) = check(&node.left);
                    let (r_ok, r_depth) = check(&node.right);
                    let balanced = l_ok && r_ok && l_depth.abs_diff(r_depth) <= 1;
                    (balanced, 1 + l_depth.max(r_depth))
                }
            }
        }
        check(&self.root).0
    }

    pub fn is_balanced_iter(&self) -> bool {
        use std::collections::HashMap;

        let mut stack: Vec<&Node<T>> = Vec::new();
        let mut current = self.root.as_deref();
        let mut last_visited: Option<*const Node<T>> = None;

        // Maps each node's address → its computed height.
        // We populate this bottom-up (post-order), so by the time
        // we process a parent, both children's heights are already here.
        let mut heights: HashMap<*const Node<T>, usize> = HashMap::new();

        while current.is_some() || !stack.is_empty() {
            // Phase 1: push left spine
            while let Some(node) = current {
                stack.push(node);
                current = node.left.as_deref();
            }

            // Phase 2: peek — only visit when right subtree is done
            if let Some(&top) = stack.last() {
                let right     = top.right.as_deref();
                let right_ptr = right.map(|r| r as *const Node<T>);

                if right.is_some() && right_ptr != last_visited {
                    // Right subtree not yet visited — descend into it first
                    current = right;
                } else {
                    // Both children are done — look up their heights
                    let left_h = top.left.as_deref()
                        .and_then(|l| heights.get(&(l as *const Node<T>)))
                        .copied()
                        .unwrap_or(0);

                    let right_h = top.right.as_deref()
                        .and_then(|r| heights.get(&(r as *const Node<T>)))
                        .copied()
                        .unwrap_or(0);

                    // Check balance at this node before going further up
                    if left_h.abs_diff(right_h) > 1 {
                        return false;  // early exit — no need to check ancestors
                    }

                    // Record this node's height for its parent to use later
                    let h = 1 + left_h.max(right_h);
                    heights.insert(top as *const Node<T>, h);

                    last_visited = Some(stack.pop().unwrap() as *const Node<T>);
                }
            }
        }
        true
    }

    // -------- traversals --------

    pub fn inorder(&self) -> Vec<T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive(link: &Link<T>, result: &mut Vec<T>) {
        if let Some(node) = link {
            Self::inorder_recursive(&node.left, result);
            result.push(node.elem.clone());
            Self::inorder_recursive(&node.right, result);
        }
    }

    pub fn inorder_iter(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut stack  = Vec::new();
        let mut current = self.root.as_deref();
        while current.is_some() || !stack.is_empty() {
            while let Some(node) = current {
                stack.push(node);
                current = node.left.as_deref();
            }
            if let Some(node) = stack.pop() {
                result.push(node.elem.clone());
                current = node.right.as_deref();
            }
        }
        result
    }

    pub fn preorder(&self) -> Vec<T> {
        let mut result = Vec::new();
        Self::preorder_recursive(&self.root, &mut result);
        result
    }

    fn preorder_recursive(link: &Link<T>, result: &mut Vec<T>) {
        if let Some(node) = link {
            result.push(node.elem.clone());
            Self::preorder_recursive(&node.left,  result);
            Self::preorder_recursive(&node.right, result);
        }
    }

    pub fn preorder_iter(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut stack  = Vec::new();
        if let Some(node) = &self.root { stack.push(node.as_ref()); }
        while let Some(node) = stack.pop() {
            result.push(node.elem.clone());
            if let Some(right) = &node.right { stack.push(right); }
            if let Some(left)  = &node.left  { stack.push(left);  }
        }
        result
    }

    pub fn postorder(&self) -> Vec<T> {
        let mut result = Vec::new();
        Self::postorder_recursive(&self.root, &mut result);
        result
    }

    fn postorder_recursive(link: &Link<T>, result: &mut Vec<T>) {
        if let Some(node) = link {
            Self::postorder_recursive(&node.left,  result);
            Self::postorder_recursive(&node.right, result);
            result.push(node.elem.clone());
        }
    }

    pub fn postorder_iter(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut stack: Vec<&Node<T>> = Vec::new();
        let mut current = self.root.as_deref();
        let mut last_visited: Option<*const Node<T>> = None;

        while current.is_some() || !stack.is_empty() {
            // go to the left deepest branch
            while let Some(node) = current {
                stack.push(node);
                current = node.left.as_deref();
            }

            // if there is any node on the top of stack,
            // which means there are unvisited right branch.
            if let Some(&top) = stack.last() {
                let right     = top.right.as_deref();
                let right_ptr = right.map(|r| r as *const Node<T>);
                // continue to visit the right child branch if it was not visited.
                // it will go through the left branch of this right child again.
                if right.is_some() && right_ptr != last_visited {
                    current = right;
                } else {
                    // right is none, or it was visited. Start to visit data item.
                    // and pop the node which is visited now.
                    result.push(top.elem.clone());
                    last_visited = Some(stack.pop().unwrap() as *const Node<T>);
                }
            }
        }
        result
    }

    // -------- dfs (visits every node, caller supplies closure) --------
    pub fn dfs<F>(&mut self, mut f: F)
    where F: FnMut(&mut T) {
        let mut stack = Vec::new();
        if let Some(node) = &mut self.root { stack.push(node.as_mut()); }
        while let Some(node) = stack.pop() {
            f(&mut node.elem);
            if let Some(right) = &mut node.right { stack.push(right); }
            if let Some(left)  = &mut node.left  { stack.push(left);  }
        }
    }

    // -------- bfs (visits every node level by level) --------
    pub fn bfs<F>(&mut self, mut f: F)
    where F: FnMut(&mut T) {
        let mut queue = VecDeque::new();
        if let Some(node) = &mut self.root { queue.push_back(node.as_mut()); }
        while let Some(node) = queue.pop_front() {
            f(&mut node.elem);
            if let Some(left)  = &mut node.left  { queue.push_back(left);  }
            if let Some(right) = &mut node.right { queue.push_back(right); }
        }
    }
}

// ==================== Iterators ====================

// -------- owned (consumes the tree, in-order) --------
pub struct IntoIter<T> {
    stack: Vec<Box<Node<T>>>,
}

impl<T: Ord + Clone> IntoIterator for BinaryTree<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let mut stack = Vec::new();
        let mut cur = self.root.take();     // take root so Drop is a no-op
        while let Some(mut node) = cur {
            cur = node.left.take();
            stack.push(node);
        }
        IntoIter { stack }
    }
}

impl<T: Ord + Clone> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.stack.pop()?;
        let mut cur = node.right.take();
        while let Some(mut n) = cur {
            cur = n.left.take();
            self.stack.push(n);
        }
        Some(node.elem.clone())
    }
}

// -------- borrowed (in-order) --------
pub struct Iter<'a, T> {
    stack: Vec<&'a Node<T>>,
}

impl<'a, T: Ord + Clone> IntoIterator for &'a BinaryTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Ord + Clone> BinaryTree<T> {
    pub fn iter(&'_ self) -> Iter<T> {
        let mut stack = Vec::new();
        let mut cur = self.root.as_deref();
        while let Some(n) = cur {
            stack.push(n);
            cur = n.left.as_deref();
        }
        Iter { stack }
    }
}

impl<'a, T: Ord + Clone> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let mut cur = node.right.as_deref();
        while let Some(n) = cur {
            self.stack.push(n);
            cur = n.left.as_deref();
        }
        Some(&node.elem)
    }
}

// ==================== Tests ====================

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
    fn depth_both_agree() {
        let t = build();
        assert_eq!(t.depth(), 3);
        assert_eq!(t.depth_iter(), 3);
    }

    #[test]
    fn balanced() {
        let t = build();
        assert!(t.is_balanced());
        let mut skewed = BinaryTree::new();
        for v in [1, 2, 3, 4, 5] { skewed.insert(v); }
        assert!(!skewed.is_balanced());
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

    #[test]
    fn drop_no_stack_overflow() {
        let mut t = BinaryTree::new();
        for v in 0..100_000 { t.insert(v); } // degenerate right spine
        drop(t);
    }
}