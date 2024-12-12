use criterion::{criterion_group, criterion_main, Criterion};
use seize::Collector;
use crossbeam_epoch as epoch;
use haphazard::HazardPointer;
use std::sync::Arc;
use std::hint::black_box;
use std::sync::atomic::AtomicPtr;
use sysinfo::System;
use std::fs::File;
use std::io::{Write, BufWriter};
use chrono::Utc;

use seize::structures::lock_free_hash::LockFreeHashMap;
use seize::structures::lock_free_link_list::LockFreeList;

fn benchmark_lockfree_hash_map_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeHashMap Memory");
    let mut sys = System::new_all();

    let file = File::create("lockfree_hash_map_memory_usage.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    // CSV header
    writeln!(
        writer,
        "Reclamation Scheme,Operation,Memory Change (KB)"
    ).expect("Unable to write to file");

    // Helper closure to record if memory changed
    let record_if_changed = |writer: &mut BufWriter<File>,
                             ds: &str,
                             scheme: &str,
                             op: &str,
                             before: u64,
                             after: u64,
                             change: i64| {
            // let timestamp = Utc::now().to_rfc3339();
            writeln!(
                writer,
                "{},{},{} KB",
                 scheme, op, change
            ).expect("Unable to write to file");
    };

    // Reference Counting: Insert
    group.bench_function("Insert Memory (Reference Counting)", |b| {
        let map = Arc::new(LockFreeHashMap::new());
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(map.insert(42, 42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "ref_counting", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Reference Counting: Remove
    group.bench_function("Remove Memory (Reference Counting)", |b| {
        let map = Arc::new(LockFreeHashMap::new());
        map.insert(42, 42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(map.remove(&42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "ref_counting", "remove", memory_before, memory_after, memory_change);
        });
    });

    // Seize: Insert
    group.bench_function("Insert Memory (Seize)", |b| {
        let collector = Collector::new();
        let map = LockFreeHashMap::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(map.insert(42, 42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "seize", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Seize: Remove
    group.bench_function("Remove Memory (Seize)", |b| {
        let collector = Collector::new();
        let map = LockFreeHashMap::new();
        map.insert(42, 42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(map.remove(&42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "seize", "remove", memory_before, memory_after, memory_change);
        });
    });

    // Crossbeam Epoch: Insert
    group.bench_function("Insert Memory (Crossbeam Epoch)", |b| {
        let map = LockFreeHashMap::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(map.insert(42, 42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "crossbeam", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Crossbeam Epoch: Remove
    group.bench_function("Remove Memory (Crossbeam Epoch)", |b| {
        let map = LockFreeHashMap::new();
        map.insert(42, 42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(map.remove(&42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "crossbeam", "remove", memory_before, memory_after, memory_change);
        });
    });

    // Hazard Pointer: Insert
    group.bench_function("Insert Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let map = LockFreeHashMap::new();
        let mut hazard_pointer = HazardPointer::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(map.insert(42, 42));
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "hazard_pointer", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Hazard Pointer: Remove
    group.bench_function("Remove Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let map = LockFreeHashMap::new();
        let mut hazard_pointer = HazardPointer::new();
        map.insert(42, 42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(map.remove(&42));
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_hash_map", "hazard_pointer", "remove", memory_before, memory_after, memory_change);
        });
    });

    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}

fn benchmark_lockfree_list_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("LockFreeList Memory");
    let mut sys = System::new_all();

    let file = File::create("lockfree_list_memory_usage.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    // CSV header
    writeln!(
        writer,
        "Reclamation Scheme,Operation,Memory Change (KB)"
    ).expect("Unable to write to file");
    let record_if_changed = |writer: &mut BufWriter<File>,
                             ds: &str,
                             scheme: &str,
                             op: &str,
                             before: u64,
                             after: u64,
                             change: i64| {

        writeln!(
            writer,
            "{},{},{} KB",
                scheme, op, change
        ).expect("Unable to write to file");
    };

    // Reference Counting: Insert
    group.bench_function("Insert Memory (Reference Counting)", |b| {
        let list = Arc::new(LockFreeList::new());
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(list.insert(42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "ref_counting", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Reference Counting: Remove
    group.bench_function("Remove Memory (Reference Counting)", |b| {
        let list = Arc::new(LockFreeList::new());
        list.insert(42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            black_box(list.remove(&42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "ref_counting", "remove", memory_before, memory_after, memory_change);
        });
    });

    // Seize: Insert
    group.bench_function("Insert Memory (Seize)", |b| {
        let collector = Collector::new();
        let list = LockFreeList::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(list.insert(42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "seize", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Seize: Remove
    group.bench_function("Remove Memory (Seize)", |b| {
        let collector = Collector::new();
        let list = LockFreeList::new();
        list.insert(42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = collector.enter();
            black_box(list.remove(&42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "seize", "remove", memory_before, memory_after, memory_change);
        });
    });

    // Crossbeam Epoch: Insert
    group.bench_function("Insert Memory (Crossbeam Epoch)", |b| {
        let list = LockFreeList::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(list.insert(42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "crossbeam", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Crossbeam Epoch: Remove
    group.bench_function("Remove Memory (Crossbeam Epoch)", |b| {
        let list = LockFreeList::new();
        list.insert(42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            let _guard = epoch::pin();
            black_box(list.remove(&42));
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "crossbeam", "remove", memory_before, memory_after, memory_change);
        });
    });

    // Hazard Pointer: Insert
    group.bench_function("Insert Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let list = LockFreeList::new();
        let mut hazard_pointer = HazardPointer::new();
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(list.insert(42));
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "hazard_pointer", "insert", memory_before, memory_after, memory_change);
        });
    });

    // Hazard Pointer: Remove
    group.bench_function("Remove Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let list = LockFreeList::new();
        let mut hazard_pointer = HazardPointer::new();
        list.insert(42);
        b.iter(|| {
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            unsafe {
                let _protected = hazard_pointer.protect(&atomic_ptr);
                black_box(list.remove(&42));
            }
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            record_if_changed(&mut writer, "lockfree_list", "hazard_pointer", "remove", memory_before, memory_after, memory_change);
        });
    });

    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}

criterion_group!(
    benches,
    benchmark_lockfree_hash_map_memory,
    benchmark_lockfree_list_memory
);
criterion_main!(benches);
