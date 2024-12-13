// benches/thread_hash_test.rs

use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::Collector;
use crossbeam_epoch as epoch;
use haphazard::{Domain, HazardPointer};


use seize::structures::lock_free_hash::LockFreeHashMap;

const ITEMS: usize = 200;

/// Type alias for LockFreeHashMap with concrete types for keys and values.
/// Adjust `usize` to other types if necessary.
type HashMapType = LockFreeHashMap<usize, usize>;

/// Benchmark for the `insert` operation
fn bench_lockfree_hash_insert_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Insert Multi-threaded");

    let thread_counts = [4, 8, 16, 32, 64];
    for &threads in &thread_counts {
        // No Memory Management
        group.bench_with_input(
            BenchmarkId::new("Insert Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Initialize a shared LockFreeHashMap without any memory reclamation scheme
                    let hash_map = HashMapType::new();

                    // Wrap the hash map in an Arc to share among threads
                    let hash_map = Arc::new(hash_map);

                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                let value = black_box(i);
                                hash_map_clone.insert(key, value);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting
        group.bench_with_input(
            BenchmarkId::new("Insert Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                // Initialize the shared LockFreeHashMap outside the benchmark iteration
                let hash_map = Arc::new(HashMapType::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                let value = black_box(i);
                                hash_map_clone.insert(key, value);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Seize
        group.bench_with_input(
            BenchmarkId::new("Insert Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let hash_map = Arc::new(HashMapType::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        let collector_clone = collector.clone();
                        handles.push(thread::spawn(move || {
                            // Enter the Seize collector domain
                            let _guard = collector_clone.enter();
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                let value = black_box(i);
                                hash_map_clone.insert(key, value);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Insert Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let hash_map = Arc::new(HashMapType::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            // Pin the current epoch
                            let _guard = epoch::pin();
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                let value = black_box(i);
                                hash_map_clone.insert(key, value);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Insert Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let domain = Domain::global();
                let hash_map = Arc::new(HashMapType::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        let domain_clone = domain.clone();
                        handles.push(thread::spawn(move || {
                            // Initialize Hazard Pointer for the thread
                            let _hazard_pointer = HazardPointer::new(); // Corrected: No arguments
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                let value = black_box(i);
                                hash_map_clone.insert(key, value);
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
}
/// Benchmark for the `remove` operation
fn bench_lockfree_hash_remove_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Remove Multi-threaded");

    let thread_counts = [4, 8, 16, 32, 64];
    for &threads in &thread_counts {
        // No Memory Management
        group.bench_with_input(
            BenchmarkId::new("Remove Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Initialize and pre-populate the LockFreeHashMap
                    let hash_map = HashMapType::new();
                    let hash_map = Arc::new(hash_map);

                    // Pre-populate the hash map with ITEMS * threads elements
                    for thread_id in 0..threads {
                        for i in 0..ITEMS {
                            let key = black_box(thread_id * ITEMS + i);
                            let value = black_box(i);
                            hash_map.insert(key, value);
                        }
                    }

                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.remove(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting
        group.bench_with_input(
            BenchmarkId::new("Remove Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let hash_map = Arc::new(HashMapType::new());
                // Pre-populate the hash map outside the benchmarked iteration
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.remove(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Seize
        group.bench_with_input(
            BenchmarkId::new("Remove Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let hash_map = Arc::new(HashMapType::new());

                // Pre-populate the hash map
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        let collector_clone = collector.clone();
                        handles.push(thread::spawn(move || {
                            // Enter the Seize collector domain
                            let _guard = collector_clone.enter();
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.remove(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Remove Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let hash_map = Arc::new(HashMapType::new());

                // Pre-populate the hash map
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            // Pin the current epoch
                            let _guard = epoch::pin();
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.remove(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Remove Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let domain = Domain::global();
                let hash_map = Arc::new(HashMapType::new());

                // Pre-populate the hash map
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        let domain_clone = domain.clone();
                        handles.push(thread::spawn(move || {
                            // Initialize Hazard Pointer for the thread
                            let _hazard_pointer = HazardPointer::new(); // Corrected: No arguments
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.remove(&key);
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

/// Benchmark for the `contains` operation
fn bench_lockfree_hash_contains_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Contains Multi-threaded");

    let thread_counts = [4, 8, 16, 32, 64];
    for &threads in &thread_counts {
        // No Memory Management
        group.bench_with_input(
            BenchmarkId::new("Contains Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Initialize and pre-populate the LockFreeHashMap
                    let hash_map = HashMapType::new();
                    let hash_map = Arc::new(hash_map);

                    // Pre-populate the hash map with ITEMS * threads elements
                    for thread_id in 0..threads {
                        for i in 0..ITEMS {
                            let key = black_box(thread_id * ITEMS + i);
                            let value = black_box(i);
                            hash_map.insert(key, value);
                        }
                    }

                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.get(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Reference Counting
        group.bench_with_input(
            BenchmarkId::new("Contains Multi-threaded (Ref Counting)", threads),
            &threads,
            |b, &threads| {
                let hash_map = Arc::new(HashMapType::new());
                // Pre-populate the hash map outside the benchmarked iteration
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.get(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Seize
        group.bench_with_input(
            BenchmarkId::new("Contains Multi-threaded (Seize)", threads),
            &threads,
            |b, &threads| {
                let collector = Collector::new();
                let hash_map = Arc::new(HashMapType::new());

                // Pre-populate the hash map
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        let collector_clone = collector.clone();
                        handles.push(thread::spawn(move || {
                            // Enter the Seize collector domain
                            let _guard = collector_clone.enter();
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.get(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Crossbeam Epoch
        group.bench_with_input(
            BenchmarkId::new("Contains Multi-threaded (Crossbeam Epoch)", threads),
            &threads,
            |b, &threads| {
                let hash_map = Arc::new(HashMapType::new());

                // Pre-populate the hash map
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        handles.push(thread::spawn(move || {
                            // Pin the current epoch
                            let _guard = epoch::pin();
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.get(&key);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // Hazard Pointer
        group.bench_with_input(
            BenchmarkId::new("Contains Multi-threaded (Hazard Pointer)", threads),
            &threads,
            |b, &threads| {
                let domain = Domain::global();
                let hash_map = Arc::new(HashMapType::new());

                // Pre-populate the hash map
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let key = black_box(thread_id * ITEMS + i);
                        let value = black_box(i);
                        hash_map.insert(key, value);
                    }
                }

                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let hash_map_clone = Arc::clone(&hash_map);
                        let domain_clone = domain.clone();
                        handles.push(thread::spawn(move || {
                            // Initialize Hazard Pointer for the thread
                            let _hazard_pointer = HazardPointer::new(); // Corrected: No arguments
                            for i in 0..ITEMS {
                                let key = black_box(thread_id * ITEMS + i);
                                hash_map_clone.get(&key);
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

criterion_group!(
    benches,
    // bench_lockfree_hash_insert_multi_threaded,
    bench_lockfree_hash_remove_multi_threaded,
    // bench_lockfree_hash_contains_multi_threaded
);
criterion_main!(benches);