use std::sync::{Arc, Barrier};
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;


use seize::structures::lock_free_link_list::LockFreeList;
use seize::structures::lock_free_hash::LockFreeHashMap;

use seize::Collector;
use crossbeam_epoch as epoch;
use haphazard::{Domain, HazardPointer};
use std::sync::atomic::AtomicPtr;

const ITEMS: usize = 200;
const THREAD_COUNTS: [usize; 5] = [ 4, 8, 16, 32, 64];

// ============================== bench_lock_free_list_insert ==============================

// A helper function that benchmarks a given operation for a given variant and thread count.
fn run_list_operation_test<T, F>(list: &Arc<LockFreeList<T>>, threads: usize, operation: F)
where
    T: From<usize> + Send + Sync + 'static,
    F: Fn(&LockFreeList<T>, T) + Send + Sync + 'static + Clone,
{
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let list = Arc::clone(list);
            let barrier = Arc::clone(&barrier);
            let operation = operation.clone();
            thread::spawn(move || {
                barrier.wait();
                for i in 0..ITEMS {
                    operation(&list, T::from(i));
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}



// Benchmarks Insert operation for all reclamation schemes
fn bench_lock_free_list_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Insert Comparison");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Insert
        group.bench_with_input(BenchmarkId::new("Ref Counting Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                run_list_operation_test(&list, threads, |list, i: usize| {
                    list.insert(i);
                });
            });
        });

        // Seize variant - Insert
        group.bench_with_input(BenchmarkId::new("Seize Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let list = Arc::new(LockFreeList::new());
                let _guard = collector.enter();
                run_list_operation_test(&list, threads, |list, i: usize| {
                    list.insert(i);
                });
            });
        });

        // Crossbeam Epoch variant - Insert
        group.bench_with_input(BenchmarkId::new("Crossbeam Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                let _guard = epoch::pin();
                run_list_operation_test(&list, threads, |list, i: usize| {
                    list.insert(i);
                });
            });
        });

        // Hazard Pointer variant - Insert
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let list = Arc::new(LockFreeList::new());
                run_list_operation_test(&list, threads, |list, i: usize| {
                    list.insert(i);
                });
            });
        });

        // No Memory Management variant - Insert
        group.bench_with_input(BenchmarkId::new("No Memory Management Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                run_list_operation_test(&list, threads, |list, i: usize| {
                    list.insert(i);
                });
            });
        });
    }

    group.finish();
}



// Benchmarks Mixed operation for all reclamation schemes with 90% contains and 10% insert/remove
fn bench_lock_free_list_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Mixed Operations Comparison");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Mixed
        group.bench_with_input(BenchmarkId::new("Ref Counting Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    if i % 10 == 0 {
                        list.insert(i);
                    } else if i % 10 == 1 {
                        list.remove(&i);
                    } else {
                        list.contains(&i);
                    }
                });
            });
        });

        // Seize variant - Mixed
        group.bench_with_input(BenchmarkId::new("Seize Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let list = Arc::new(LockFreeList::new());
                let _guard = collector.enter();
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    if i % 10 == 0 {
                        list.insert(i);
                    } else if i % 10 == 1 {
                        list.remove(&i);
                    } else {
                        list.contains(&i);
                    }
                });
            });
        });

        // Crossbeam Epoch variant - Mixed
        group.bench_with_input(BenchmarkId::new("Crossbeam Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                let _guard = epoch::pin();
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    if i % 10 == 0 {
                        list.insert(i);
                    } else if i % 10 == 1 {
                        list.remove(&i);
                    } else {
                        list.contains(&i);
                    }
                });
            });
        });

        // Hazard Pointer variant - Mixed
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let list = Arc::new(LockFreeList::new());
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    if i % 10 == 0 {
                        list.insert(i);
                    } else if i % 10 == 1 {
                        list.remove(&i);
                    } else {
                        list.contains(&i);
                    }
                });
            });
        });

        // No Memory Management variant - Mixed
        group.bench_with_input(BenchmarkId::new("No Memory Management Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    if i % 10 == 0 {
                        list.insert(i);
                    } else if i % 10 == 1 {
                        list.remove(&i);
                    } else {
                        list.contains(&i);
                    }
                });
            });
        });
    }

    group.finish();
}


// Benchmarks Contains operation for all reclamation schemes
fn bench_lock_free_list_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Contains Comparison");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Contains
        group.bench_with_input(BenchmarkId::new("Ref Counting Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    list.contains(&i);
                });
            });
        });

        // Seize variant - Contains
        group.bench_with_input(BenchmarkId::new("Seize Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let list = Arc::new(LockFreeList::new());
                let _guard = collector.enter();
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    list.contains(&i);
                });
            });
        });

        // Crossbeam Epoch variant - Contains
        group.bench_with_input(BenchmarkId::new("Crossbeam Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                let _guard = epoch::pin();
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    list.contains(&i);
                });
            });
        });

        // Hazard Pointer variant - Contains
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let list = Arc::new(LockFreeList::new());
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    list.contains(&i);
                });
            });
        });

        // No Memory Management variant - Contains
        group.bench_with_input(BenchmarkId::new("No Memory Management Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::new());
                for i in 0..ITEMS {
                    list.insert(i);
                }
                run_list_operation_test(&list, threads, |list, i| {
                    list.contains(&i);
                });
            });
        });
    }

    group.finish();
}


// ============================== bench_lock_free_hash_map_all ==============================

// A helper function that benchmarks a given operation for a given variant and thread count.
fn run_hash_map_operation_test<F>(map: &Arc<LockFreeHashMap<i32, i32>>, threads: usize, operation: F)
where
    F: Fn(&LockFreeHashMap<i32, i32>, i32) + Send + Sync + 'static + Clone,
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
                    operation(&map, i as i32);
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}
// Benchmarks Insert operation for all reclamation schemes
fn bench_lock_free_hash_map_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Insert Comparison");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Insert
        group.bench_with_input(BenchmarkId::new("Ref Counting Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.insert(i, i);
                });
            });
        });

        // Seize variant - Insert
        group.bench_with_input(BenchmarkId::new("Seize Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = collector.enter();
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.insert(i, i);
                });
            });
        });

        // Crossbeam Epoch variant - Insert
        group.bench_with_input(BenchmarkId::new("Crossbeam Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = epoch::pin();
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.insert(i, i);
                });
            });
        });

        // Hazard Pointer variant - Insert
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let map = Arc::new(LockFreeHashMap::new());
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.insert(i, i);
                });
            });
        });

        // No Memory Management variant - Insert
        group.bench_with_input(BenchmarkId::new("No Memory Management Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.insert(i, i);
                });
            });
        });
    }

    group.finish();
}


// Benchmarks Remove operation for all reclamation schemes
fn bench_lock_free_hash_map_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Remove Operations");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Remove
        group.bench_with_input(BenchmarkId::new("Ref Counting Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.remove(&i);
                });
            });
        });

        // Seize variant - Remove
        group.bench_with_input(BenchmarkId::new("Seize Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = collector.enter();
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.remove(&i);
                });
            });
        });

        // Crossbeam Epoch variant - Remove
        group.bench_with_input(BenchmarkId::new("Crossbeam Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = epoch::pin();
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.remove(&i);
                });
            });
        });

        // Hazard Pointer variant - Remove
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.remove(&i);
                });
            });
        });

        // No Memory Management variant - Remove
        group.bench_with_input(BenchmarkId::new("No Memory Management Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.remove(&i);
                });
            });
        });
    }

    group.finish();
}

// Benchmarks Mixed operation for all reclamation schemes with 90% contains and 10% insert/remove
fn bench_lock_free_hash_map_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Mixed Operations");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Mixed
        group.bench_with_input(BenchmarkId::new("Ref Counting Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    if i % 10 == 0 {
                        map.insert(i as i32, i as i32);
                    } else if i % 10 == 1 {
                        map.remove(&i);
                    } else {
                        map.get(&i);
                    }
                });
            });
        });

        // Seize variant - Mixed
        group.bench_with_input(BenchmarkId::new("Seize Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = collector.enter();
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    if i % 10 == 0 {
                        map.insert(i as i32, i as i32);
                    } else if i % 10 == 1 {
                        map.remove(&i);
                    } else {
                        map.get(&i);
                    }
                });
            });
        });

        // Crossbeam Epoch variant - Mixed
        group.bench_with_input(BenchmarkId::new("Crossbeam Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = epoch::pin();
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    if i % 10 == 0 {
                        map.insert(i as i32, i as i32);
                    } else if i % 10 == 1 {
                        map.remove(&i);
                    } else {
                        map.get(&i);
                    }
                });
            });
        });

        // Hazard Pointer variant - Mixed
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    if i % 10 == 0 {
                        map.insert(i as i32, i as i32);
                    } else if i % 10 == 1 {
                        map.remove(&i);
                    } else {
                        map.get(&i);
                    }
                });
            });
        });

        // No Memory Management variant - Mixed
        group.bench_with_input(BenchmarkId::new("No Memory Management Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    if i % 10 == 0 {
                        map.insert(i as i32, i as i32);
                    } else if i % 10 == 1 {
                        map.remove(&i);
                    } else {
                        map.get(&i);
                    }
                });
            });
        });
    }

    group.finish();
}

// Benchmarks Contains operation for all reclamation schemes
fn bench_lock_free_hash_map_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Contains Operations");

    for &threads in THREAD_COUNTS.iter() {
        // Reference Counting variant - Contains
        group.bench_with_input(BenchmarkId::new("Ref Counting Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.get(&i);
                });
            });
        });

        // Seize variant - Contains
        group.bench_with_input(BenchmarkId::new("Seize Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let collector = Arc::new(Collector::new());
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = collector.enter();
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.get(&i);
                });
            });
        });

        // Crossbeam Epoch variant - Contains
        group.bench_with_input(BenchmarkId::new("Crossbeam Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let _guard = epoch::pin();
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.get(&i);
                });
            });
        });

        // Hazard Pointer variant - Contains
        group.bench_with_input(BenchmarkId::new("Hazard Pointer Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let _domain = Domain::global();
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.get(&i);
                });
            });
        });

        // No Memory Management variant - Contains
        group.bench_with_input(BenchmarkId::new("No Memory Management Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                for i in 0..ITEMS {
                    map.insert(i as i32, i as i32);
                }
                run_hash_map_operation_test(&map, threads, |map, i| {
                    map.get(&i);
                });
            });
        });
    }

    group.finish();
}

// ============================== bench_lock_free_list_operations_all ==============================


fn bench_lock_free_list_operations_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Operations Comparison");

    for &threads in &[2, 4, 6, 8] {
        // Insert operation
        group.bench_with_input(BenchmarkId::new("Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::<usize>::new());
                run_list_operation(&list, threads, |list, i| {
                    list.insert(black_box(i));
                });
            });
        });

        // Contains operation
        group.bench_with_input(BenchmarkId::new("Contains", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::<usize>::new());
                for i in 0..(ITEMS) {
                    list.insert(i);
                }
                run_list_operation(&list, threads, |list, i| {
                    black_box(list.contains(&i));
                });
            });
        });

        // Remove operation
        group.bench_with_input(BenchmarkId::new("Remove", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::<usize>::new());
                for i in 0..(ITEMS) {
                    list.insert(i);
                }
                run_list_operation(&list, threads, |list, i| {
                    black_box(list.remove(&i));
                });
            });
        });

        // Mixed workload
        group.bench_with_input(BenchmarkId::new("Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::<usize>::new());
                let half = threads / 2;
                run_mixed_workload_list(&list, threads, half);
            });
        });

        // Find operation
        group.bench_with_input(BenchmarkId::new("Find", threads), &threads, |b, &threads| {
            b.iter(|| {
                let list = Arc::new(LockFreeList::<usize>::new());
                for i in 0..(ITEMS) {
                    list.insert(i);
                }
                run_list_operation(&list, threads, |list, i| {
                    assert!(list.contains(&black_box(i)));
                });
            });
        });
    }

    group.finish();
}

// Helper function to run list operations
fn run_list_operation<F>(list: &Arc<LockFreeList<usize>>, threads: usize, operation: F)
where
    F: Fn(&LockFreeList<usize>, usize) + Send + Sync + 'static + Clone,
{
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let list = Arc::clone(list);
            let barrier = Arc::clone(&barrier);
            let operation = operation.clone();
            thread::spawn(move || {
                barrier.wait();
                for i in 0..ITEMS {
                    operation(&list, i);
                }
            })
        })
        .collect();

    barrier.wait();
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_mixed_workload_list(list: &Arc<LockFreeList<usize>>, threads: usize, half: usize) {
    let barrier = Arc::new(Barrier::new(threads + 1));
    let handles: Vec<_> = (0..threads)
        .map(|i| {
            let list = Arc::clone(list);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                if i < half {
                    for i in 0..ITEMS {
                        list.insert(i);
                    }
                } else {
                    for i in 0..ITEMS {
                        let _ = list.contains(&i);
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



// ============================== Criterion Group and Main ==============================

criterion_group!(
    benches,
    // LockFreeList benchmarks
    // bench_lock_free_list_operations_all,
    // bench_lock_free_list_insert,
    // bench_lock_free_list_contains,
    // bench_lock_free_list_mixed,


    // // LockFreeHashMap benchmarks
    bench_lock_free_hash_map_insert,
    // bench_lock_free_hash_map_contains,
    bench_lock_free_hash_map_remove,
    // bench_lock_free_hash_map_mixed
);


criterion_main!(benches);