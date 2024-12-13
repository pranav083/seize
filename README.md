Benchmarking Concurrent Data Structures and Memory Reclamation Schemes

This project benchmarks the performance of concurrent data structures using various memory reclamation schemes. It evaluates key metrics such as latency, throughput, scalability, and memory usage to analyze the trade-offs between different approaches to memory reclamation.
Overview

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

The benchmarks measure:

Latency: The time taken to complete individual operations.
Throughput: The total number of operations completed in a fixed period.
Scalability: Performance trends as thread counts increase.
Free Memory Usage: Changes in memory usage during operations to evaluate reclamation efficiency.

Features

Single-Threaded and Multi-Threaded Benchmarks: Analyze performance across varying levels of concurrency.
Custom Graph Generation: Automatically generates graphs to visualize benchmark results.
Comprehensive Metrics: Tracks memory usage, operation latency, and scalability for each combination of data structure and memory reclamation scheme.
Support for Rust's Ownership Model: Leverages Rust's safety features for concurrent programming to ensure thread safety and memory safety.