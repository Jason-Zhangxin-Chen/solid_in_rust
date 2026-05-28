

pub struct MyStack<T> {
    slots: Vec<T>,
}

impl<T> MyStack<T> {
    pub fn new() -> Self {
        let mut slots: Vec<T> = Vec::new();
        slots.reserve(1000);
        Self{slots: slots}
    }

    pub fn push(&mut self, value: T) {
        self.slots.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.slots.len() > 0 {
            self.slots.pop()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn peek(&self) -> Option<&T> {
        self.slots.last()
    }
}