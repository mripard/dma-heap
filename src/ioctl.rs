use std::{
    io,
    os::fd::{BorrowedFd, FromRawFd, OwnedFd, RawFd},
};

use rustix::{
    fs::OFlags,
    io::Errno,
    ioctl::{ioctl, ReadWriteOpcode, Updater},
};

use crate::{HeapError, Result};

const DMA_HEAP_IOC_MAGIC: u8 = b'H';
const DMA_HEAP_IOC_ALLOC: u8 = 0;

#[derive(Default)]
#[repr(C)]
struct dma_heap_allocation_data {
    len: u64,
    fd: u32,
    fd_flags: u32,
    heap_flags: u64,
}

fn dma_heap_alloc_ioctl(
    fd: BorrowedFd<'_>,
    data: &mut dma_heap_allocation_data,
) -> core::result::Result<(), Errno> {
    type Opcode = ReadWriteOpcode<DMA_HEAP_IOC_MAGIC, DMA_HEAP_IOC_ALLOC, dma_heap_allocation_data>;

    // SAFETY: This function is unsafe because the opcode has to be valid, and the value type must
    // match. We have checked those, so we're good.
    let ioctl_type = unsafe { Updater::<Opcode, dma_heap_allocation_data>::new(data) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl,
    // and to implement it properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_type) }
}

pub(crate) fn dma_heap_alloc(fd: BorrowedFd<'_>, len: usize) -> Result<OwnedFd> {
    let mut fd_flags = OFlags::empty();

    fd_flags.insert(OFlags::CLOEXEC);
    fd_flags.insert(OFlags::RDWR);

    let mut data = dma_heap_allocation_data {
        len: len as u64,
        fd_flags: fd_flags.bits(),
        ..dma_heap_allocation_data::default()
    };

    dma_heap_alloc_ioctl(fd, &mut data).map_err(|err| match err {
        Errno::INVAL => HeapError::InvalidAllocation(len),
        Errno::NOMEM => HeapError::NoMemoryLeft,
        _ => io::Error::from_raw_os_error(err.raw_os_error()).into(),
    })?;

    // SAFETY: This function is unsafe because the file descriptor might not be valid, might
    // have been closed, or we might not be the sole owners of it. However, they are all
    // mitigated by the fact that the kernel has just given us that file descriptor so it's
    // valid, we are the exclusive owner of that fd, and we haven't closed it either.
    let fd = unsafe { OwnedFd::from_raw_fd(data.fd as RawFd) };

    Ok(fd)
}
