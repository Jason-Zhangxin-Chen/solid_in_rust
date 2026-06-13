use std::cell::RefCell;
use std::rc::{Rc, Weak};

type NodeRef<T> = Rc<RefCell<GraphNode<T>>>;
type WeakRef<T> = Weak<RefCell<GraphNode<T>>>;

pub struct GraphNode<T> {
    value: T,
    // ✅ Weak refs: edges don't contribute to the ref count,
    //    breaking any potential ownership cycles.
    neighbors: Vec<WeakRef<T>>,
}

impl<T> GraphNode<T> {
    pub fn new(value: T) -> Self {
        GraphNode {
            value,
            neighbors: Vec::new(),
        }
    }

    pub fn add_edge(from: &NodeRef<T>, to: &NodeRef<T>) {
        // Rc::downgrade produces a Weak pointer — no strong-ref cycle
        from.borrow_mut().neighbors.push(Rc::downgrade(to));
    }
}

pub struct Graph<T> {
    // Graph is the *sole* strong owner of every node
    nodes: Vec<NodeRef<T>>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Graph { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, value: T) -> NodeRef<T> {
        let node = Rc::new(RefCell::new(GraphNode::new(value)));
        self.nodes.push(Rc::clone(&node));
        node
    }

    pub fn add_edge(&mut self, from: &NodeRef<T>, to: &NodeRef<T>) {
        GraphNode::add_edge(from, to);
    }

    pub fn dfs<F>(start: &NodeRef<T>, visit: &mut F)
    where
        F: FnMut(&T),
    {
        use std::collections::HashSet;
        let mut stack = vec![Rc::clone(start)];
        let mut visited = HashSet::new();

        while let Some(node_rc) = stack.pop() {
            let node_ptr = Rc::as_ptr(&node_rc) as *const ();
            if !visited.insert(node_ptr) {
                // already visited.
                continue;
            }
            let node = node_rc.borrow();
            visit(&node.value);

            for weak_neighbor in node.neighbors.iter().rev() {
                // upgrade() returns None if the node was dropped — skip it safely
                if let Some(neighbor_rc) = weak_neighbor.upgrade() {
                    stack.push(neighbor_rc);
                }
            }
        }
    }

    pub fn bfs<F>(start: &NodeRef<T>, visit: &mut F)
    where
        F: FnMut(&T),
    {
        use std::collections::{HashSet, VecDeque};
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(Rc::clone(start));

        while let Some(node_rc) = queue.pop_front() {
            let node_ptr = Rc::as_ptr(&node_rc) as *const ();
            if !visited.insert(node_ptr) {
                // node already visited.
                continue;
            }
            let node = node_rc.borrow();
            visit(&node.value);

            for weak_neighbor in &node.neighbors {
                if let Some(neighbor_rc) = weak_neighbor.upgrade() {
                    queue.push_back(neighbor_rc);
                }
            }
        }
    }
}