use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct HashMap<K, V> {
    size: usize,
    buckets: Vec<Vec<(K, V)>>,
}

const DEFAULT_CAPACITY: usize = 31;
const LOAD_FACTOR: f64 = 0.75;

impl<K, V> HashMap<K, V>
where K: Eq + Hash + Debug
{
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..=capacity {
            buckets.push(Vec::new());
        }
        Self {
            size: 0,
            buckets,
        }
    }

    pub fn resize(&mut self) {
        let new_size = self.buckets.len() * 2 + 1;
        let mut new_buckets: Vec<Vec<(K,V)>> = Vec::with_capacity(new_size);

        // rehash the items in the legacy one and move them into resized one.
        for bucket in self.buckets.drain(..) {
            for (k, v) in bucket {
                let mut hasher = DefaultHasher::new();
                k.hash(&mut hasher);
                let slot = hasher.finish() as usize %new_size;
                new_buckets[slot].push((k, v));
            }
        }
        self.buckets = new_buckets;
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Resize if load factor exceeded
        if (self.buckets.len() as f64) >= (self.buckets.len() as f64 * LOAD_FACTOR) {
            self.resize();
        }

        let index = self.bucket_index(&key);
        let bucket = &mut self.buckets[index];

        // Check if key already exists in this bucket
        for (k, v) in bucket.iter_mut() {
            if k == &key {
                // use std::mem::replace to change a value with its mut reference.
                return Some(std::mem::replace(v, value));
            }
        }

        // Key not found, insert new pair
        bucket.push((key, value));
        self.size += 1;
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.bucket_index(&key);
        let bucket = &self.buckets[index];
        bucket.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = self.bucket_index(&key);
        let bucket = &mut self.buckets[index];
        bucket.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.bucket_index(&key);
        let bucket = &mut self.buckets[index];
        if let Some(pos) = bucket.iter().position(|(k, _)| k == key) {
            let (_, v) = bucket.swap_remove(pos);
            self.size -= 1;
            Some(v)
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    fn bucket_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_map() {

    }
}