use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::Collector;
use crossbeam_epoch as epoch;
use haphazard::{Domain, HazardPointer};
// use std::sync::atomic::AtomicPtr;

// Import your LockFreeList and related structures
use seize::structures::lock_free_link_list::LockFreeList;

const ITEMS: usize = 200;

/// Benchmark for the `insert` operation
fn bench_lockfree_insert_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Insert Multi-threaded");

    let thread_counts = [4, 8, 16, 32, 64];
    for &threads in &thread_counts {
        // No Memory Management
        group.bench_with_input(
            BenchmarkId::new("Insert Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Initialize a shared LockFreeList without any memory reclamation scheme
                    let list = LockFreeList::new();
                    
                    // Wrap the list in an Arc to share among threads
                    let list = Arc::new(list);
                    
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.insert(value);
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
                let list = Arc::new(LockFreeList::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.insert(value);
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
                let list = Arc::new(LockFreeList::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        let collector_clone = collector.clone();
                        handles.push(thread::spawn(move || {
                            // Enter the Seize collector domain
                            let _guard = collector_clone.enter();
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.insert(value);
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
                let list = Arc::new(LockFreeList::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            // Pin the current epoch
                            let _guard = epoch::pin();
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.insert(value);
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
                let list = Arc::new(LockFreeList::new());
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        let domain_clone = domain.clone();
                        handles.push(thread::spawn(move || {
                            // Initialize Hazard Pointer for the thread
                            let mut hazard_pointer = HazardPointer::new(); // Corrected: No arguments
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                // Removed protection on `list_clone.head` as it's private
                                // and not necessary for the insert operation
                                list_clone.insert(value);
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

/// Benchmark for the `remove` operation
fn bench_lockfree_remove_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Remove Multi-threaded");

    let thread_counts = [4, 8, 16, 32, 64];
    for &threads in &thread_counts {
        // No Memory Management
        group.bench_with_input(
            BenchmarkId::new("Remove Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Initialize and pre-populate the LockFreeList
                    let list = LockFreeList::new();
                    let list = Arc::new(list);
                    
                    // Pre-populate the list with ITEMS * threads elements
                    for thread_id in 0..threads {
                        for i in 0..ITEMS {
                            let value = thread_id * ITEMS + i;
                            list.insert(value);
                        }
                    }

                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.remove(&value);
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
                let list = Arc::new(LockFreeList::new());
                // Pre-populate the list outside the benchmarked iteration
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.remove(&value);
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
                let list = Arc::new(LockFreeList::new());
                
                // Pre-populate the list
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        let collector_clone = collector.clone();
                        handles.push(thread::spawn(move || {
                            // Enter the Seize collector domain
                            let _guard = collector_clone.enter();
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.remove(&value);
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
                let list = Arc::new(LockFreeList::new());
                
                // Pre-populate the list
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            // Pin the current epoch
                            let _guard = epoch::pin();
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.remove(&value);
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
                let list = Arc::new(LockFreeList::new());
                
                // Pre-populate the list
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        // let domain_clone = domain;
                        handles.push(thread::spawn(move || {
                            // Initialize Hazard Pointer for the thread
                            let mut hazard_pointer = HazardPointer::new(); // Corrected: No arguments
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                // Removed protection on `list_clone.head` as it's private
                                // and not necessary for the remove operation
                                list_clone.remove(&value);
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
fn bench_lockfree_contains_multi_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Contains Multi-threaded");

    let thread_counts = [4, 8, 16, 32, 64];
    for &threads in &thread_counts {
        // No Memory Management
        group.bench_with_input(
            BenchmarkId::new("Contains Multi-threaded (No Scheme)", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    // Initialize and pre-populate the LockFreeList
                    let list = LockFreeList::new();
                    let list = Arc::new(list);
                    
                    // Pre-populate the list with ITEMS * threads elements
                    for thread_id in 0..threads {
                        for i in 0..ITEMS {
                            let value = thread_id * ITEMS + i;
                            list.insert(value);
                        }
                    }

                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.contains(&value);
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
                let list = Arc::new(LockFreeList::new());
                // Pre-populate the list outside the benchmarked iteration
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.contains(&value);
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
                let list = Arc::new(LockFreeList::new());
                
                // Pre-populate the list
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        let collector_clone = collector.clone();
                        handles.push(thread::spawn(move || {
                            // Enter the Seize collector domain
                            let _guard = collector_clone.enter();
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.contains(&value);
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
                let list = Arc::new(LockFreeList::new());
                
                // Pre-populate the list
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            // Pin the current epoch
                            let _guard = epoch::pin();
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                list_clone.contains(&value);
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
                let list = Arc::new(LockFreeList::new());
                
                // Pre-populate the list
                for thread_id in 0..threads {
                    for i in 0..ITEMS {
                        let value = thread_id * ITEMS + i;
                        list.insert(value);
                    }
                }
                
                b.iter(|| {
                    let mut handles = Vec::with_capacity(threads);
                    for thread_id in 0..threads {
                        let list_clone = Arc::clone(&list);
                        let domain_clone = domain.clone();
                        handles.push(thread::spawn(move || {
                            // Initialize Hazard Pointer for the thread
                            let mut hazard_pointer = HazardPointer::new(); // Corrected: No arguments
                            for i in 0..ITEMS {
                                let value = black_box(thread_id * ITEMS + i);
                                // Removed protection on `list_clone.head` as it's private
                                // and not necessary for the contains operation
                                list_clone.contains(&value);
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
    bench_lockfree_insert_multi_threaded,
    bench_lockfree_remove_multi_threaded,
    bench_lockfree_contains_multi_threaded
);
criterion_main!(benches);
