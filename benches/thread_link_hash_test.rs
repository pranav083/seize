use std::sync::{Arc, Barrier};
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::hash::Hash;


use seize::structures::lock_free_link_list::LockFreeList;
use seize::structures::lock_free_hash::LockFreeHashMap;

use seize::Collector;
use crossbeam_epoch as epoch;
use haphazard::{Domain, HazardPointer};
use std::sync::atomic::AtomicPtr;

const ITEMS: usize = 100;

// ============================== LockFreeList Benchmarks ==============================

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
        group.bench_with_input(BenchmarkId::new("Remove", threads), &threads, |b, &threads| {
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
        });
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

// ============================== LockFreeHashMap Benchmarks ==============================

// Multi-threaded Insert Performance
fn bench_lock_free_hash_map_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Multi-threaded Insert");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let barrier = Arc::new(Barrier::new(threads + 1));
                let map = Arc::new(LockFreeHashMap::new());

                let handles: Vec<_> = (0..threads)
                    .map(|_| {
                        let map = Arc::clone(&map);
                        let barrier = Arc::clone(&barrier);
                        thread::spawn(move || {
                            barrier.wait();
                            for i in 0..ITEMS {
                                map.insert(black_box(i), black_box(i));
                            }
                        })
                    })
                    .collect();

                barrier.wait();
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

// Multi-threaded Get Performance
fn bench_lock_free_hash_map_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Multi-threaded Get");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Get", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..(threads * ITEMS) {
                    map.insert(i, i);
                }

                let barrier = Arc::new(Barrier::new(threads + 1));
                let handles: Vec<_> = (0..threads)
                    .map(|_| {
                        let map = Arc::clone(&map);
                        let barrier = Arc::clone(&barrier);
                        thread::spawn(move || {
                            barrier.wait();
                            for i in 0..ITEMS {
                                black_box(map.get(&i));
                            }
                        })
                    })
                    .collect();

                barrier.wait();
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

// Multi-threaded Remove Performance
fn bench_lock_free_hash_map_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Multi-threaded Remove");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..(threads * ITEMS) {
                    map.insert(i, i);
                }

                let barrier = Arc::new(Barrier::new(threads + 1));
                let handles: Vec<_> = (0..threads)
                    .map(|_| {
                        let map = Arc::clone(&map);
                        let barrier = Arc::clone(&barrier);
                        thread::spawn(move || {
                            barrier.wait();
                            for i in 0..ITEMS {
                                black_box(map.remove(&i));
                            }
                        })
                    })
                    .collect();

                barrier.wait();
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

// Mixed Workload: Half inserting, half getting
fn bench_lock_free_hash_map_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Mixed Workload");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let barrier = Arc::new(Barrier::new(threads + 1));
                let half = threads / 2;

                let mut handles = Vec::with_capacity(threads);

                // Insert threads
                for _ in 0..half {
                    let map = Arc::clone(&map);
                    let barrier = Arc::clone(&barrier);
                    handles.push(thread::spawn(move || {
                        barrier.wait();
                        for i in 0..ITEMS {
                            map.insert(black_box(i), black_box(i));
                        }
                    }));
                }

                // Get threads
                for _ in half..threads {
                    let map = Arc::clone(&map);
                    let barrier = Arc::clone(&barrier);
                    handles.push(thread::spawn(move || {
                        barrier.wait();
                        for i in 0..ITEMS {
                            let _ = map.get(&i);
                        }
                    }));
                }

                barrier.wait();
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

// Seize Integration with LockFreeHashMap
fn bench_lock_free_hash_map_seize(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Seize");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Seize", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let map = Arc::new(LockFreeHashMap::new());
                let barrier = Arc::new(Barrier::new(threads + 1));

                let handles: Vec<_> = (0..threads)
                    .map(|_| {
                        let map = Arc::clone(&map);
                        let barrier = Arc::clone(&barrier);
                        let collector = Arc::clone(&collector);

                        thread::spawn(move || {
                            let _guard = collector.enter();
                            barrier.wait();
                            for i in 0..ITEMS {
                                map.insert(black_box(i), black_box(i));
                            }
                        })
                    })
                    .collect();

                barrier.wait();

                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    group.finish();
}

// Crossbeam Epoch Integration with LockFreeHashMap
fn bench_lock_free_hash_map_crossbeam_epoch(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Crossbeam Epoch");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Crossbeam Epoch", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(LockFreeHashMap::new());
                    let barrier = Arc::new(Barrier::new(threads + 1));

                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let map = Arc::clone(&map);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                let _guard = epoch::pin();
                                barrier.wait();
                                for i in 0..ITEMS {
                                    map.insert(black_box(i), black_box(i));
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

// Hazard Pointer Integration with LockFreeHashMap
fn bench_lock_free_hash_map_hazard_pointer(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Hazard Pointer");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Hazard Pointer", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let _domain = Domain::global();
                    let map = Arc::new(LockFreeHashMap::new());
                    let barrier = Arc::new(Barrier::new(threads + 1));

                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let map = Arc::clone(&map);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                let mut hazard_pointer = HazardPointer::new();
                                let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                                barrier.wait();
                                for i in 0..ITEMS {
                                    unsafe {
                                        let _protected = hazard_pointer.protect(&atomic_ptr);
                                        map.insert(black_box(i), black_box(i));
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

// Benchmark for Find and Contains (using `get` as a contains check)
fn bench_lock_free_hash_map_find_and_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Find and Contains");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("Find and Contains", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(LockFreeHashMap::new());

                    // Pre-fill the map
                    for i in 0..(threads * ITEMS) {
                        map.insert(i, i);
                    }

                    let barrier = Arc::new(Barrier::new(threads + 1));
                    let handles: Vec<_> = (0..threads)
                        .map(|_| {
                            let map = Arc::clone(&map);
                            let barrier = Arc::clone(&barrier);
                            thread::spawn(move || {
                                barrier.wait();
                                for i in 0..ITEMS {
                                    // Check if value is present (similar to contains)
                                    assert!(map.get(&black_box(i)).is_some());
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


// ============================== bench_lock_free_hash_map_operation_all ==============================

fn bench_lock_free_hash_map_operation_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Operations Comparison");

    for &threads in &[2, 4, 6, 8] {
        // Insert operation
        group.bench_with_input(BenchmarkId::new("Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::<usize, usize>::new());
                run_hash_map_operation(&map, threads, |map, i| {
                    map.insert(black_box(i), black_box(i));
                });
            });
        });

        // Get operation
        group.bench_with_input(BenchmarkId::new("Get", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::<usize, usize>::new());
                for i in 0..(threads * ITEMS) {
                    map.insert(i, i);
                }
                run_hash_map_operation(&map, threads, |map, i| {
                    black_box(map.get(&i));
                });
            });
        });

        // Remove operation
        group.bench_with_input(BenchmarkId::new("Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::<usize, usize>::new());
                for i in 0..(threads * ITEMS) {
                    map.insert(i, i);
                }
                run_hash_map_operation(&map, threads, |map, i| {
                    black_box(map.remove(&i));
                });
            });
        });

        // Mixed workload
        group.bench_with_input(BenchmarkId::new("Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::<usize, usize>::new());
                let half = threads / 2;
                run_mixed_workload(&map, threads, half);
            });
        });

        // Find and Contains operation
        group.bench_with_input(BenchmarkId::new("Find and Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::<usize, usize>::new());
                for i in 0..(threads * ITEMS) {
                    map.insert(i, i);
                }
                run_hash_map_operation(&map, threads, |map, i| {
                    assert!(map.get(&black_box(i)).is_some());
                });
            });
        });
    }

    group.finish();
}


// Helper function to run hash map operations
fn run_hash_map_operation<K, V, F>(map: &Arc<LockFreeHashMap<K, V>>, threads: usize, operation: F)
where
    K: Send + Sync + 'static + From<usize>,
    V: Send + Sync + 'static,
    F: Fn(&LockFreeHashMap<K, V>, K) + Send + Sync + 'static + Clone,
{
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let map = Arc::clone(map);
            let barrier = Arc::clone(&barrier);
            let operation = operation.clone();
            thread::spawn(move || {
                barrier.wait();
                for i in 0..ITEMS {
                    operation(&map, K::from(i));
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}
fn run_mixed_workload(map: &Arc<LockFreeHashMap<usize, usize>>, threads: usize, half: usize) {
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|i| {
            let map = Arc::clone(map);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                if i < half {
                    for i in 0..ITEMS {
                        map.insert(i, i);
                    }
                } else {
                    for i in 0..ITEMS {
                        let _ = map.get(&i);
                    }
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}

// ============================== bench_lock_free_hash_map_all ==============================

// A helper function that benchmarks insert for a given variant and thread count.
fn run_hash_map_insert_test(map: &Arc<LockFreeHashMap<i32, i32>>, threads: usize) {
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let map = Arc::clone(map);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                for i in 0..ITEMS {
                    map.insert(black_box(i as i32), black_box(i as i32));
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}

// This function runs all variants (Ref Counting, Seize, Crossbeam, Hazard Pointer) 
// in the same benchmark group for a lock-free hash map.
fn bench_lock_free_hash_map_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Insert Comparison");

    // These could be thread counts, input sizes, or any parameter you vary.
    for &threads in &[2, 4, 6, 8] {
        // Reference Counting variant
        group.bench_with_input(BenchmarkId::new("Ref Counting", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                run_hash_map_insert_test(&map, threads);
            });
        });

        // Seize variant
        group.bench_with_input(BenchmarkId::new("Seize", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = collector.enter();
                run_hash_map_insert_test(&map, threads);
            });
        });

        // Crossbeam Epoch variant
        group.bench_with_input(BenchmarkId::new("Crossbeam Epoch", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = epoch::pin();
                run_hash_map_insert_test(&map, threads);
            });
        });

        // Hazard Pointer variant
        group.bench_with_input(BenchmarkId::new("Hazard Pointer", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let map = Arc::new(LockFreeHashMap::new());
                let mut hazard_pointer = HazardPointer::new();
                let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                // Protect some pointer, ensuring hazard pointers are used
                unsafe { hazard_pointer.protect(&atomic_ptr); }
                run_hash_map_insert_test(&map, threads);
            });
        });
    }

    group.finish();
}

// ============================== bench_lock_free_list_all ==============================

// A helper function that benchmarks insert for a given variant.
fn run_insert_test(list: &Arc<LockFreeList<i32>>, threads: usize) {
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let list = Arc::clone(list);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                for i in 0..ITEMS {
                    list.insert(black_box(i as i32));
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}

fn bench_lock_free_list_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Insert Comparison");

    // Define the thread counts or input sizes you want to compare
    for &threads in &[2, 4, 6, 8] {
        // Reference Counting variant
        group.bench_with_input(BenchmarkId::new("Ref Counting", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                run_insert_test(&list, threads);
            });
        });

        // Seize variant
        group.bench_with_input(BenchmarkId::new("Seize", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let list = Arc::new(LockFreeList::new());
                let _guard = collector.enter(); // Enter collector scope
                run_insert_test(&list, threads);
            });
        });

        // Crossbeam Epoch variant
        group.bench_with_input(BenchmarkId::new("Crossbeam Epoch", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                let _guard = epoch::pin(); // Enter crossbeam epoch
                run_insert_test(&list, threads);
            });
        });

        // Hazard Pointer variant
        group.bench_with_input(BenchmarkId::new("Hazard Pointer", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let list = Arc::new(LockFreeList::new());
                let mut hazard_pointer = HazardPointer::new();
                let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(0)));
                // Protect some pointer, ensuring hazard pointers are used
                unsafe { hazard_pointer.protect(&atomic_ptr); }
                run_insert_test(&list, threads);
            });
        });
    }

    group.finish();
}


// ============================== Criterion Group and Main ==============================

criterion_group!(
    benches,
    // LockFreeList benchmarks

    // bench_lock_free_list_insert,
    // bench_lock_free_list_find_and_contains,
    // bench_lock_free_list_all,
    // bench_lock_free_list_remove,
    // bench_lock_free_list_reference_counting,
    // bench_lock_free_list_seize,
    // bench_lock_free_list_crossbeam_epoch,
    // bench_lock_free_list_hazard_pointer,

    // LockFreeHashMap benchmarks

    // bench_lock_free_hash_map_insert,
    // bench_lock_free_hash_map_get,
    // bench_lock_free_hash_map_remove,
    // bench_lock_free_hash_map_mixed,
    // bench_lock_free_hash_map_find_and_contains,
    // bench_lock_free_hash_map_all,
    bench_lock_free_hash_map_operation_all,
    // bench_lock_free_hash_map_seize,
    // bench_lock_free_hash_map_crossbeam_epoch,
    // bench_lock_free_hash_map_hazard_pointer,
);


criterion_main!(benches);