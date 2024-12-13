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

use seize::structures::lock_free_hash::LockFreeHashMap;
use seize::structures::lock_free_link_list::LockFreeList;
use std::time::Duration;

const BATCH_SIZE: usize = 100;
const MAX_OPERATIONS: usize = 100_000;

fn benchmark_lockfree_list_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free List Memory");
    let mut sys = System::new_all();

    // Open a CSV file for logging memory usage
    let file = File::create("lockfree_list_memory_usage.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    // Write CSV header
    writeln!(
        writer,
        "Benchmark,Reclamation Scheme,Operation,Memory Before (KB),Memory After (KB),Memory Change (KB)"
    )
    .expect("Unable to write to file");

    // No Scheme
    group.bench_function("Insert Memory (No scheme)", |b| {
        let list = LockFreeList::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(list.insert(42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,none,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (No scheme)", |b| {
        let list = LockFreeList::new();
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(list.insert(42));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(list.remove(&42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,none,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Reference Counting
    group.bench_function("Insert Memory (Reference Counting)", |b| {
        let list = Arc::new(LockFreeList::new());
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(list.insert(42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,ref_counting,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (Reference Counting)", |b| {
        let list = Arc::new(LockFreeList::new());
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(list.insert(42));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(list.remove(&42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,ref_counting,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Seize
    group.bench_function("Insert Memory (Seize)", |b| {
        let collector = Collector::new();
        let list = LockFreeList::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = collector.enter();
                for _ in 0..BATCH_SIZE {
                    black_box(list.insert(42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,seize,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (Seize)", |b| {
        let collector = Collector::new();
        let list = LockFreeList::new();
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(list.insert(42));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = collector.enter();
                for _ in 0..BATCH_SIZE {
                    black_box(list.remove(&42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,seize,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Crossbeam Epoch
    group.bench_function("Insert Memory (Crossbeam Epoch)", |b| {
        let list = LockFreeList::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();
                for _ in 0..BATCH_SIZE {
                    black_box(list.insert(42));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,crossbeam,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });
    // Hazard Pointer
    group.bench_function("Insert Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(42)));
        let list = LockFreeList::new();
        let mut hazard_pointer = HazardPointer::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    unsafe {
                        let _protected = hazard_pointer.protect(&atomic_ptr);
                        black_box(list.insert(42));
                    }
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,hazard_pointer,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(42)));
        let list = LockFreeList::new();
        let mut hazard_pointer = HazardPointer::new();
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(list.insert(42));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    unsafe {
                        let _protected = hazard_pointer.protect(&atomic_ptr);
                        black_box(list.remove(&42));
                    }
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_list,hazard_pointer,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}


fn benchmark_lockfree_hash_map_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("Lock-Free Hash Map Memory");
    let mut sys = System::new_all();

    // Open a CSV file for logging memory usage
    let file = File::create("lockfree_hash_map_memory_usage.csv").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    // Write CSV header
    writeln!(
        writer,
        "Benchmark,Reclamation Scheme,Operation,Memory Before (KB),Memory After (KB),Memory Change (KB)"
    )
    .expect("Unable to write to file");

    // No Scheme
    group.bench_function("Insert Memory (No scheme)", |b| {
        let map = LockFreeHashMap::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(map.insert(1, 1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,none,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (No scheme)", |b| {
        let map = LockFreeHashMap::new();
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(map.insert(1, 1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(map.remove(&1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,none,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Reference Counting
    group.bench_function("Insert Memory (Reference Counting)", |b| {
        let map = Arc::new(LockFreeHashMap::new());
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(map.insert(1, 1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,ref_counting,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (Reference Counting)", |b| {
        let map = Arc::new(LockFreeHashMap::new());
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(map.insert(1, 1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(map.remove(&1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,ref_counting,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Seize
    group.bench_function("Insert Memory (Seize)", |b| {
        let collector = Collector::new();
        let map = LockFreeHashMap::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = collector.enter();
                for _ in 0..BATCH_SIZE {
                    black_box(map.insert(1, 1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,seize,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Hazard Pointer
    group.bench_function("Insert Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new((1, 1))));
        let map = LockFreeHashMap::new();
        let mut hazard_pointer = HazardPointer::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    unsafe {
                        let _protected = hazard_pointer.protect(&atomic_ptr);
                        black_box(map.insert(1, 1));
                    }
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,hazard_pointer,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new((1, 1))));
        let map = LockFreeHashMap::new();
        let mut hazard_pointer = HazardPointer::new();
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(map.insert(1, 1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    unsafe {
                        let _protected = hazard_pointer.protect(&atomic_ptr);
                        black_box(map.remove(&1));
                    }
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,hazard_pointer,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Crossbeam Epoch
    group.bench_function("Insert Memory (Crossbeam Epoch)", |b| {
        let map = LockFreeHashMap::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();
                for _ in 0..BATCH_SIZE {
                    black_box(map.insert(1, 1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,crossbeam,insert,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Remove Memory (Crossbeam Epoch)", |b| {
        let map = LockFreeHashMap::new();
        for _ in 0..(BATCH_SIZE * 100) {
            black_box(map.insert(1, 1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();
                for _ in 0..BATCH_SIZE {
                    black_box(map.remove(&1));
                    total_operations += 1;
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }

            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_hash_map,crossbeam,remove,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}

criterion_group!(benches, 
    benchmark_lockfree_list_memory,
    benchmark_lockfree_hash_map_memory
    );
criterion_main!(benches);
