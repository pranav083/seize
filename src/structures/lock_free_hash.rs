// src/structures/lock_free_hash.rs

use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use crate::structures::mcs_lock::{MCSLock, MCSNode};
use std::mem::MaybeUninit;

/// Number of buckets in the hash map. Adjust based on expected concurrency.
const NUM_BUCKETS: usize = 256;

/// Node representing a key-value pair in the hash map.
struct HashNode<K, V> {
    key: K,
    value: V,
    next: AtomicPtr<HashNode<K, V>>,
}

impl<K, V> HashNode<K, V> {
    fn new(key: K, value: V) -> Box<Self> {
        Box::new(HashNode {
            key,
            value,
            next: AtomicPtr::new(ptr::null_mut()),
        })
    }
}

/// Concurrent Hash Map using MCS Lock for each bucket.
pub struct LockFreeHashMap<K, V, S = RandomState>
where
    K: Eq + Hash,
    V: Clone,
    S: BuildHasher,
{
    buckets: Vec<(MCSLock, AtomicPtr<HashNode<K, V>>)>,
    hash_builder: S,
}

impl<K, V> LockFreeHashMap<K, V, RandomState>
where
    K: Eq + Hash,
    V: Clone,
{
    /// Creates a new, empty `LockFreeHashMap`.
    pub fn new() -> Self {
        let hash_builder = RandomState::new();
        Self::with_hasher(hash_builder)
    }
}

impl<K, V, S> LockFreeHashMap<K, V, S>
where
    K: Eq + Hash,
    V: Clone,
    S: BuildHasher,
{
    /// Creates a new, empty `LockFreeHashMap` with a specified hasher.
    pub fn with_hasher(hash_builder: S) -> Self {
        let mut buckets = Vec::with_capacity(NUM_BUCKETS);
        for _ in 0..NUM_BUCKETS {
            buckets.push((
                MCSLock::new(),
                AtomicPtr::new(ptr::null_mut()), // Head of the linked list
            ));
        }
        LockFreeHashMap {
            buckets,
            hash_builder,
        }
    }

    /// Computes the hash of a key and maps it to a bucket index.
    fn bucket_index<Q: ?Sized>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % NUM_BUCKETS
    }

    /// Inserts a key-value pair into the MCS hash map.
    pub fn insert(&self, key: K, value: V) {
        let index = self.bucket_index(&key);
        let node = Box::into_raw(HashNode::new(key, value));

        // Initialize MCS node for locking
        let mut mcs_node = MaybeUninit::<MCSNode>::uninit();
        let mcs_node_ptr = mcs_node.as_mut_ptr();
        unsafe { ptr::write(mcs_node_ptr, MCSNode::new()) };
        let mut mcs_node = unsafe { mcs_node.assume_init() };

        // Acquire lock
        self.buckets[index].0.lock(&mut mcs_node);

        // Insert at the head of the linked list
        unsafe {
            (*node).next.store(self.buckets[index].1.load(Ordering::Acquire), Ordering::Relaxed);
            self.buckets[index].1.store(node, Ordering::Release);
        }

        // Release lock
        self.buckets[index].0.unlock(&mut mcs_node);
    }

    /// Retrieves a cloned value corresponding to the key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = self.bucket_index(key);
        let mut result = None;

        // Initialize MCS node for locking
        let mut mcs_node = MaybeUninit::<MCSNode>::uninit();
        let mcs_node_ptr = mcs_node.as_mut_ptr();
        unsafe { ptr::write(mcs_node_ptr, MCSNode::new()) };
        let mut mcs_node = unsafe { mcs_node.assume_init() };

        // Acquire lock
        self.buckets[index].0.lock(&mut mcs_node);

        // Traverse the linked list
        let mut current = self.buckets[index].1.load(Ordering::Acquire);
        while !current.is_null() {
            unsafe {
                if (*current).key.borrow() == key {
                    result = Some((*current).value.clone());
                    break;
                }
                current = (*current).next.load(Ordering::Acquire);
            }
        }

        // Release lock
        self.buckets[index].0.unlock(&mut mcs_node);

        result
    }

    /// Removes a key-value pair from the MCS hash map.
    pub fn remove<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = self.bucket_index(key);
        let mut removed_value = None;

        // Initialize MCS node for locking
        let mut mcs_node = MaybeUninit::<MCSNode>::uninit();
        let mcs_node_ptr = mcs_node.as_mut_ptr();
        unsafe { ptr::write(mcs_node_ptr, MCSNode::new()) };
        let mut mcs_node = unsafe { mcs_node.assume_init() };

        // Acquire lock
        self.buckets[index].0.lock(&mut mcs_node);

        let mut prev_ptr = &self.buckets[index].1;
        let mut current = self.buckets[index].1.load(Ordering::Acquire);

        while !current.is_null() {
            unsafe {
                if (*current).key.borrow() == key {
                    // Remove the node
                    let next = (*current).next.load(Ordering::Acquire);
                    (*prev_ptr).store(next, Ordering::Release);
                    removed_value = Some((*current).value.clone());
                    // Deallocate the node
                    Box::from_raw(current);
                    break;
                }
                prev_ptr = &(*current).next;
                current = (*current).next.load(Ordering::Acquire);
            }
        }

        // Release lock
        self.buckets[index].0.unlock(&mut mcs_node);

        removed_value
    }
}

impl<K, V, S> Drop for LockFreeHashMap<K, V, S>
where
    K: Eq + Hash,
    V: Clone,
    S: BuildHasher,
{
    fn drop(&mut self) {
        for (_, bucket) in &self.buckets {
            let mut current = bucket.load(Ordering::Relaxed);
            while !current.is_null() {
                unsafe {
                    let next = (*current).next.load(Ordering::Relaxed);
                    // Reconstruct the Box to deallocate
                    Box::from_raw(current);
                    current = next;
                }
            }
        }
    }
}
