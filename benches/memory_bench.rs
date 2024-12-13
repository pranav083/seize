use criterion::{criterion_group, criterion_main, Criterion};
use seize::Collector;
use seize::LockFreeQueue;
use crossbeam_epoch as epoch;
use haphazard::HazardPointer;
use std::sync::Arc;
use std::hint::black_box;
use std::sync::atomic::AtomicPtr;
use seize::structures::atomic_queue::AtomicQueue;
use sysinfo::System;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::time::Duration;


const BATCH_SIZE: usize = 100;
const MAX_OPERATIONS: usize = 100_000;

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

    // No Scheme
    group.bench_function("Enqueue Memory (No scheme)", |b| {
        let queue = LockFreeQueue::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,ref_counting,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (No scheme)", |b| {
        let queue = LockFreeQueue::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,ref_counting,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Reference Counting
    group.bench_function("Enqueue Memory (Reference Counting)", |b| {
        let queue = Arc::new(LockFreeQueue::new());
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,ref_counting,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Reference Counting)", |b| {
        let queue = Arc::new(LockFreeQueue::new());
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,ref_counting,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Seize
    group.bench_function("Enqueue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = LockFreeQueue::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,seize,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = LockFreeQueue::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = collector.enter();

                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,seize,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Crossbeam Epoch
    group.bench_function("Enqueue Memory (Crossbeam Epoch)", |b| {
        let queue = LockFreeQueue::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();
                
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,crossbeam,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Crossbeam Epoch)", |b| {
        let queue = LockFreeQueue::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();

                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,crossbeam,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Hazard Pointer
    group.bench_function("Enqueue Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let queue = LockFreeQueue::new();
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
                        black_box(queue.enqueue(1));
                    }
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
                "lockfree_queue,hazard_pointer,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Hazard Pointer)", |b| {
        let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
        let queue = LockFreeQueue::new();
        let mut hazard_pointer = HazardPointer::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
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
                        black_box(queue.dequeue());
                    }
                    if total_operations >= MAX_OPERATIONS {
                        break;
                    }
                }
            }
            sys.refresh_memory();
            let memory_before = sys.available_memory();
            
            sys.refresh_memory();
            let memory_after = sys.available_memory();
            let memory_change = memory_after as i64 - memory_before as i64;

            writeln!(
                writer,
                "lockfree_queue,hazard_pointer,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
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

    // No Scheme
    group.bench_function("Enqueue Memory (No scheme)", |b| {
        let queue = AtomicQueue::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,ref_counting,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (No scheme)", |b| {
        let queue = AtomicQueue::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,ref_counting,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Reference Counting
    group.bench_function("Enqueue Memory (Reference Counting)", |b| {
        let queue = Arc::new(AtomicQueue::new());
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,ref_counting,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Reference Counting)", |b| {
        let queue = Arc::new(AtomicQueue::new());
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,ref_counting,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Seize
    group.bench_function("Enqueue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = AtomicQueue::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,seize,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Seize)", |b| {
        let collector = Collector::new();
        let queue = AtomicQueue::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = collector.enter();

                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,seize,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    // Crossbeam Epoch
    group.bench_function("Enqueue Memory (Crossbeam Epoch)", |b| {
        let queue = AtomicQueue::new();
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();
                
                for _ in 0..BATCH_SIZE {
                    black_box(queue.enqueue(1));
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
                "lockfree_queue,crossbeam,enqueue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

    group.bench_function("Dequeue Memory (Crossbeam Epoch)", |b| {
        let queue = AtomicQueue::new();
        for _ in 0..(BATCH_SIZE*100) {
            black_box(queue.enqueue(1));
        }
        b.iter_custom(|iters| {
            let mut total_operations = 0;
            let total_batches = (iters as usize) / BATCH_SIZE;

            sys.refresh_memory();
            let memory_before = sys.available_memory();

            for _ in 0..total_batches {
                let _guard = epoch::pin();
                
                for _ in 0..BATCH_SIZE {
                    black_box(queue.dequeue());
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
                "lockfree_queue,crossbeam,dequeue,{} KB,{} KB,{} KB",
                memory_before, memory_after, memory_change
            )
            .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
        });
    });

        // Hazard Pointer
        group.bench_function("Enqueue Memory (Hazard Pointer)", |b| {
            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
            let queue = AtomicQueue::new();
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
                            black_box(queue.enqueue(1));
                        }
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
                    "lockfree_queue,hazard_pointer,enqueue,{} KB,{} KB,{} KB",
                    memory_before, memory_after, memory_change
                )
                .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
            });
        });
    
        group.bench_function("Dequeue Memory (Hazard Pointer)", |b| {
            let atomic_ptr = AtomicPtr::new(Box::into_raw(Box::new(1)));
            let queue = AtomicQueue::new();
            let mut hazard_pointer = HazardPointer::new();
            for _ in 0..(BATCH_SIZE*100) {
                black_box(queue.enqueue(1));
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
                            black_box(queue.dequeue());
                        }
                        if total_operations >= MAX_OPERATIONS {
                            break;
                        }
                    }
                }
                sys.refresh_memory();
                let memory_before = sys.available_memory();
                
                sys.refresh_memory();
                let memory_after = sys.available_memory();
                let memory_change = memory_after as i64 - memory_before as i64;
    
                writeln!(
                    writer,
                    "lockfree_queue,hazard_pointer,dequeue,{} KB,{} KB,{} KB",
                    memory_before, memory_after, memory_change
                )
                .expect("Unable to write to file");
        Duration::from_secs_f64(0.1)
            });
        });
    
    group.finish();
    writer.flush().expect("Failed to flush memory usage data");
}

criterion_group!(benches, benchmark_lockfree_queue_memory, benchmark_atomic_queue_memory);
criterion_main!(benches);
