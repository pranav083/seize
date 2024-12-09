use criterion::{criterion_group, criterion_main, Criterion};
use sysinfo::{System, SystemExt};
use seize::LockFreeQueue;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn memory_usage_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    let queue = Arc::new(LockFreeQueue::new());
    let memory_usage = Arc::new(Mutex::new(Vec::new()));
    let mut run_id = 0;

    group.bench_function("memory_tracking", |b| {
        let queue = queue.clone();
        let memory_usage = memory_usage.clone();

        b.iter_custom(|iters| {
            run_id += 1;
            let start = Instant::now();

            let tracking_handle = thread::spawn({
                let memory_usage = memory_usage.clone();
                let mut sys = System::new_all();
                move || {
                    let interval = Duration::from_millis(100);
                    while start.elapsed() < Duration::from_secs(5) {
                        sys.refresh_memory();
                        let mut usage = memory_usage.lock().unwrap();
                        usage.push((
                            run_id,
                            start.elapsed().as_secs_f64(),
                            sys.available_memory(),
                        ));
                        thread::sleep(interval);
                    }
                }
            });

            for _ in 0..(iters / 10) {
                queue.enqueue(1);
                queue.dequeue();
            }

            tracking_handle.join().unwrap();
            start.elapsed()
        });

        let memory_usage = memory_usage.lock().unwrap();
        let data: String = memory_usage
            .iter()
            .map(|(run, time, memory)| format!("{},{:.2},{}\n", run, time, memory))
            .collect();
        std::fs::write("memory_usage.csv", data).unwrap();
    });

    group.finish();
}

criterion_group!(benches, memory_usage_benchmark);
criterion_main!(benches);
