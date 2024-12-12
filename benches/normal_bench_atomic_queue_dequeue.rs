use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use seize::Collector;
use seize::LockFreeQueue;
use crossbeam_epoch as epoch;
use haphazard::HazardPointer;
use std::sync::Arc;
use std::hint::black_box;
use std::sync::atomic::AtomicPtr;
use seize::structures::atomic_queue::AtomicQueue;

fn benchmark_atomic_queue_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("Atomic Queue Single-threaded");
    
    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Single-threaded (Ref Counting)", size),
            &size,
            |b, &size| {
                let queue = Arc::new(AtomicQueue::new());
                queue.enqueue(1);
                b.iter(|| {
                    for i in 0..size {
                        black_box(queue.dequeue());
                    }
                });
            }
        );
    }
    
    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Single-threaded (Seize)", size),
            &size,
            |b, &size| {
                let collector = Collector::new();
                let queue = AtomicQueue::new();
                queue.enqueue(1);
                b.iter(|| {
                    for i in 0..size {
                        let _guard = collector.enter();
                        black_box(queue.dequeue());
                    }
                });
            }
        );
    }
    
    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Single-threaded (Crossbeam Epoch)", size),
            &size,
            |b, &size| {
                let queue = AtomicQueue::new();
            queue.enqueue(1);
                b.iter(|| {
                    for i in 0..size {
                        let _guard = epoch::pin();
                        black_box(queue.dequeue());
                    }
                });
            }
        );
    }

    for &size in &[200, 400, 600, 800, 1_000] {
        group.bench_with_input(
            BenchmarkId::new("Dequeue Single-threaded (Hazard Pointer)", size),
            &size,
            |b, &size| {
                let queue = AtomicQueue::new();
                queue.enqueue(1);
                b.iter(|| {
                    for i in 0..size {
                        let _hazard = HazardPointer::new();
                        black_box(queue.dequeue());
                    }
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_atomic_queue_single_threaded);
criterion_main!(benches);