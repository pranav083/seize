use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::structures::lock_free_hash::LockFreeHashMap;
use std::sync::{Arc, Barrier};
use std::thread;
use std::hint::black_box;

const ITEMS: usize = 100;

// Multi-threaded Insert Performance
fn bench_lock_free_hash_map_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Multi-threaded Insert");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Insert", threads), &threads, |b, &threads| {
            b.iter(|| {
                // Create a barrier for thread synchronization: threads + 1 for main thread
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
// Here we first insert (threads * ITEMS) items to ensure they exist, then spawn threads to get them.
fn bench_lock_free_hash_map_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Multi-threaded Get");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Get", threads), &threads, |b, &threads| {
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
                                // Just black_box to prevent compiler from optimizing too much
                                let _ = black_box(map.get(&i));
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
// Similar to insert: we pre-insert items, then measure how long it takes to remove them.
fn bench_lock_free_hash_map_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Multi-threaded Remove");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Remove", threads), &threads, |b, &threads| {
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

// Benchmark Insert + Get Mixture (Optional)
// This could simulate a workload where threads insert some keys and others read them.
// Adjust as necessary.
fn bench_lock_free_hash_map_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Mixed Workload");

    for &threads in &[2, 4, 6, 8] {
        group.bench_with_input(BenchmarkId::new("Mixed", threads), &threads, |b, &threads| {
            b.iter(|| {
                let map = Arc::new(LockFreeHashMap::new());
                let barrier = Arc::new(Barrier::new(threads + 1));

                // Half threads insert, half threads get
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

criterion_group!(
    benches,
    bench_lock_free_hash_map_insert,
    bench_lock_free_hash_map_get,
    bench_lock_free_hash_map_remove,
    bench_lock_free_hash_map_mixed,
);
criterion_main!(benches);
