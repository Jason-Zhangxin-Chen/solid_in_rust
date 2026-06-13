use std::mem::MaybeUninit;
use std::ptr;

/// A generic ring buffer with fixed capacity.
pub struct RingBuffer<T> {
    buffer: Box<[MaybeUninit<T>]>, // heap‑allocated storage (smart pointer)
    read: usize,                   // next index to read from
    write: usize,                  // next index to write to
    count: usize,                  // number of active elements
}

impl<T> RingBuffer<T> {
    /// Creates a new empty ring buffer with the given capacity.
    ///
    /// # Panics
    /// Panics if capacity is zero.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be at least 1");

        // Create a boxed slice of uninitialised memory
        let buffer = (0..capacity)
            .map(|_| MaybeUninit::uninit())
            .collect::<Vec<MaybeUninit<T>>>()
            .into_boxed_slice();

        RingBuffer {
            buffer,
            read: 0,
            write: 0,
            count: 0,
        }
    }

    /// Returns the maximum number of elements the buffer can hold.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns the number of elements currently in the buffer.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the buffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns `true` if the buffer is full.
    pub fn is_full(&self) -> bool {
        self.count == self.capacity()
    }

    /// Adds an element to the buffer. If the buffer is full, the oldest element is overwritten.
    /// Returns the overwritten element (if any).
    pub fn push_overwrite(&mut self, value: T) -> Option<T> {
        let overwritten = if self.is_full() {
            // Buffer is full – we will overwrite the oldest element (at `read`)
            unsafe {
                let old = ptr::read(self.buffer[self.read].as_ptr());
                Some(old)
            }
        } else {
            None
        };

        // Write the new value into the buffer at `write`
        self.buffer[self.write].write(value);

        // Advance write index
        self.write = (self.write + 1) % self.capacity();

        if overwritten.is_some() {
            // We overwrote the oldest element, so read index moves forward as well
            self.read = (self.read + 1) % self.capacity();
        } else {
            self.count += 1;
        }

        overwritten
    }

    /// Adds an element only if the buffer is not full.
    /// Returns `Ok(())` on success, or `Err(value)` if the buffer was full.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            Err(value)
        } else {
            self.buffer[self.write].write(value);
            self.write = (self.write + 1) % self.capacity();
            self.count += 1;
            Ok(())
        }
    }

    /// Removes and returns the oldest element, or `None` if the buffer is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let value = unsafe { self.buffer[self.read].assume_init_read() };
            self.read = (self.read + 1) % self.capacity();
            self.count -= 1;
            Some(value)
        }
    }

    /// Returns a reference to the oldest element without removing it.
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.buffer[self.read].assume_init_ref()) }
        }
    }

    /// Returns a mutable reference to the oldest element.
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.buffer[self.read].assume_init_mut()) }
        }
    }

    /// Clears the buffer, dropping all contained elements.
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }
}

// When the ring buffer is dropped, we must drop any remaining elements.
impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

// Example: iteration over references
impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut vec = Vec::with_capacity(self.len());
        let mut this = self;
        while let Some(v) = this.pop() {
            vec.push(v);
        }
        vec.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structures::ring_buffer::RingBuffer;

    #[test]
    fn test_ring_buffer() {
        let mut buf = RingBuffer::new(3);
        assert_eq!(buf.push(10), Ok(()));
        assert_eq!(buf.push(20), Ok(()));
        assert_eq!(buf.push(30), Ok(()));
        assert_eq!(buf.push(40), Err(40)); // full → error

        assert_eq!(buf.pop(), Some(10));
        assert_eq!(buf.pop(), Some(20));
        assert_eq!(buf.pop(), Some(30));
        assert_eq!(buf.pop(), None);

        // Overwrite mode
        let mut buf2 = RingBuffer::new(2);
        buf2.push_overwrite('a');
        buf2.push_overwrite('b');
        assert_eq!(buf2.push_overwrite('c'), Some('a')); // overwrites 'a'
        assert_eq!(buf2.pop(), Some('b'));
        assert_eq!(buf2.pop(), Some('c'));
    }
}