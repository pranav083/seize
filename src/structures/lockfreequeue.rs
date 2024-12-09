use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

pub struct Node<T> {
    pub value: Option<T>,
    pub next: AtomicPtr<Node<T>>,
}

pub struct LockFreeQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            value: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));
        Self {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
        }
    }

    pub fn enqueue(&self, value: T) {
        let new_tail = Box::into_raw(Box::new(Node {
            value: Some(value),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let tail_next = unsafe { &(*tail).next };

            if tail_next
                .compare_exchange(
                    ptr::null_mut(),
                    new_tail,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                self.tail
                    .compare_exchange(tail, new_tail, Ordering::Release, Ordering::Relaxed)
                    .ok();
                return;
            } else {
                self.tail
                    .compare_exchange(tail, tail_next.load(Ordering::Acquire), Ordering::Release, Ordering::Relaxed)
                    .ok();
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if head == tail {
                if next.is_null() {
                    return None; // Queue is empty
                }
                self.tail.compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed).ok();
            } else if self
                .head
                .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                let value = unsafe { (*next).value.take() };
                unsafe { drop(Box::from_raw(head)) };
                return value;
            }
        }
    }
}