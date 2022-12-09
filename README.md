# DMA-Buf Heap Helper Library

The DMA-Buf Heap interface in Linux is aimed at providing a way for the user-space to allocate
memory buffers that can be efficiently shared between multiple devices through the DMA-Buf
mechanism. It aims at superseeding the ION Interface previously found in Android.

This library provides a safe abstraction over this interface for Rust.

# Hello World

```rust,no_run
use std::os::unix::io::OwnedFd;
use dma_heap::{Heap, HeapKind};

let heap = Heap::new(HeapKind::Cma)
    .unwrap();

// Buffer will automatically be freed when `buffer` goes out of scope.
let buffer: OwnedFd = heap.allocate(1024).unwrap();
```
