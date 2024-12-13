// src/structures/mcs_lock.rs

use std::sync::atomic::{AtomicPtr, AtomicBool, Ordering, AtomicUsize};
use std::ptr;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Write;

/// Enum to identify the source of the operation.
#[derive(Debug, Clone, Copy)]
pub enum OperationSource {
    HashMap,
    LinkedList,
}

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

    /// Acquires the lock using the provided `MCSNode` and `OperationSource`.
    pub fn lock(&self, node: &mut MCSNode, source: OperationSource) {
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

    /// Releases the lock using the provided `MCSNode` and `OperationSource`.
    pub fn unlock(&self, node: &mut MCSNode, source: OperationSource) {
        let next = node.next.load(Ordering::Acquire);
        if next.is_null() {
            // No successor; attempt to reset the tail to null
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
            // CAS failed; increment the appropriate counter
            match source {
                OperationSource::HashMap => {
                    CAS_FAILURES_HASHMAP.fetch_add(1, Ordering::Relaxed);
                }
                OperationSource::LinkedList => {
                    CAS_FAILURES_LINKEDLIST.fetch_add(1, Ordering::Relaxed);
                }
            }
            // Wait for successor to appear
            while node.next.load(Ordering::Acquire).is_null() {}
        }
        unsafe {
            (*node.next.load(Ordering::Acquire)).locked.store(false, Ordering::Release);
        }
    }
}

// Define global atomic counters for CAS failures
static CAS_FAILURES_HASHMAP: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static CAS_FAILURES_LINKEDLIST: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

/// Structure responsible for writing CAS failure counts to a CSV file upon program termination.
struct CsvWriter;

impl Drop for CsvWriter {
    fn drop(&mut self) {
        let hashmap_failures = CAS_FAILURES_HASHMAP.load(Ordering::Relaxed);
        let linkedlist_failures = CAS_FAILURES_LINKEDLIST.load(Ordering::Relaxed);

        let mut file = match File::create("cas_failures.csv") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to create CSV file: {}", e);
                return;
            }
        };

        if let Err(e) = writeln!(file, "DataStructure,CASFailures") {
            eprintln!("Failed to write CSV header: {}", e);
            return;
        }
        if let Err(e) = writeln!(file, "HashMap,{}", hashmap_failures) {
            eprintln!("Failed to write HashMap data to CSV: {}", e);
        }
        if let Err(e) = writeln!(file, "LinkedList,{}", linkedlist_failures) {
            eprintln!("Failed to write LinkedList data to CSV: {}", e);
        }
    }
}

// Initialize the CsvWriter to ensure it gets dropped at program exit
static CSV_WRITER: Lazy<CsvWriter> = Lazy::new(|| CsvWriter);
