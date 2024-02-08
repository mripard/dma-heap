// Copyright 2020-2021, Cerno
// Licensed under the MIT License
// See the LICENSE file or <http://opensource.org/licenses/MIT>

#![doc = include_str!("../README.md")]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::{
    fs::File,
    os::{
        fd::AsFd,
        unix::io::{FromRawFd, OwnedFd},
    },
    path::PathBuf,
};

mod ioctl;
use ioctl::dma_heap_alloc;

use log::debug;
use strum_macros::Display;

/// Error Type for dma-heap
#[derive(thiserror::Error, Debug)]
pub enum HeapError {
    /// The requested DMA Heap doesn't exist
    #[error("The Requested DMA Heap Type ({0}) doesn't exist: {1}")]
    Missing(HeapKind, PathBuf),

    /// An Error occured while accessing the DMA Heap
    #[error("An Error occurred while accessing the DMA Heap")]
    Access(std::io::Error),

    /// The allocation is invalid
    #[error("The requested allocation is invalid: {0} bytes")]
    InvalidAllocation(usize),

    /// There is no memory left to allocate from the DMA Heap
    #[error("No Memory Left in the Heap")]
    NoMemoryLeft,
}

impl From<std::io::Error> for HeapError {
    fn from(err: std::io::Error) -> Self {
        Self::Access(err)
    }
}

/// Generic Result type with [Error] as its error variant
pub type Result<T> = core::result::Result<T, HeapError>;

/// Various Types of DMA-Buf Heap
#[derive(Clone, Debug, Display)]
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
    pub fn new(name: HeapKind) -> Result<Self> {
        let path = match name {
            HeapKind::Cma => PathBuf::from("/dev/dma_heap/linux,cma"),
            HeapKind::System => PathBuf::from("/dev/dma_heap/system"),
            HeapKind::Custom(ref p) => p.clone(),
        };

        debug!("Using the {} DMA-Buf Heap, at {:#?}", name, path);

        let file = File::open(&path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => HeapError::Missing(name.clone(), path),
            _ => HeapError::from(err),
        })?;

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
    pub fn allocate(&self, len: usize) -> Result<OwnedFd> {
        debug!("Allocating Buffer of size {} on {} Heap", len, self.name);

        let raw_fd = dma_heap_alloc(self.file.as_fd(), len)?;

        debug!("Allocation succeeded, Buffer File Descriptor {}", raw_fd);

        // SAFETY: This function is unsafe because the file descriptor might not be valid, might
        // have been closed, or we might not be the sole owners of it. However, they are all
        // mitigated by the fact that the kernel has just given us that file descriptor so it's
        // valid, we are the exclusive owner of that fd, and we haven't closed it either.
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

        Ok(fd)
    }
}
