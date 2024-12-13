// src/structures/mcs_lock.rs

use std::sync::atomic::{AtomicPtr, AtomicBool, Ordering};
use std::ptr;

/// Represents a node in the MCS queue.
pub struct MCSNode {
    pub next: AtomicPtr<MCSNode>,
    pub locked: AtomicBool,
}

impl MCSNode {
    /// Creates a new `MCSNode`. Initially locked.
    pub fn new() -> Self {
        MCSNode {
            next: AtomicPtr::new(ptr::null_mut()),
            locked: AtomicBool::new(true),
        }
    }
}

/// MCS Lock structure.
pub struct MCSLock {
    pub tail: AtomicPtr<MCSNode>,
}

impl MCSLock {
    /// Creates a new `MCSLock`.
    pub fn new() -> Self {
        MCSLock {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Acquires the lock using the provided `MCSNode`.
    pub fn lock(&self, node: &mut MCSNode) {
        node.next.store(ptr::null_mut(), Ordering::Relaxed);
        let prev = self.tail.swap(node as *mut MCSNode, Ordering::AcqRel);
        if !prev.is_null() {
            unsafe {
                (*prev).next.store(node as *mut MCSNode, Ordering::Release);
            }
            // Spin until the predecessor gives up the lock
            while node.locked.load(Ordering::Acquire) {}
        }
    }

    /// Releases the lock using the provided `MCSNode`.
    pub fn unlock(&self, node: &mut MCSNode) {
        let next = node.next.load(Ordering::Acquire);
        if next.is_null() {
            // No successor
            if self
                .tail
                .compare_exchange(
                    node as *mut MCSNode,
                    ptr::null_mut(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return;
            }
            // Wait for successor to appear
            while node.next.load(Ordering::Acquire).is_null() {}
        }
        unsafe {
            (*node.next.load(Ordering::Acquire)).locked.store(false, Ordering::Release);
        }
    }
}
