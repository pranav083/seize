use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

pub struct Node<T> {
    value: Option<T>,
    next: AtomicPtr<Node<T>>,
}

pub struct AtomicQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T> AtomicQueue<T> {
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
                .compare_exchange(ptr::null_mut(), new_tail, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.tail.compare_exchange(tail, new_tail, Ordering::AcqRel, Ordering::Acquire).ok();
                return;
            } else {
                let next = tail_next.load(Ordering::Acquire);
                self.tail.compare_exchange(tail, next, Ordering::AcqRel, Ordering::Acquire).ok();
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let head_next = unsafe { (*head).next.load(Ordering::Acquire) };

            if head == tail {
                if head_next.is_null() {
                    return None;
                }
                self.tail.compare_exchange(tail, head_next, Ordering::AcqRel, Ordering::Acquire).ok();
            } else if !head_next.is_null() {
                let next = unsafe { &mut *head_next };
                let value = next.value.take();
                if self.head.compare_exchange(head, head_next, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                    unsafe { drop(Box::from_raw(head)) };
                    return value;
                }
            }
        }
    }
}

impl<T> Drop for AtomicQueue<T> {
    fn drop(&mut self) {
        while self.dequeue().is_some() {}
        let dummy = self.head.load(Ordering::Relaxed);
        unsafe { drop(Box::from_raw(dummy)) };

    }
}
