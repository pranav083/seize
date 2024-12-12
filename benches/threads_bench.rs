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

fn bench_atomic_queue_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Atomic Queue Multi-threaded");
    group.sample_size(10);

    // Reference Counting
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                // Pre-fill the queue
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let _queue_clone = Arc::clone(&queue);
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let queue_clone = Arc::clone(&queue);
                            thread::spawn(move || {
                                for _ in 0..200 {
                                    if let Some(value) = queue_clone.dequeue() {
                                        black_box(value);
                                    }
                                }
                            })
                        })
                        .collect();
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    

    // Seize
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        let _guard = collector.enter();
                        handles.push(thread::spawn(move || {
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(AtomicQueue::new());
                // Pre-fill the queue
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        let _guard = collector.enter();
                        handles.push(thread::spawn(move || {
                            for _ in (t * 200)..((t + 1) * 200) {
                                queue_clone.dequeue();
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

    // Crossbeam Epoch
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let _guard = epoch::pin();
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(AtomicQueue::new());
                // Pre-fill the queue
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let _guard = epoch::pin();
                            for _ in (t * 200)..((t + 1) * 200) {
                                queue_clone.dequeue();
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

    // Hazard Pointer
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(AtomicQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let mut hazard_pointer = HazardPointer::new();
                            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                unsafe {
                                    let _protected = hazard_pointer.protect(&atomic_ptr);
                                    queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(AtomicQueue::new());
                // Pre-fill the queue
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let mut hazard_pointer = HazardPointer::new();
                            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
                            for _ in (t * 200)..((t + 1) * 200) {
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

fn bench_lock_free_queue_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free Queue Multi-threaded");
    group.sample_size(10);

    // Reference Counting
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let _queue_clone = Arc::clone(&queue);
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let queue_clone = Arc::clone(&queue);
                            thread::spawn(move || {
                                for _ in 0..200 {
                                    if let Some(value) = queue_clone.dequeue() {
                                        black_box(value);
                                    }
                                }
                            })
                        })
                        .collect();
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }
    

    // Seize
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        let _guard = collector.enter();
                        handles.push(thread::spawn(move || {
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        let _guard = collector.enter();
                        handles.push(thread::spawn(move || {
                            for _ in (t * 200)..((t + 1) * 200) {
                                queue_clone.dequeue();
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

    // Crossbeam Epoch
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let _guard = epoch::pin();
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let _guard = epoch::pin();
                            for _ in (t * 200)..((t + 1) * 200) {
                                queue_clone.dequeue();
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

    // Hazard Pointer
    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let mut hazard_pointer = HazardPointer::new();
                            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                            for i in (t * 200)..((t + 1) * 200) {
                                let value = black_box(i);
                                unsafe {
                                    let _protected = hazard_pointer.protect(&atomic_ptr);
                                    queue_clone.enqueue(value);
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

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let _domain = Domain::global();
                let queue = Arc::new(LockFreeQueue::new());
                for i in 0..(threads * 200) {
                    queue.enqueue(i);
                }
                b.iter(|| {
                    let mut handles = vec![];
                    for t in 0..threads {
                        let queue_clone = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let mut hazard_pointer = HazardPointer::new();
                            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
                            for _ in (t * 200)..((t + 1) * 200) {
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

criterion_group!(benches, bench_atomic_queue_multi_threaded, bench_lock_free_queue_multi_threaded);
criterion_main!(benches);
