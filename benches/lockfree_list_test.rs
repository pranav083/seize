// Benchmarking file
// Import the necessary crates
use std::sync::{Arc, Barrier};
use std::thread;
use criterion::{criterion_group, criterion_main, Criterion};
use seize::structures::lock_free_link_list::LockFreeList;
// use seize::Collector;
use std::time::Instant;
use crossbeam::queue::SegQueue;

const THREADS: usize = 4;
const ITEMS: usize = 100;

// Benchmark for scalability
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Scalability");

    group.bench_function("Scalability", |b| {
        b.iter(run_scalability)
    });

    group.finish();
}

// Benchmark for latency
fn bench_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Latency");

    group.bench_function("Latency", |b| {
        b.iter(run_latency)
    });

    group.finish();
}

// Benchmark for throughput
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Throughput");

    group.bench_function("Throughput", |b| {
        b.iter(run_throughput)
    });

    group.finish();
}

// Benchmark for hazard pointers
fn bench_hazard_pointer(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hazard Pointer");
    let list = Arc::new(LockFreeList::new());

    group.bench_function("Hazard Pointer Overhead", |b| {
        b.iter(|| {
            let value = 42;
            list.insert(value);
            assert!(list.contains(&value));
            list.remove(&value);
        })
    });

    group.finish();
}

// Benchmark for reference counting
fn bench_reference_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Reference Counting");
    let list = Arc::new(LockFreeList::new());

    group.bench_function("Reference Counting Overhead", |b| {
        b.iter(|| {
            let value = Arc::new(42);
            list.insert(*value);
            assert!(list.contains(&*value));
            list.remove(&*value);
        })
    });

    group.finish();
}

// Benchmark for memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Usage");
    let list = Arc::new(LockFreeList::new());

    group.bench_function("Memory Footprint", |b| {
        b.iter(|| {
            for i in 0..ITEMS {
                list.insert(i);
            }
            for i in 0..ITEMS {
                list.remove(&i);
            }
        })
    });

    group.finish();
}

// Benchmark for crossbeam integration
fn bench_crossbeam_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crossbeam Queue");
    let queue = Arc::new(SegQueue::new());

    group.bench_function("Crossbeam Queue Performance", |b| {
        b.iter(|| {
            for i in 0..ITEMS {
                queue.push(i);
            }
            for _ in 0..ITEMS {
                queue.pop().unwrap();
            }
        })
    });

    group.finish();
}

// Scalability benchmark
fn run_scalability() {
    let list = Arc::new(LockFreeList::new());
    let barrier = Arc::new(Barrier::new(THREADS));

    let handles = (0..THREADS).map(|_| {
        let list = list.clone();
        let barrier = barrier.clone();

        thread::spawn(move || {
            barrier.wait();
            for i in 0..ITEMS {
                list.insert(i);
                if i % 2 == 0 {
                    list.remove(&i);
                }
            }
        })
    }).collect::<Vec<_>>();

    barrier.wait();

    for handle in handles {
        handle.join().unwrap();
    }
}

// Latency benchmark
fn run_latency() {
    let list = LockFreeList::new();
    let start = Instant::now();

    for i in 0..ITEMS {
        list.insert(i);
        list.contains(&i);
        list.remove(&i);
    }

    let duration = start.elapsed();
    println!("Latency: {:?} per operation", duration / ITEMS as u32);
}

// Throughput benchmark
fn run_throughput() {
    let list = Arc::new(LockFreeList::new());
    let barrier = Arc::new(Barrier::new(THREADS));

    let handles = (0..THREADS).map(|_| {
        let list = list.clone();
        let barrier = barrier.clone();

        thread::spawn(move || {
            barrier.wait();
            for i in 0..ITEMS {
                list.insert(i);
                list.remove(&i);
            }
        })
    }).collect::<Vec<_>>();

    barrier.wait();

    for handle in handles {
        handle.join().unwrap();
    }
}

criterion_group!(
    benches,
    // bench_scalability,
    // bench_latency,
    // bench_throughput,
    bench_hazard_pointer,
    bench_reference_counting,
    bench_memory_usage,
    bench_crossbeam_queue
);
criterion_main!(benches);
