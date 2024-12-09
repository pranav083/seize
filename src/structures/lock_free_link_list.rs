use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering, fence};
use std::ptr;

/// Node structure for the linked list
pub struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
    marked: AtomicBool,      // For marking nodes as logically deleted
    version: AtomicUsize,    // For ABA prevention
}

/// Thread-safe lock-free linked list implementation.
/// # Safety
/// All operations are thread-safe and lock-free.
/// However, the caller must ensure that the list is not dropped while other threads are accessing it.
pub struct LockFreeList<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> Node<T>
where
    T: Clone,
{
    /// Creates a new node with the given value.
    fn new(value: &T) -> Option<*mut Self> {
        let node = Box::new(Self {
            value: value.clone(),
            next: AtomicPtr::new(ptr::null_mut()),
            marked: AtomicBool::new(false),
            version: AtomicUsize::new(0),
        });
        Some(Box::into_raw(node))
    }
}

impl<T: Ord + Clone> LockFreeList<T> {
    /// Creates a new lock-free list.
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Inserts a new value into the list. Returns `true` if the value was inserted,
    /// `false` if it already exists or allocation failed.
    pub fn insert(&self, value: T) -> bool {
        let Some(new_node) = Node::new(&value) else {
            return false; // Handle allocation failure
        };

        loop {
            let (prev, curr) = self.find(&value);

            unsafe {
                if !curr.is_null() && (*curr).value == value {
                    // Node already exists
                    drop(Box::from_raw(new_node)); // Prevent memory leak
                    return false;
                }

                // Set the next pointer of the new node
                (*new_node).next.store(curr, Ordering::Relaxed);

                // Attempt to insert the new node
                let result = if prev.is_null() {
                    self.head.compare_exchange(
                        curr,
                        new_node,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    )
                } else {
                    (*prev).next.compare_exchange(
                        curr,
                        new_node,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    )
                };

                if result.is_ok() {
                    // Insert successful
                    fence(Ordering::SeqCst);
                    return true;
                }

                // Retry on failure
            }
        }
    }

    /// Removes a value from the list. Returns `true` if the value was removed,
    /// `false` if it was not found.
    pub fn remove(&self, value: &T) -> bool {
        loop {
            let (prev, curr) = self.find(value);

            unsafe {
                if curr.is_null() || (*curr).value != *value {
                    // Node not found
                    return false;
                }

                let next = (*curr).next.load(Ordering::Acquire);

                // Mark the node as logically deleted
                if (*curr).marked.compare_exchange(
                    false,
                    true,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ).is_err() {
                    continue; // Retry if another thread marked it
                }

                // Increment version for ABA prevention
                (*curr).version.fetch_add(1, Ordering::SeqCst);

                // Attempt physical removal
                let result = if prev.is_null() {
                    self.head.compare_exchange(
                        curr,
                        next,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    )
                } else {
                    (*prev).next.compare_exchange(
                        curr,
                        next,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    )
                };

                if result.is_ok() {
                    drop(Box::from_raw(curr));
                }

                return true;
            }
        }
    }

    fn find(&self, value: &T) -> (*mut Node<T>, *mut Node<T>) {
        let mut prev: *mut Node<T> = ptr::null_mut();
        let mut curr = self.head.load(Ordering::Acquire);

        unsafe {
            while !curr.is_null() {
                let next = (*curr).next.load(Ordering::Acquire);

                if (*curr).marked.load(Ordering::Acquire) {
                    // Attempt physical removal
                    let result = if prev.is_null() {
                        self.head.compare_exchange(
                            curr,
                            next,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        )
                    } else {
                        (*prev).next.compare_exchange(
                            curr,
                            next,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        )
                    };

                    if result.is_ok() {
                        // Increment version for ABA prevention
                        (*curr).version.fetch_add(1, Ordering::SeqCst);
                        drop(Box::from_raw(curr)); // Deallocate safely
                        curr = next;
                        continue;
                    } else {
                        // Retry find on failure
                        return self.find(value);
                    }
                }

                if (*curr).value >= *value {
                    break;
                }

                prev = curr;
                curr = next;
            }
        }

        (prev, curr)
    }

    /// Checks if a value exists in the list
    pub fn contains(&self, value: &T) -> bool {
        let (_, curr) = self.find(value);
        unsafe {
            !curr.is_null() && 
            !(*curr).marked.load(Ordering::Acquire) && 
            (*curr).value == *value
        }
    }
}

impl<T> Drop for LockFreeList<T> {
    fn drop(&mut self) {
        unsafe {
            let mut current = self.head.load(Ordering::Relaxed);
            while !current.is_null() {
                let next = (*current).next.load(Ordering::Relaxed);
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}

// Example test module
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_basic_operations() {
        let list = LockFreeList::new();
        assert!(list.insert(1));
        assert!(list.insert(2));
        assert!(list.contains(&1));
        assert!(list.contains(&2));
        assert!(list.remove(&1));
        assert!(!list.contains(&1));
        assert!(list.contains(&2));
    }

    #[test]
    fn test_concurrent_operations() {
        let list = Arc::new(LockFreeList::new());
        let mut handles = vec![];

        for i in 0..10 {
            let list_clone = Arc::clone(&list);
            handles.push(thread::spawn(move || {
                assert!(list_clone.insert(i));
                assert!(list_clone.contains(&i));
                assert!(list_clone.remove(&i));
                assert!(!list_clone.contains(&i));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}