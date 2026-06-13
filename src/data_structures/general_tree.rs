use std::collections::VecDeque;

/// A node in the general tree.
#[derive(Debug)]
pub struct Node<T> {
    value: T,
    children: Vec<Box<Node<T>>>,
}

impl<T> Node<T> {
    /// Creates a new leaf node with the given value.
    pub fn new(value: T) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }

    /// Adds a child to this node.
    pub fn add_child(&mut self, child: Node<T>) {
        // todo: usually need to sort the childrens by value.
        self.children.push(Box::new(child));
    }

    /// Returns an immutable reference to the value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the value.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Returns a slice of the node's children.
    pub fn children(&self) -> &[Box<Node<T>>] {
        &self.children
    }

    // ---------- Search ----------
    /// Searches the tree for a value that satisfies the predicate.
    /// Returns the first node (as a reference) where `predicate` returns `true`.
    pub fn find<F>(&self, predicate: &F) -> Option<&Self>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(&self.value) {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(predicate) {
                return Some(found);
            }
        }
        None
    }

    /// Mutable version of `find`.
    pub fn find_mut<F>(&mut self, predicate: &F) -> Option<&mut Self>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(&self.value) {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(predicate) {
                return Some(found);
            }
        }
        None
    }

    // ---------- Depth ----------
    /// Returns the maximum depth (height) of the tree rooted at this node.
    /// A leaf node has depth 1.
    pub fn depth(&self) -> usize {
        self.children
            .iter()
            .map(|child| child.depth())
            .max()
            .map_or(1, |max_child_depth| 1 + max_child_depth)
    }

    // ---------- Traversals ----------
    /// Pre‑order traversal: visits the node first, then each child recursively.
    pub fn traverse_preorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        visit(&self.value);
        for child in &self.children {
            child.traverse_preorder(visit);
        }
    }

    /// Post‑order traversal: visits children recursively first, then the node.
    pub fn traverse_postorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        for child in &self.children {
            child.traverse_postorder(visit);
        }
        visit(&self.value);
    }

    /// Breadth‑First Search (Level‑order) traversal.
    pub fn traverse_bfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        let mut queue = VecDeque::new();
        queue.push_back(self);
        while let Some(node) = queue.pop_front() {
            visit(&node.value);
            for child in &node.children {
                queue.push_back(child);
            }
        }
    }

    /// Iterative Depth‑First Search (using a stack).
    /// This is equivalent to pre‑order but without recursion.
    pub fn traverse_dfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        let mut stack = Vec::new();
        stack.push(self);
        while let Some(node) = stack.pop() {
            visit(&node.value);
            // Push children in reverse order so that the first child is processed first.
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
    }
}

/// Optional wrapper struct for a general tree.
#[derive(Debug)]
pub struct Tree<T> {
    root: Option<Box<Node<T>>>,
}

impl<T> Tree<T> {
    /// Creates an empty tree.
    pub fn new() -> Self {
        Tree { root: None }
    }

    /// Creates a tree with the given root node.
    pub fn with_root(root: Node<T>) -> Self {
        Tree {
            root: Some(Box::new(root)),
        }
    }

    /// Returns a reference to the root node, if any.
    pub fn root(&self) -> Option<&Node<T>> {
        self.root.as_deref()
    }

    /// Returns a mutable reference to the root node, if any.
    pub fn root_mut(&mut self) -> Option<&mut Node<T>> {
        self.root.as_deref_mut()
    }

    /// Sets the root of the tree.
    pub fn set_root(&mut self, root: Node<T>) {
        self.root = Some(Box::new(root));
    }

    // Delegate common operations to the root.
    pub fn depth(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.depth())
    }

    pub fn find<F>(&self, predicate: F) -> Option<&Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        self.root.as_ref().and_then(|root| root.find(&predicate))
    }

    pub fn traverse_preorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_preorder(visit);
        }
    }

    pub fn traverse_postorder<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_postorder(visit);
        }
    }

    pub fn traverse_bfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_bfs(visit);
        }
    }

    pub fn traverse_dfs<F>(&self, visit: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_dfs(visit);
        }
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------- Example Usage ----------
fn main() {
    // Build a tree representing a simple filesystem hierarchy.
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

    println!("Tree depth: {}", tree.depth()); // 3

    println!("\nPre‑order traversal:");
    tree.traverse_preorder(&mut |val| print!("{} ", val));
    // root/ documents/ resume.pdf notes.txt pictures/ vacaction.png family.jpg config.toml

    println!("\n\nPost‑order traversal:");
    tree.traverse_postorder(&mut |val| print!("{} ", val));
    // resume.pdf notes.txt documents/ vacaction.png family.jpg pictures/ config.toml root/

    println!("\n\nBFS (level‑order):");
    tree.traverse_bfs(&mut |val| print!("{} ", val));
    // root/ documents/ pictures/ config.toml resume.pdf notes.txt vacaction.png family.jpg

    println!("\n\nDFS (iterative pre‑order):");
    tree.traverse_dfs(&mut |val| print!("{} ", val));
    // Same as pre‑order

    // Search for a file
    if let Some(node) = tree.find(|name| name == &"notes.txt") {
        println!("\n\nFound: {}", node.value());
    }
}