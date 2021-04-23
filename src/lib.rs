// Copyright 2020-2021, Cerno
// Licensed under the MIT License
// See the LICENSE file or <http://opensource.org/licenses/MIT>
//
//! # DMA-Buf Heap Helper Library
//!
//! The DMA-Buf Heap interface in Linux is aimed at providing a way for the user-space to allocate
//! memory buffers that can be efficiently shared between multiple devices through the DMA-Buf
//! mechanism. It aims at superseeding the ION Interface previously found in Android.
//!
//! This library provides a safe abstraction over this interface for Rust.
//!
//! # Hello World
//!
//! ```no_run
//! use std::fs::File;
//! use std::os::unix::io::RawFd;
//! use dma_heap::{DmaBufHeap, DmaBufHeapType};
//!
//! let heap = DmaBufHeap::new(DmaBufHeapType::Cma)
//!     .unwrap();
//!
//! // Buffer will automatically be freed when `buffer_file` goes out of scope.
//! let buffer_file: File = heap.allocate(1024).unwrap();
//! // Buffer lifetime must be manually managed.
//! let buffer_rawfd: RawFd = heap.allocate(1024).unwrap();
//! ```

#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unreadable_literal)]

use std::{
    fs::File,
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
};

mod ioctl;
use ioctl::dma_heap_alloc;
use ioctl::dma_heap_allocation_data;

use log::debug;
use strum_macros::Display;

/// Error Type for dma-heap
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An Error happened when allocating a buffer
    #[error("Couldn't allocate the buffer")]
    Allocation(#[from] nix::Error),

    /// An Error happened when opening the Heap
    #[error("Couldn't open the DMA-Buf Heap")]
    Open(#[from] std::io::Error),
}

/// Generic Result type with [Error] as its error variant
pub type Result<T> = std::result::Result<T, Error>;

/// Various Types of DMA-Buf Heap
#[derive(Clone, Copy, Debug, Display)]
pub enum DmaBufHeapType {
    /// A Heap backed by the Contiguous Memory Allocator in the Linux kernel, returning physically
    /// contiguous, cached, buffers
    Cma,

    /// A Heap backed by the vmalloc allocator in the Linux kernel, returning virtually contiguous,
    /// cached, buffers
    System,
}

/// Our DMA-Buf Heap
#[derive(Debug)]
pub struct DmaBufHeap {
    file: File,
    name: DmaBufHeapType,
}

impl DmaBufHeap {
    /// Opens A DMA-Buf Heap of the specified type
    ///
    /// # Errors
    ///
    /// Will return [Error] if the Heap Type is not found in the system, or if the open call fails.
    pub fn new(name: DmaBufHeapType) -> Result<Self> {
        let path = match name {
            DmaBufHeapType::Cma => "/dev/dma_heap/reserved",
            DmaBufHeapType::System => "/dev/dma_heap/system",
        };

        debug!("Using the {} DMA-Buf Heap, at {}", name, path);

        let file = File::open(path)?;

        debug!("Heap found!");

        Ok(Self { file, name })
    }

    /// Allocates a DMA-Buf from the Heap with the specified size
    ///
    /// # Errors
    ///
    /// Will return [Error] if the underlying ioctl fails.
    pub fn allocate<T: FromRawFd>(&self, len: usize) -> Result<T> {
        let mut fd_flags = nix::fcntl::OFlag::empty();

        fd_flags.insert(nix::fcntl::OFlag::O_CLOEXEC);
        fd_flags.insert(nix::fcntl::OFlag::O_RDWR);

        let mut data = dma_heap_allocation_data {
            len: len as u64,
            fd_flags: fd_flags.bits() as u32,
            ..dma_heap_allocation_data::default()
        };

        debug!("Allocating Buffer of size {} on {} Heap", len, self.name);

        let _ = unsafe { dma_heap_alloc(self.file.as_raw_fd(), &mut data) }?;

        debug!("Allocation succeeded, Buffer File Descriptor {}", data.fd);

        // Safe because we have confirmed that the ioctl has succeeded, thus
        // the FD is valid.
        Ok(unsafe { T::from_raw_fd(data.fd as RawFd) })
    }
}
