// Copyright 2020-2021, Cerno
// Licensed under the MIT License
// See the LICENSE file or <http://opensource.org/licenses/MIT>

#![cfg_attr(
    feature = "nightly",
    feature(non_exhaustive_omitted_patterns_lint, strict_provenance_lints)
)]
#![cfg_attr(
    feature = "nightly",
    warn(
        fuzzy_provenance_casts,
        lossy_provenance_casts,
        unnameable_types,
        non_exhaustive_omitted_patterns,
    )
)]
#![doc = include_str!("../README.md")]
#![allow(unsafe_code)]

use core::fmt;
use std::{
    fs::File,
    io,
    os::{fd::AsFd as _, unix::io::OwnedFd},
    path::PathBuf,
};

mod ioctl;
use ioctl::dma_heap_alloc;
use log::debug;

/// Various Types of DMA-Buf Heap
#[derive(Clone, Debug)]
pub enum HeapKind {
    /// A Heap backed by the Contiguous Memory Allocator in the Linux kernel, returning physically
    /// contiguous, cached, buffers
    Cma,

    /// A Heap backed by the vmalloc allocator in the Linux kernel, returning virtually contiguous,
    /// cached, buffers
    System,

    /// The Path to a custom Heap Type.
    Custom(PathBuf),
}

impl fmt::Display for HeapKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeapKind::Cma => f.write_str("CMA"),
            HeapKind::System => f.write_str("System"),
            HeapKind::Custom(p) => f.write_fmt(format_args!("Custom Heap ({})", p.display())),
        }
    }
}

/// Our DMA-Buf Heap
#[derive(Debug)]
pub struct Heap {
    file: File,
    name: HeapKind,
}

impl Heap {
    /// Opens A DMA-Buf Heap of the specified type
    ///
    /// # Errors
    ///
    /// Will return [Error] if the Heap Type is not found in the system, or if the open call fails.
    pub fn new(name: HeapKind) -> io::Result<Self> {
        let path = match &name {
            HeapKind::Cma => PathBuf::from("/dev/dma_heap/linux,cma"),
            HeapKind::System => PathBuf::from("/dev/dma_heap/system"),
            HeapKind::Custom(p) => p.clone(),
        };

        debug!("Using the {name} DMA-Buf Heap, at {}", path.display());

        let file = File::open(&path)?;

        debug!("Heap found!");

        Ok(Self { file, name })
    }

    /// Allocates a DMA-Buf from the Heap with the specified size
    ///
    /// # Panics
    ///
    /// If the errno returned by the underlying `ioctl()` cannot be decoded
    /// into an `std::io::Error`.
    ///
    /// # Errors
    ///
    /// Will return [Error] if the underlying ioctl fails.
    pub fn allocate(&self, len: usize) -> io::Result<OwnedFd> {
        debug!("Allocating Buffer of size {} on {} Heap", len, self.name);

        let fd = dma_heap_alloc(self.file.as_fd(), len)?;

        debug!("Allocation succeeded, Buffer File Descriptor {fd:#?}");

        Ok(fd)
    }
}
