use criterion::{criterion_group, criterion_main, Criterion};
use seize::Collector;
use seize::LockFreeQueue;
use crossbeam_epoch as epoch;
use haphazard::HazardPointer;
use std::sync::Arc;
use std::hint::black_box;
use std::sync::atomic::AtomicPtr;
use seize::structures::atomic_queue::AtomicQueue;
use sysinfo::{System, SystemExt};
use std::fs::File;
use std::io::{Write, BufWriter};

fn benchmark_lockfree_queue_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free Queue Memory");
    let mut sys = System::new_all();

    // Open a CSV file for logging memory usage
    let file = File::create("lockfree_queue_memory_usage.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    // Write CSV header
    writeln!(
        writer,
        "Benchmark,Reclamation Scheme,Operation,Memory Before (KB),Memory After (KB),Memory Free Change (KB)"
    )
    .expect("Unable to write to file");

    // Reference Counting
    group.bench_function("Enqueue Memory (Reference Counting)", |b| {
        let queue = Arc::new(LockFreeQueue::new());
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(queue.enqueue(1));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,ref_counting,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Reference Counting)", |b| {
        let queue = Arc::new(LockFreeQueue::new());
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(queue.dequeue());
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,ref_counting,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    // Seize
    group.bench_function("Enqueue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = LockFreeQueue::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(queue.enqueue(1));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,seize,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = LockFreeQueue::new();
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(queue.dequeue());
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,seize,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    // Crossbeam Epoch
    group.bench_function("Enqueue Memory (Crossbeam Epoch)", |b| {
        let queue = LockFreeQueue::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(queue.enqueue(1));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,crossbeam,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Crossbeam Epoch)", |b| {
        let queue = LockFreeQueue::new();
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(queue.dequeue());
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,crossbeam,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    // Hazard Pointer
    group.bench_function("Enqueue Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let queue = LockFreeQueue::new();
        let mut hazard_pointer = HazardPointer::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(queue.enqueue(1));
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,hazard_pointer,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let queue = LockFreeQueue::new();
        let mut hazard_pointer = HazardPointer::new();
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(queue.dequeue());
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,hazard_pointer,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}

fn benchmark_atomic_queue_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("Atomic Queue Memory");
    let mut sys = System::new_all();

    let file = File::create("atomic_queue_memory_usage.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "Benchmark,Reclamation Scheme,Operation,Memory Before (KB),Memory After (KB),Memory Free Change (KB)"
    )
    .expect("Unable to write to file");

    // Reference Counting
    group.bench_function("Enqueue Memory (Reference Pointing)", |b| {
        let queue = Arc::new(AtomicQueue::new());
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(queue.enqueue(1));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "atomic_queue,ref_counting,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Reference Pointing)", |b| {
        let queue = Arc::new(AtomicQueue::new());
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(queue.dequeue());
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "atomic_queue,ref_counting,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    // Seize
    group.bench_function("Enqueue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = AtomicQueue::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(queue.enqueue(1));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "atomic_queue,seize,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = AtomicQueue::new();
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(queue.dequeue());
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "atomic_queue,seize,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    // Crossbeam
    group.bench_function("Enqueue Memory (Crossbeam Epoch)", |b| {
        let queue = AtomicQueue::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(queue.enqueue(1));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "atomic_queue,crossbeam,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Crossbeam Epoch)", |b| {
        let queue = AtomicQueue::new();
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(queue.dequeue());
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "atomic_queue,crossbeam,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    // Hazard Pointer
    group.bench_function("Enqueue Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let queue = AtomicQueue::new();
        let mut hazard_pointer = HazardPointer::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(queue.enqueue(1));
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;
    
            writeln!(
                writer,
                "atomic_queue,hazard_pointer,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });

    group.bench_function("Dequeue Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let queue = AtomicQueue::new();
        let mut hazard_pointer = HazardPointer::new();
        queue.enqueue(1);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(queue.dequeue());
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;
    
            writeln!(
                writer,
                "atomic_queue,hazard_pointer,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        });
    });
    

    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}

criterion_group!(benches, benchmark_lockfree_queue_memory, benchmark_atomic_queue_memory);
criterion_main!(benches);
