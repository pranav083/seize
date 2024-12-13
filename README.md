# `Benchmarking Concurrent Data Structures and Memory Reclamation Schemes`

This project benchmarks the performance of concurrent data structures using various memory reclamation schemes. It evaluates key metrics such as latency, throughput, scalability, and memory usage to analyze the trade-offs between different approaches to memory reclamation.
Overview

## Overview

The primary goal of this project is to benchmark the following data structures under various memory reclamation schemes:
Lock-Free Queue
Atomic Queue
Hashmap
Linked List

Each data structure was tested for common operations (e.g., enqueue, dequeue, insertion, deletion, lookup) under different memory reclamation schemes, specifically:
Reference Counting (via Rust's Arc)
Hazard Pointers (via the haphazard crate)
Epoch-Based Reclamation (via crossbeam_epoch)
Seize (via the seize crate)

The benchmarks measured:
Latency: The time taken to complete individual operations.
Throughput: The total number of operations completed in a fixed period.
Scalability: Performance trends as thread counts increase.
Free Memory Usage: Changes in memory usage during operations to evaluate reclamation efficiency.

# Implementation

The structure of this repository is based on the Seize GitHub repository with additional files added on. The additional files are queue_memory_bench.rs, memory_link_hash_test.rs, thread_link_hash_test.rs, that are located in the benches folder, and the four data structures created that are located in src/structures. A Python file called memory_graph is also within the project. 

To compile the project, you can run the command:

    cargo build
To generate the benchmarks for all bench files located in the benches folder, run the command:

    cargo bench
This will also build the code if it has not been build yet and run the benches.
To specify which bench you want to run, you can run the command:

    cargo bench --bench <bench_name>
where bench_name is the name of your benchmark file. Running the benchmark(s) will generate html files that are located within target/criterion. Under it, there will be every benchmark group that was run and their bench functions. There are a number of html files generated that included plots and graphs for individual bench functions as well as overall lin plots and violin graphs for benchmark groups.

To graph the change in free memory, one can run the memory benchmarks. These benchmarks will produce csv files that include hte change in free memory. To plot the csv files, one can run the Python file. OThe fiel_path may have to be changed depending on which csv file you want to use to generate a graph of. The operations may also need to be changed within the Python file depending on if you want to graph the queues or the hashmap or linked list.
