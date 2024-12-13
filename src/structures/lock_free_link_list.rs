// src/structures/lock_free_link_list.rs

use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use crate::structures::mcs_lock::{MCSLock, MCSNode, OperationSource};

/// Node structure for the linked list.
pub struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}

impl<T> Node<T> {
    fn new(value: T) -> *mut Self {
        Box::into_raw(Box::new(Node {
            value,
            next: AtomicPtr::new(ptr::null_mut()),
        }))
    }
}

/// Lock-based linked list using MCS Lock.
pub struct LockFreeList<T> {
    head: AtomicPtr<Node<T>>,
    lock: Arc<MCSLock>,
}

impl<T: Ord + Clone + Send + Sync + 'static> LockFreeList<T> {
    /// Creates a new empty list.
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            lock: Arc::new(MCSLock::new()),
        }
    }

    /// Internal helper to find the appropriate position for a value.
    /// Returns a tuple of (prev, curr) where `prev` is the node
    /// before the target position and `curr` is the node at or after
    /// the target position.
    fn find(&self, value: &T) -> (*mut Node<T>, *mut Node<T>) {
        let mut prev = ptr::null_mut();
        let mut curr = self.head.load(Ordering::Acquire);

        while !curr.is_null() {
            unsafe {
                if (*curr).value >= *value {
                    break;
                }
                prev = curr;
                curr = (*curr).next.load(Ordering::Acquire);
            }
        }

        (prev, curr)
    }

    /// Inserts a value into the list in sorted order.
    /// Returns `true` if the insertion was successful,
    /// or `false` if the value already exists.
    pub fn insert(&self, value: T) -> bool {
        let mut node = MCSNode::new();
        // Acquire lock with OperationSource::LinkedList
        self.lock.lock(&mut node, OperationSource::LinkedList);

        unsafe {
            let (prev, curr) = self.find(&value);

            if !curr.is_null() && (*curr).value == value {
                // Value already exists
                self.lock.unlock(&mut node, OperationSource::LinkedList);
                return false;
            }

            let new_node = Node::new(value);
            if prev.is_null() {
                // Insert at the head
                (*new_node).next.store(self.head.load(Ordering::Acquire), Ordering::Relaxed);
                self.head.store(new_node, Ordering::Release);
            } else {
                // Insert between prev and curr
                (*new_node).next.store(curr, Ordering::Relaxed);
                (*prev).next.store(new_node, Ordering::Release);
            }

            // Release lock with OperationSource::LinkedList
            self.lock.unlock(&mut node, OperationSource::LinkedList);
            true
        }
    }

    /// Removes a value from the list.
    /// Returns `true` if the removal was successful,
    /// or `false` if the value was not found.
    pub fn remove(&self, value: &T) -> bool {
        let mut node = MCSNode::new();
        // Acquire lock with OperationSource::LinkedList
        self.lock.lock(&mut node, OperationSource::LinkedList);

        unsafe {
            let (prev, curr) = self.find(value);

            if curr.is_null() || (*curr).value != *value {
                // Value not found
                self.lock.unlock(&mut node, OperationSource::LinkedList);
                return false;
            }

            let next = (*curr).next.load(Ordering::Acquire);
            if prev.is_null() {
                // Remove head
                self.head.store(next, Ordering::Release);
            } else {
                // Remove between prev and next
                (*prev).next.store(next, Ordering::Release);
            }

            // Deallocate the removed node
            Box::from_raw(curr);

            // Release lock with OperationSource::LinkedList
            self.lock.unlock(&mut node, OperationSource::LinkedList);
            true
        }
    }

    /// Checks if the list contains a value.
    /// Returns `true` if the value is present, or `false` otherwise.
    pub fn contains(&self, value: &T) -> bool {
        let mut node = MCSNode::new();
        // Acquire lock with OperationSource::LinkedList
        self.lock.lock(&mut node, OperationSource::LinkedList);

        let mut found = false;
        unsafe {
            let mut curr = self.head.load(Ordering::Acquire);
            while !curr.is_null() {
                if (*curr).value == *value {
                    found = true;
                    break;
                } else if (*curr).value > *value {
                    break;
                }
                curr = (*curr).next.load(Ordering::Acquire);
            }
        }

        // Release lock with OperationSource::LinkedList
        self.lock.unlock(&mut node, OperationSource::LinkedList);
        found
    }
}

impl<T> Drop for LockFreeList<T> {
    fn drop(&mut self) {
        let mut curr = self.head.load(Ordering::Relaxed);
        while !curr.is_null() {
            unsafe {
                let next = (*curr).next.load(Ordering::Relaxed);
                Box::from_raw(curr);
                curr = next;
            }
        }
    }
}
