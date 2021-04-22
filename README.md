# DMA-Buf Heap Helper Library

The DMA-Buf Heap interface in Linux is aimed at providing a way for the user-space to allocate
memory buffers that can be efficiently shared between multiple devices through the DMA-Buf
mechanism. It aims at superseeding the ION Interface previously found in Android.

This library provides a safe abstraction over this interface for Rust.

# Hello World

```no_run
let heap = DmaBufHeap::new(DmaBufHeapType::CMA)
    .unwrap();

let buffer = heap.allocate(1024).unwrap();
```
