use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::Collector;
use seize::LockFreeQueue;
use crossbeam_epoch as epoch;
use haphazard::HazardPointer;
use std::sync::Arc;
use std::hint::black_box;
use std::sync::atomic::AtomicPtr;
use seize::structures::atomic_queue::AtomicQueue;

fn benchmark_lockfree_queue_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free Queue Single-threaded");

    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Single-threaded (Ref Counting)", size),
            &size,
            |b, &size| {
                let queue = Arc::new(LockFreeQueue::new());
                b.iter(|| {
                    for i in 0..size {
                        black_box(queue.enqueue(i));
                    }
                });
            }
        );
    }

    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Single-threaded (Seize)", size),
            &size,
            |b, &size| {
                let collector = Collector::new();
                let queue = LockFreeQueue::new();
                b.iter(|| {
                    for i in 0..size {
                        let _guard = collector.enter();
                        black_box(queue.enqueue(i));
                    }
                });
            }
        );
    }

    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Single-threaded (Crossbeam Epoch)", size),
            &size,
            |b, &size| {
                let queue = LockFreeQueue::new();
                b.iter(|| {
                    for i in 0..size {
                        let _guard = epoch::pin();
                        black_box(queue.enqueue(i));
                    }
                });
            }
        );
    }

    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Enqueue Single-threaded (Hazard Pointer)", size),
            &size,
            |b, &size| {
                let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
                let queue = LockFreeQueue::new();
                let mut hazard_pointer = HazardPointer::new();
                b.iter(|| {
                    for i in 0..size {
                        unsafe {
                            let _protected = hazard_pointer.protect(&atomic_ptr);
                            black_box(queue.enqueue(i));
                        }
                    }
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_lockfree_queue_single_threaded);
criterion_main!(benches);