use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared, Guard};

pub struct Node<T> {
    value: T,
    next: Atomic<Node<T>>,
    marked: AtomicBool,
    version: AtomicUsize,
}

impl<T> Node<T>
where
    T: Clone,
{
    fn new(value: &T) -> Owned<Self> {
        Owned::new(Node {
            value: value.clone(),
            next: Atomic::null(),
            marked: AtomicBool::new(false),
            version: AtomicUsize::new(0),
        })
    }
}

pub struct LockFreeList<T> {
    head: Atomic<Node<T>>,
}

// Important: Remove additional trait bounds from Drop
impl<T> LockFreeList<T> {
    pub fn new() -> Self {
        Self {
            head: Atomic::null(),
        }
    }
}

impl<T: Ord + Clone + Send + Sync + 'static> LockFreeList<T> {
    fn find<'g>(&'g self, value: &T, guard: &'g Guard) -> (Shared<'g, Node<T>>, Shared<'g, Node<T>>) {
        loop {
            let mut prev = self.head.load(Ordering::Acquire, guard);
            let mut curr = prev;

            while let Some(curr_ref) = unsafe { curr.as_ref() } {
                let next = curr_ref.next.load(Ordering::Acquire, guard);

                if curr_ref.marked.load(Ordering::Acquire) {
                    let res = if prev == curr {
                        // removing the head
                        self.head.compare_exchange(
                            curr,
                            next,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                            guard,
                        )
                    } else {
                        // removing a middle node
                        let prev_ref = unsafe { prev.as_ref().unwrap() };
                        prev_ref.next.compare_exchange(
                            curr,
                            next,
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                            guard,
                        )
                    };

                    if res.is_ok() {
                        let raw_ptr = curr.as_raw() as *mut Node<T>;
                        let owned = unsafe { Owned::from_raw(raw_ptr) };
                        guard.defer(move || drop(owned));
                        curr = next;
                        continue;
                    } else {
                        break;
                    }
                }

                if curr_ref.value >= *value {
                    return (prev, curr);
                }

                prev = curr;
                curr = next;
            }

            return (prev, Shared::null());
        }
    }
    pub fn insert(&self, value: T) -> bool {
        let guard = &epoch::pin();
        let mut new_node = Node::new(&value);

        loop {
            let (prev, curr) = self.find(&value, guard);

            unsafe {
                if !curr.is_null() {
                    let curr_ref = curr.as_ref().unwrap();
                    if curr_ref.value == value && !curr_ref.marked.load(Ordering::Acquire) {
                        return false; // Already in the list
                    }
                }

                new_node.next.store(curr, Ordering::Relaxed);
                let new_shared = new_node.into_shared(guard);

                let res = if prev.is_null() {
                    self.head.compare_exchange(
                        curr,
                        new_shared,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                        guard,
                    )
                } else {
                    let prev_ref: &Node<T> = prev.as_ref().unwrap();
                    prev_ref.next.compare_exchange(
                        curr,
                        new_shared,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                        guard,
                    )
                };

                match res {
                    Ok(_) => return true,
                    Err(_err) => {
                        new_node = Node::new(&value);
                    }
                }
            }
        }
    }

    pub fn remove(&self, value: &T) -> bool {
        let guard = &epoch::pin();
        loop {
            let (prev, curr) = self.find(value, guard);
    
            if curr.is_null() {
                return false;
            }
    
            unsafe {
                let curr_ref = curr.as_ref().unwrap();
                if curr_ref.value != *value || curr_ref.marked.load(Ordering::Acquire) {
                    return false;
                }
    
                let next = curr_ref.next.load(Ordering::Acquire, guard);
    
                // Attempt to mark the node
                if curr_ref.marked.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_err() {
                    continue; // Another thread marked it; retry
                }
    
                curr_ref.version.fetch_add(1, Ordering::SeqCst);
    
                // Attempt to physically remove the node
                let res = if prev.is_null() {
                    self.head.compare_exchange(
                        curr,
                        next,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                        guard,
                    )
                } else {
                    let prev_ref: &Node<T> = prev.as_ref().unwrap();
                    prev_ref.next.compare_exchange(
                        curr,
                        next,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                        guard,
                    )
                };
    
                // If physical removal succeeded, defer its destruction
                if res.is_ok() {
                    guard.defer_destroy(curr);
                }
    
                return true;
            }
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        let guard = &epoch::pin();
        let (_, curr) = self.find(value, guard);
        unsafe {
            if curr.is_null() {
                false
            } else {
                let curr_ref = curr.as_ref().unwrap();
                !curr_ref.marked.load(Ordering::Acquire) && curr_ref.value == *value
            }
        }
    }
}

// Drop implementation without additional trait bounds
impl<T> Drop for LockFreeList<T> {
    fn drop(&mut self) {
        let guard = &epoch::pin();
        unsafe {
            let mut current = self.head.swap(Shared::null(), Ordering::Relaxed, guard);
            while let Some(_curr_ref) = current.as_ref() {
                let next = _curr_ref.next.load(Ordering::Relaxed, guard);
                // Schedule the node for reclamation instead of manually dropping
                guard.defer_destroy(current);
                current = next;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_operations() {
        let list = LockFreeList::<i32>::new();
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
