use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A node in the graph.
///
/// `neighbors` is a list of raw, non-null pointers to other nodes.
/// Because the `Graph` owns every node and never removes them individually,
/// these pointers remain valid for the entire lifetime of the `Graph`.
pub struct Node<T> {
    pub value: T,
    neighbors: RefCell<Vec<NonNull<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            neighbors: RefCell::new(Vec::new()),
        }
    }

    /// Adds an edge from `from` to `to`.
    ///
    /// # Safety
    ///
    /// Both `from` and `to` must be pointers to nodes that are currently alive
    /// (i.e., they are owned by a `Graph` that has not been dropped).
    pub fn add_edge(from: NonNull<Node<T>>, to: NonNull<Node<T>>) {
        // SAFETY: The caller guarantees that `from` points to a valid, initialized `Node`.
        unsafe {
            (*from.as_ptr()).neighbors.borrow_mut().push(to);
        }
    }
}

/// The graph owns all its nodes, keeping them in a `Vec` of `NonNull` pointers.
/// When the `Graph` is dropped, each allocation is properly freed.
pub struct Graph<T> {
    nodes: Vec<NonNull<Node<T>>>,
    _marker: PhantomData<T>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Graph { nodes: Vec::new() , _marker: PhantomData }
    }

    /// Adds a new node with the given value and returns a `NonNull` pointer to it.
    /// The pointer can be used later to connect edges or as a starting point for traversals.
    pub fn add_node(&mut self, value: T) -> NonNull<Node<T>> {
        let boxed = Box::new(Node::new(value));
        // Convert the Box into a raw pointer and wrap it in NonNull.
        let ptr = NonNull::new(Box::into_raw(boxed)).expect("Box never returns null");
        self.nodes.push(ptr);
        ptr
    }

    /// Adds an edge from `from` to `to`.
    ///
    /// # Safety
    ///
    /// Both `from` and `to` must be pointers that were returned by `add_node` of this same
    /// `Graph`, and the graph must not have been dropped.
    pub fn add_edge(&mut self, from: NonNull<Node<T>>, to: NonNull<Node<T>>) {
        Node::add_edge(from, to);
    }

    /// Performs a depth-first search starting from `start`, calling `visit` for each
    /// node's value exactly once.
    ///
    /// # Safety
    ///
    /// `start` must be a valid pointer to a node that belongs to a live `Graph`.
    /// The graph must not be mutated during traversal (this is guaranteed by the
    /// immutable `&self` receiver in a single-threaded context).
    pub fn dfs<F>(&self, start: NonNull<Node<T>>, visit: &mut F)
    where
        F: FnMut(&T),
    {
        use std::collections::HashSet;

        let mut stack = vec![start];
        // We identify visited nodes by their address.
        let mut visited: HashSet<*const Node<T>> = HashSet::new();

        while let Some(node_ptr) = stack.pop() {
            // If we've already processed this node, skip it.
            if !visited.insert(node_ptr.as_ptr() as *const Node<T>) {
                continue;
            }

            // SAFETY: `node_ptr` is a valid pointer to a node that is still alive.
            let node = unsafe { &*node_ptr.as_ptr() };
            visit(&node.value);

            // Push neighbours in reverse order to mimic the original stack behaviour.
            for neighbor in node.neighbors.borrow().iter().rev() {
                stack.push(*neighbor);
            }
        }
    }

    /// Performs a breadth-first search starting from `start`, calling `visit` for each
    /// node's value exactly once.
    ///
    /// # Safety
    ///
    /// Same constraints as `dfs`.
    pub fn bfs<F>(&self, start: NonNull<Node<T>>, visit: &mut F)
    where
        F: FnMut(&T),
    {
        use std::collections::{HashSet, VecDeque};

        let mut queue = VecDeque::new();
        let mut visited: HashSet<*const Node<T>> = HashSet::new();
        queue.push_back(start);

        while let Some(node_ptr) = queue.pop_front() {
            if !visited.insert(node_ptr.as_ptr() as *const Node<T>) {
                continue;
            }

            // SAFETY: valid, live node.
            let node = unsafe { &*node_ptr.as_ptr() };
            visit(&node.value);

            for neighbor in node.neighbors.borrow().iter() {
                queue.push_back(*neighbor);
            }
        }
    }
}

// When the Graph is dropped, we must free each allocated node.
impl<T> Drop for Graph<T> {
    fn drop(&mut self) {
        for &ptr in &self.nodes {
            // SAFETY: `ptr` came from a `Box::into_raw` call and we haven't freed it yet.
            // Converting it back to a Box will drop the Node and its contents.
            unsafe {
                drop(Box::from_raw(ptr.as_ptr()));
            }
        }
    }
}