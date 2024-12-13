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

The structure of this repository is based on the Seize GitHub repository with additional files added on. the additional files are queue_memory_bench.rs, memory_link_hash_test.rs, thread_link_hash_test.rs, and 