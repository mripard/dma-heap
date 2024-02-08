use std::os::fd::{AsRawFd, BorrowedFd, RawFd};

use nix::{fcntl::OFlag, ioctl_readwrite};

use crate::{HeapError, Result};

const DMA_HEAP_IOC_MAGIC: u8 = b'H';
const DMA_HEAP_IOC_ALLOC: u8 = 0;

#[derive(Default)]
#[repr(C)]
pub(crate) struct dma_heap_allocation_data {
    pub(crate) len: u64,
    pub(crate) fd: u32,
    pub(crate) fd_flags: u32,
    pub(crate) heap_flags: u64,
}

ioctl_readwrite!(
    dma_heap_alloc_ioctl,
    DMA_HEAP_IOC_MAGIC,
    DMA_HEAP_IOC_ALLOC,
    dma_heap_allocation_data
);

pub(crate) fn dma_heap_alloc(fd: BorrowedFd<'_>, len: usize) -> Result<RawFd> {
    let mut fd_flags = OFlag::empty();

    fd_flags.insert(OFlag::O_CLOEXEC);
    fd_flags.insert(OFlag::O_RDWR);

    let mut data = dma_heap_allocation_data {
        len: len as u64,

        // Nix Oflags representation is an i32 for some reason, but the kernel actually expects an
        // u32 because the sign doesn't matter for a bitmask. Since the conversion between i32 and
        // u32 is a noop, we can cast it directly and expect it to work.
        #[allow(clippy::cast_sign_loss)]
        fd_flags: fd_flags.bits() as u32,
        ..dma_heap_allocation_data::default()
    };

    // SAFETY: This function is unsafe because the file descriptor might be invalid. However, the
    // BorrowedFd Rust type guarantees its validity so we are safe there.
    let res = unsafe { dma_heap_alloc_ioctl(fd.as_raw_fd(), &mut data) };

    let _ret: i32 = res.map_err(|err| {
        let err: std::io::Error = err.into();

        #[cfg_attr(feature = "nightly", allow(non_exhaustive_omitted_patterns))]
        #[allow(clippy::wildcard_enum_match_arm)]
        match err.kind() {
            std::io::ErrorKind::InvalidInput => HeapError::InvalidAllocation(len),
            std::io::ErrorKind::OutOfMemory => HeapError::NoMemoryLeft,
            _ => HeapError::from(err),
        }
    })?;

    Ok(data.fd as RawFd)
}
