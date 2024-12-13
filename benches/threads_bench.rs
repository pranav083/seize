use std::hint::black_box;
use std::sync::{Arc};
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::{Collector};
use seize::LockFreeQueue;
use seize::structures::atomic_queue::AtomicQueue;
use crossbeam_epoch as epoch;
use haphazard::{Domain, HazardPointer};
use std::sync::atomic::AtomicPtr;

const ITEMS: usize = 200;

fn bench_atomic_enqueue_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Atomic Enqueue Multi-threaded");

    let thread_counts = [2, 4, 8, 16, 32];
    for &threads in &thread_counts {
        // No Scheme
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    handles.push(thread::spawn(move || {
                        let queue = AtomicQueue::new();
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue_clone.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Seize
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    let _guard = collector.enter();
                    handles.push(thread::spawn(move || {
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue_clone.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let _guard = epoch::pin();
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue_clone.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let mut hazard_pointer = HazardPointer::new();
                        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            unsafe {
                                let _protected = hazard_pointer.protect(&atomic_ptr);
                                queue_clone.enqueue(value);
                            }
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_atomic_dequeue_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Atomic Dequeue Multi-threaded");

    let thread_counts = [2, 4, 8, 16, 32];
    for &threads in &thread_counts {
        // No Scheme
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    handles.push(thread::spawn(move || {
                        let queue = AtomicQueue::new();
                        for i in 0..ITEMS {
                            queue.enqueue(i);
                        }
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue.dequeue();
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        for _ in 0..ITEMS {
                            if let Some(value) = queue_clone.dequeue() {
                                black_box(value);
                            }
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Seize
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(AtomicQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    let _guard = collector.enter();
                    handles.push(thread::spawn(move || {
                        for _ in 0..ITEMS {
                            queue_clone.dequeue();
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let _guard = epoch::pin();
                        for _ in 0..ITEMS {
                            queue_clone.dequeue();
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(AtomicQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let mut hazard_pointer = HazardPointer::new();
                        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
                        for _ in 0..ITEMS {
                            unsafe {
                                let _protected = hazard_pointer.protect(&atomic_ptr);
                                queue_clone.dequeue();
                            }
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_lock_free_enqueue_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free Enqueue Multi-threaded");
    
    for &threads in &[2, 4, 8, 16, 32] {
        // No Scheme
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    handles.push(thread::spawn(move || {
                        let queue = LockFreeQueue::new();
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue_clone.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    
        // Seize
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    let _guard = collector.enter();
                    handles.push(thread::spawn(move || {
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue_clone.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let _guard = epoch::pin();
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue_clone.enqueue(value);
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let mut hazard_pointer = HazardPointer::new();
                        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            unsafe {
                                let _protected = hazard_pointer.protect(&atomic_ptr);
                                queue_clone.enqueue(value);
                            }
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_lock_free_dequeue_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free Dequeue Multi-threaded");
    
    for &threads in &[2, 4, 8, 16, 32] {
        // No Scheme
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    handles.push(thread::spawn(move || {
                        let queue = LockFreeQueue::new();
                        for i in 0..ITEMS {
                            queue.enqueue(i);
                        }
                        for i in 0..ITEMS {
                            let value = black_box(i);
                            queue.dequeue();
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting        
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        for _ in 0..ITEMS {
                            if let Some(value) = queue_clone.dequeue() {
                                black_box(value);
                            }
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    
        // Seize
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    let _guard = collector.enter();
                    handles.push(thread::spawn(move || {
                        for _ in 0..ITEMS {
                            queue_clone.dequeue();
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    let queue_clone = Arc::clone(&queue);
                    handles.push(thread::spawn(move || {
                        let _guard = epoch::pin();
                        for _ in 0..ITEMS {
                            queue_clone.dequeue();
                        }
                    }));
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..ITEMS {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let mut hazard_pointer = HazardPointer::new();
                            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
                            for _ in 0..ITEMS {
                                unsafe {
                                    let _protected = hazard_pointer.protect(&atomic_ptr);
                                    queue_clone.dequeue();
                                }
                            }
                        }));
                    }
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_lock_free_enqueue_multi_threaded, bench_lock_free_dequeue_multi_threaded,
    bench_atomic_enqueue_multi_threaded, bench_atomic_dequeue_multi_threaded);
criterion_main!(benches);