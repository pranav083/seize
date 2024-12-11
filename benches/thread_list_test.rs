// Updated Benchmarking File for LockFreeList
use std::sync::{Arc, Barrier};
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::structures::lock_free_link_list::LockFreeList;
use seize::Collector;
use crossbeam_epoch as epoch;
use haphazard::{Domain, HazardPointer};
use std::sync::atomic::AtomicPtr;
use std::hint::black_box;

const ITEMS: usize = 100;

// Multi-threaded Insert Performance
fn bench_lock_free_list_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Multi-threaded Insert");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Insert", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Increased the barrier count to threads + 1
                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let list = Arc::new(LockFreeList::new());

                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let list = Arc::clone(&list);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                barrier.wait();
                                for i in 0..ITEMS {
                                    list.insert(black_box(i));
                                }
                            })
                        })
                        .collect();

                    // Now the main thread also waits on the barrier
                    barrier.wait();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// Multi-threaded Remove Performance
fn bench_lock_free_list_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Multi-threaded Remove");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Remove", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let list = Arc::new(LockFreeList::new());

                    // Pre-fill the list
                    for i in 0..(threads * ITEMS) {
                        list.insert(i);
                    }

                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let list = Arc::clone(&list);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                barrier.wait();
                                for i in 0..ITEMS {
                                    list.remove(&black_box(i));
                                }
                            })
                        })
                        .collect();

                    barrier.wait();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// Reference Counting Overhead (No barrier needed here)
fn bench_lock_free_list_reference_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Reference Counting");

    group.bench_function("Reference Counting", |b| {
        b.iter(|| {
            let list = Arc::new(LockFreeList::new());

            let value = Arc::new(42);
            list.insert(*value);
            assert!(list.contains(&*value));
            list.remove(&*value);
        });
    });

    group.finish();
}

// Seize Integration
fn bench_lock_free_list_seize(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Seize");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Seize", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let collector = Arc::new(Collector::new());
                    let list = Arc::new(LockFreeList::new());
                    let barrier = Arc::new(Barrier::new(threads + 1));

                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let list = Arc::clone(&list);
                            let barrier = Arc::clone(&barrier);
                            let collector = Arc::clone(&collector);

                            thread::spawn(move || {
                                let _guard = collector.enter();
                                barrier.wait();
                                for i in 0..ITEMS {
                                    list.insert(black_box(i));
                                }
                            })
                        })
                        .collect();

                    barrier.wait();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}


// Crossbeam Epoch Integration
fn bench_lock_free_list_crossbeam_epoch(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Crossbeam Epoch");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Crossbeam Epoch", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let list = Arc::new(LockFreeList::new());
                    let barrier = Arc::new(Barrier::new(threads + 1));

                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let list = Arc::clone(&list);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                let _guard = epoch::pin();
                                barrier.wait();
                                for i in 0..ITEMS {
                                    list.insert(black_box(i));
                                }
                            })
                        })
                        .collect();

                    barrier.wait();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// Hazard Pointer Integration
fn bench_lock_free_list_hazard_pointer(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Hazard Pointer");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Hazard Pointer", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let _domain = Domain::global();
                    let list = Arc::new(LockFreeList::new());
                    let barrier = Arc::new(Barrier::new(threads + 1));

                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let list = Arc::clone(&list);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                let mut hazard_pointer = HazardPointer::new();
                                let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                                barrier.wait();
                                for i in 0..ITEMS {
                                    unsafe {
                                        let _protected = hazard_pointer.protect(&atomic_ptr);
                                        list.insert(black_box(i));
                                    }
                                }
                            })
                        })
                        .collect();

                    barrier.wait();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// Benchmark for Find and Contains
fn bench_lock_free_list_find_and_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Find and Contains");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Find and Contains", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let list = Arc::new(LockFreeList::new());

                    // Pre-fill the list
                    for i in 0..(threads * ITEMS) {
                        list.insert(i);
                    }

                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let list = Arc::clone(&list);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                barrier.wait();
                                for i in 0..ITEMS {
                                    assert!(list.contains(&black_box(i)));
                                }
                            })
                        })
                        .collect();

                    barrier.wait();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    // bench_lock_free_list_insert,
    bench_lock_free_list_remove,
    // bench_lock_free_list_reference_counting,
    // bench_lock_free_list_seize,
    // bench_lock_free_list_crossbeam_epoch,
    // bench_lock_free_list_hazard_pointer,
    // bench_lock_free_list_find_and_contains,
);
criterion_main!(benches);
