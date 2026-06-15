mod basic;
mod data_structures;
mod algo;
mod advance;
mod concurency;
mod smart_pointers;
mod design_patterns;
mod mistakes;
mod std_traits;

use std::iter::IntoIterator;
#[derive(Debug)]
struct MyStore<T> {
    store: Vec<T>,
}

impl<T> MyStore<T> {
    fn new() -> Self {
        Self { store: Vec::new() }
    }

    // Immutable iterator (via slice)
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.store.iter()
    }

    // Mutable iterator (via slice)
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.store.iter_mut()
    }

    // Owned iterator (delegate to Vec)
    fn into_iter(self) -> std::vec::IntoIter<T> {
        self.store.into_iter()
    }
}

// Immutable borrow, implement IntoIterator trait for type: reference of MyStore<T> with 'a.
impl<'a, T> IntoIterator for &'a MyStore<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.store.iter()
    }
}

// Mutable borrow, implement IntoIterator trait for type: mutable reference of MyStore<T> with 'a.
impl<'a, T> IntoIterator for &'a mut MyStore<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.store.iter_mut()
    }
}

// Owned, implement IntoIterator trait for type: MyStore<T> which is moved out from the underlying store.
impl<T> IntoIterator for MyStore<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.store.into_iter()
    }
}

fn main() {
    
    let mut store = MyStore::new();
    store.store.push("apple".to_string());
    store.store.push("orange".to_string());
    store.store.push("banana".to_string());

    // Immutable iteration
    for item in &store {
        println!("{}", item);
    }

    // Mutable iteration (add suffix)
    for item in &mut store {
        item.push_str("_fruit");
    }

    // Owned iteration (consumes store)
    let collected: Vec<String> = store.into_iter().collect();
    println!("owned: {:?}", collected);
}