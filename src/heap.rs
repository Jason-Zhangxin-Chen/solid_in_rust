use std::fmt::Debug;

pub struct MinHeap<T> {
    data: Vec<T>,
}

impl<T: Ord> MinHeap<T> {
    /// Creates a new empty heap.
    pub fn new() -> Self {
        MinHeap { data: Vec::new() }
    }

    /// Creates a heap from a vector, heapifying it in O(n) time.
    pub fn from_vec(mut data: Vec<T>) -> Self {
        // Heapify: start from last non-leaf node and sift down.
        let n = data.len();
        for i in (0..n / 2).rev() {
            Self::sift_down(&mut data, i, n);
        }
        MinHeap { data }
    }

    /// Returns the number of elements in the heap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns a reference to the smallest element, or `None` if empty.
    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    /// Inserts a new element into the heap.
    pub fn push(&mut self, value: T) {
        self.data.push(value);
        let len = self.data.len();
        Self::sift_up(&mut self.data, len - 1);
    }

    /// Removes and returns the smallest element, or `None` if empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        let last = self.data.pop().unwrap(); // safe because non‑empty
        if self.data.is_empty() {
            Some(last)
        } else {
            let min = std::mem::replace(&mut self.data[0], last);
            let len = self.data.len();
            // sift down the 1st element as it was replaced by the last which
            // break the heap balancing rule. shift_down from idx 0, the 1st item.
            Self::sift_down(&mut self.data, 0, len);
            Some(min)
        }
    }

    // ---------- Internal helpers ----------

    /// Restores heap property by moving the element at `idx` upward.
    fn sift_up(data: &mut [T], mut idx: usize) {
        while idx > 0 {
            // compare and swap with its parent: at (idx - 1)/2
            let parent = (idx - 1) / 2;
            // already ordered, quit the shift up, otherwise exhaust the iteration to root.
            if data[idx] >= data[parent] {
                break;
            }
            data.swap(idx, parent);
            idx = parent;
        }
    }

    /// Restores heap property by moving the element at `idx` downward.
    fn sift_down(data: &mut [T], mut idx: usize, len: usize) {
        loop {
            // try to get left child.
            let left = 2 * idx + 1;
            if left >= len {
                break;
            }

            // get the corresponding right child.
            let right = left + 1;

            // mark the current smallest item idx.
            let mut smallest = idx;

            // compare with left child, and mark the smallest one idx.
            if data[left] < data[smallest] {
                smallest = left;
            }

            // compare with right child and mark the smallest item idx if right child exist.
            if right < len && data[right] < data[smallest] {
                smallest = right;
            }

            // after comparing with both left and right child, if the smallest does not change,
            // then we end up with a rebalancing.
            if smallest == idx {
                break;
            }
            // otherwise, swap the smallest with the idx, and do this iteration again until the
            // full array rebalanced.
            data.swap(idx, smallest);
            idx = smallest;
        }
    }
}

// For debugging: print heap as array (not tree structure)
impl<T: Debug> Debug for MinHeap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.data).finish()
    }
}

// ---------- MaxHeap (simple wrapper) ----------
pub struct MaxHeap<T>(MinHeap<std::cmp::Reverse<T>>);

impl<T: Ord> MaxHeap<T> {
    pub fn new() -> Self {
        MaxHeap(MinHeap::new())
    }

    pub fn push(&mut self, value: T) {
        self.0.push(std::cmp::Reverse(value));
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop().map(|rev| rev.0)
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.peek().map(|rev| &rev.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}