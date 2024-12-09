use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use crossbeam::queue::SegQueue;
use seize::structures::lock_free_hash::LockFreeHashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

fn bench_lock_free_hash_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Operations");

    // Single-threaded insert benchmark with varying sizes
    for &size in &[1, 10, 100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("Insert Single-threaded", size), &size, |b, &size| {
            let map = LockFreeHashMap::new();
            b.iter(|| {
                for i in 0..size {
                    let key = black_box(i);
                    let value = black_box(i * 10);
                    map.insert(key, value);
                }
            });
        });
    }

    // Multi-threaded insert benchmark with varying thread counts
    for &threads in &[2, 4, 8, 16, 32] {
        group.bench_with_input(BenchmarkId::new("Insert Multi-threaded", threads), &threads, |b, &threads| {
            let map = Arc::new(LockFreeHashMap::new());
            b.iter(|| {
                let mut handles = vec![];
                for t in 0..threads {
                    let map_clone = Arc::clone(&map);
                    handles.push(thread::spawn(move || {
                        for i in (t * 1_000)..((t + 1) * 1_000) {
                            let key = black_box(i);
                            let value = black_box(i * 10);
                            map_clone.insert(key, value);
                        }
                    }));
                }
                for handle in handles {
                    handle.join().unwrap();
                }
            });
        });
    }

    // Benchmark for get operation with varying sizes
    for &size in &[1, 10, 100, 1_000, 10_000] {
        let map = LockFreeHashMap::new();
        for i in 0..size {
            map.insert(i, i * 10);
        }
        group.bench_with_input(BenchmarkId::new("Get Operation", size), &size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let key = black_box(i);
                    black_box(map.get(&key));
                }
            });
        });
    }

    // Benchmark for remove operation
    group.bench_function("Remove Operation", |b| {
        let map = LockFreeHashMap::new();
        for i in 0..10_000 {
            map.insert(i, i * 10);
        }
        b.iter(|| {
            for i in 0..10_000 {
                let key = black_box(i);
                black_box(map.remove(&key));
            }
        });
    });

    group.finish();
}

fn bench_crossbeam_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crossbeam Queue Operations");

    // Benchmark for enqueue and dequeue
    group.bench_function("Enqueue and Dequeue", |b| {
        let queue = Arc::new(SegQueue::new());
        b.iter(|| {
            let enqueue_queue = Arc::clone(&queue); // Clone for enqueue thread
            let dequeue_queue = Arc::clone(&queue); // Clone for dequeue thread
    
            let enqueue = thread::spawn(move || {
                for i in 0..10_000 {
                    enqueue_queue.push(black_box(i));
                }
            });
    
            let dequeue = thread::spawn(move || {
                for _ in 0..10_000 {
                    dequeue_queue.pop();
                }
            });
    
            enqueue.join().unwrap();
            dequeue.join().unwrap();
        });
    });

    group.finish();
}

fn bench_reference_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Reference Counting Operations");

    // Benchmark for Arc usage
    group.bench_function("Arc Cloning", |b| {
        let value = Arc::new(black_box(42));
        b.iter(|| {
            let _clone = Arc::clone(&value);
        });
    });

    group.finish();
}

fn bench_hazard_pointer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hazard Pointer Operations");
    let counter = AtomicUsize::new(0);

    group.bench_function("Hazard Pointer Counter", |b| {
        b.iter(|| {
            let _value = counter.fetch_add(1, Ordering::SeqCst);
        });
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Usage");

    group.bench_function("LockFreeHashMap Memory", |b| {
        b.iter(|| {
            let map = LockFreeHashMap::new();
            for i in 0..10_000 {
                map.insert(i, i * 10);
            }
            black_box(map);
        });
    });

    group.bench_function("Standard HashMap Memory", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10_000 {
                map.insert(i, i * 10);
            }
            black_box(map);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lock_free_hash_map,
    bench_crossbeam_queue,
    bench_reference_counting,
    bench_hazard_pointer,
    bench_memory_usage
);
criterion_main!(benches);
