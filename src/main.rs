
mod iterator;
mod my_container;
mod my_container2;
mod sorts;
mod single_linked_list;
mod single_linked_list_nonnull;
mod single_linked_list_raw;
mod double_linked_list;
mod binary_tree;
mod stack;
mod ring_buffer;
mod general_tree;
mod graph;
mod heap;
mod hash_map;
mod n_queen;
mod fib;
mod smart_pointers_methods;
mod bits;
mod merkle_tree;
mod trie_db;
mod double_linked_list_raw;
mod binary_tree_raw;
mod general_tree_raw;
mod graph_raw;
mod error_raw;
mod deref;
mod pin_unpin;
mod custom_allocator;
mod sub_typing;
mod marcros;
mod order_book_state_root;
mod basic;
mod pcm;
mod tcp_service;
mod phantom_data;
mod error_bitmask;
mod common_issues;
mod uc_box;
mod uc_rc;
mod uc_arc;
mod uc_cell;
mod uc_refcell;
mod uc_weak;
mod uc_cow;
mod fat_pointer;
mod error_thiserror;
mod error_anyhow;

use std::iter::IntoIterator;
use std::sync::atomic::AtomicU64;

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