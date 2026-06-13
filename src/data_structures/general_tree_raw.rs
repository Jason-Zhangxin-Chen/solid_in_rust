use std::collections::VecDeque;
use std::marker::PhantomData;
use std::ptr::NonNull;

// ================= Node (private fields, but public API) ==================

pub struct Node<T> {
    value: T,
    children: Vec<NonNull<Node<T>>>,
}

impl<T> Node<T> {
    pub fn allocate(value: T) -> NonNull<Node<T>> {
        let boxed = Box::into_raw(Box::new(Node { value, children: Vec::new() }));
        unsafe { NonNull::new_unchecked(boxed) }
    }

    pub fn deallocate(ptr: NonNull<Node<T>>) -> Box<Self> {
        unsafe { Box::from_raw(ptr.as_ptr()) }
    }
}

impl<T> Node<T> {
    /// Creates a new leaf node.
    pub fn new(value: T) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }

    /// Adds a child node (the node is moved onto the heap).
    pub fn add_child(&mut self, child: Node<T>) {
        // usual case: sort children by value later; here we just push
        let boxed = Box::new(child);
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) };
        self.children.push(ptr);
    }

    /// Returns a reference to the stored value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the stored value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Returns an iterator over immutable references to this node’s children.
    pub fn children(&self) -> impl Iterator<Item = &Node<T>> {
        self.children.iter().map(|&ptr| unsafe { &*ptr.as_ptr() })
    }

    /// Returns an iterator over mutable references to this node’s children.
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Node<T>> {
        self.children
            .iter()
            .map(|&ptr| unsafe { &mut *ptr.as_ptr() })
    }

    // ---------- Helpers for internal use ----------
    fn child_nodes(&self) -> impl Iterator<Item = &Node<T>> {
        self.children()
    }

    fn child_nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<T>> {
        self.children_mut()
    }

    // ---------- Search ----------
    pub fn find<F>(&self, predicate: &F) -> Option<&Self>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(&self.value) {
            return Some(self);
        }
        for child in self.child_nodes() {
            if let Some(found) = child.find(predicate) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_mut<F>(&mut self, predicate: &F) -> Option<&mut Self>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(&self.value) {
            return Some(self);
        }
        for child in self.child_nodes_mut() {
            if let Some(found) = child.find_mut(predicate) {
                return Some(found);
            }
        }
        None
    }

    // ---------- Depth ----------
    pub fn depth(&self) -> usize {
        self.child_nodes()
            .map(|child| child.depth())
            .max()
            .map_or(1, |max_child_depth| 1 + max_child_depth)
    }

    // ---------- Traversals ----------
    pub fn traverse_preorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        visit(&self.value);
        for child in self.child_nodes() {
            child.traverse_preorder(visit);
        }
    }

    pub fn traverse_postorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        for child in self.child_nodes() {
            child.traverse_postorder(visit);
        }
        visit(&self.value);
    }

    pub fn traverse_bfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        let mut queue = VecDeque::new();
        queue.push_back(self);
        while let Some(node) = queue.pop_front() {
            visit(&node.value);
            for child in node.child_nodes() {
                queue.push_back(child);
            }
        }
    }

    pub fn traverse_dfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        let mut stack = Vec::new();
        stack.push(self);
        while let Some(node) = stack.pop() {
            visit(&node.value);
            // push children in reverse order so the first child is visited first
            let mut children: Vec<_> = node.child_nodes().collect();
            children.reverse();
            for child in children {
                stack.push(child);
            }
        }
    }
}

// ---------- Drop for Node (iterative, no stack overflow) ----------
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        // Move the children vector out, leaving an empty one in place.
        let children = std::mem::take(&mut self.children);
        let mut stack: Vec<NonNull<Node<T>>> = children;

        while let Some(ptr) = stack.pop() {
            // Convert the raw pointer back into an owned Box.
            let mut node_box = unsafe { Box::from_raw(ptr.as_ptr()) };

            // Take the children out of the box – this avoids a partial move.
            let child_nodes = std::mem::take(&mut node_box.children);
            stack.extend(child_nodes);

            // node_box goes out of scope here → its Node::drop runs,
            // but because we took children, it will only see an empty Vec
            // and the loop terminates immediately.  The value T is
            // dropped normally when the box is deallocated.
        }
    }
}

// ========================== Tree wrapper ====================================

pub struct Tree<T> {
    root: Option<NonNull<Node<T>>>,
    _marker: PhantomData<T>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        Tree { root: None, _marker: PhantomData }
    }

    pub fn with_root(root: Node<T>) -> Self {
        let boxed = Box::new(root);
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) };
        Tree { root: Some(ptr), _marker: PhantomData }
    }

    pub fn root(&self) -> Option<&Node<T>> {
        self.root.map(|ptr| unsafe { &*ptr.as_ptr() })
    }

    pub fn root_mut(&mut self) -> Option<&mut Node<T>> {
        self.root.map(|ptr| unsafe { &mut *ptr.as_ptr() })
    }

    pub fn set_root(&mut self, root: Node<T>) {
        // Drop old tree if any
        if let Some(old) = self.root.take() {
            let _ = unsafe { Box::from_raw(old.as_ptr()) };
        }
        let boxed = Box::new(root);
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) };
        self.root = Some(ptr);
    }

    pub fn depth(&self) -> usize {
        self.root().map_or(0, |root| root.depth())
    }

    pub fn find<F>(&self, predicate: F) -> Option<&Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        self.root().and_then(|root| root.find(&predicate))
    }

    pub fn traverse_preorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = self.root() {
            root.traverse_preorder(visit);
        }
    }

    pub fn traverse_postorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = self.root() {
            root.traverse_postorder(visit);
        }
    }

    pub fn traverse_bfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = self.root() {
            root.traverse_bfs(visit);
        }
    }

    pub fn traverse_dfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = self.root() {
            root.traverse_dfs(visit);
        }
    }
}

// ---------- Drop for Tree ----------
impl<T> Drop for Tree<T> {
    fn drop(&mut self) {
        if let Some(root_ptr) = self.root.take() {
            // Convert root back to Box; Node::drop handles the whole tree
            // As Node<T> implement the Drop Trait, it drops all the sub nodes.
            let _ = unsafe { Box::from_raw(root_ptr.as_ptr()) };
        }
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------- Debug implementations (since NonNull doesn't impl Debug) ----------
impl<T: std::fmt::Debug> std::fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("value", &self.value)
            .field("children", &self.child_nodes().collect::<Vec<_>>())
            .finish()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("root", &self.root())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_general_tree() {
        let mut root = Node::new("root/");

        let mut docs = Node::new("documents/");
        docs.add_child(Node::new("resume.pdf"));
        docs.add_child(Node::new("notes.txt"));

        let mut pics = Node::new("pictures/");
        pics.add_child(Node::new("vacation.png"));
        pics.add_child(Node::new("family.jpg"));

        root.add_child(docs);
        root.add_child(pics);
        root.add_child(Node::new("config.toml"));

        let tree = Tree::with_root(root);

        println!("Tree depth: {}", tree.depth());

        print!("\nPre-order: ");
        tree.traverse_preorder(&mut |val| print!("{} ", val));

        print!("\nPost-order: ");
        tree.traverse_postorder(&mut |val| print!("{} ", val));

        print!("\nBFS: ");
        tree.traverse_bfs(&mut |val| print!("{} ", val));

        print!("\nDFS (iterative): ");
        tree.traverse_dfs(&mut |val| print!("{} ", val));

        if let Some(node) = tree.find(|name| name == &"notes.txt") {
            println!("\n\nFound: {}", node.value());
        }
    }
}
