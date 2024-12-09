use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::RandomState;
use std::sync::Arc;
use std::hash::BuildHasher;


/// The maximum number of buckets in the hash map.
const NUM_BUCKETS: usize = 64;

/// A node in the linked list for handling collisions.
struct Node<K, V> {
    key: K,
    value: V,
    next: AtomicPtr<Node<K, V>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> *mut Self {
        Box::into_raw(Box::new(Node {
            key,
            value,
            next: AtomicPtr::new(ptr::null_mut()),
        }))
    }
}

/// A lock-free hash map.
pub struct LockFreeHashMap<K, V> {
    buckets: Vec<AtomicPtr<Node<K, V>>>,
    hash_builder: RandomState,
}

impl<K, V> LockFreeHashMap<K, V>
where
    K: Eq + Hash,
    V: Clone, // Added Clone bound here
{
    /// Creates a new, empty `LockFreeHashMap`.
    pub fn new() -> Self {
        let mut buckets = Vec::with_capacity(NUM_BUCKETS);
        for _ in 0..NUM_BUCKETS {
            buckets.push(AtomicPtr::new(ptr::null_mut()));
        }
        LockFreeHashMap {
            buckets,
            hash_builder: RandomState::new(),
        }
    }

    /// Computes the hash of a key and maps it to a bucket index.
    fn bucket_index<Q: ?Sized>(&self, key: &Q) -> usize
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % NUM_BUCKETS
    }

    /// Inserts a key-value pair into the hash map.
    pub fn insert(&self, key: K, value: V) {
        let index = self.bucket_index(&key);
        let new_node = Node::new(key, value);

        let bucket = &self.buckets[index];
        loop {
            let head = bucket.load(Ordering::Acquire);
            unsafe {
                (*new_node).next.store(head, Ordering::Relaxed);
            }
            if bucket
                .compare_exchange(head, new_node, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Retrieves a reference to the value corresponding to the key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<Arc<V>>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = self.bucket_index(key);
        let mut current = self.buckets[index].load(Ordering::Acquire);

        while !current.is_null() {
            unsafe {
                if (*current).key.borrow() == key {
                    return Some(Arc::new((*current).value.clone())); // Clone now works
                }
                current = (*current).next.load(Ordering::Acquire);
            }
        }
        None
    }
    

   /// Removes a key-value pair from the hash map.
    pub fn remove<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = self.bucket_index(key);
        let bucket = &self.buckets[index];

        let mut prev = bucket.load(Ordering::Acquire);
        let mut current = prev;

        while !current.is_null() {
            unsafe {
                let next = (*current).next.load(Ordering::Acquire);
                if (*current).key.borrow() == key {
                    let value = ptr::read(&(*current).value); // Take ownership of the value

                    if prev == current {
                        // Node is at the head of the list
                        if bucket
                            .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                            .is_ok()
                        {
                            drop(Box::from_raw(current)); // Free the memory
                            return Some(value);
                        }
                    } else {
                        // Node is in the middle or end of the list
                        let prev_node = &(*prev).next;
                        if prev_node
                            .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                            .is_ok()
                        {
                            drop(Box::from_raw(current)); // Free the memory
                            return Some(value);
                        }
                    }
                }

                // Move to the next node
                prev = current;
                current = next;
            }
        }
        None
    }
}



impl<K, V> Drop for LockFreeHashMap<K, V> {
    fn drop(&mut self) {
        for bucket in &self.buckets {
            let mut current = bucket.load(Ordering::Acquire);
            while !current.is_null() {
                unsafe {
                    let next = (*current).next.load(Ordering::Acquire);
                    drop(Box::from_raw(current)); // Explicitly drop the box
                    current = next;
                }
            }
        }
    }
}
