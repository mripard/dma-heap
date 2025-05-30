use std::{
    io,
    os::fd::{BorrowedFd, FromRawFd as _, OwnedFd, RawFd},
};

use rustix::{
    fs::OFlags,
    io::Errno,
    ioctl::{Updater, ioctl, opcode},
};

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

const DMA_HEAP_IOC_ALLOC_OPCODE: u32 =
    opcode::read_write::<dma_heap_allocation_data>(DMA_HEAP_IOC_MAGIC, DMA_HEAP_IOC_ALLOC);

fn dma_heap_alloc_ioctl(fd: BorrowedFd<'_>, data: &mut dma_heap_allocation_data) -> io::Result<()> {
    // SAFETY: This function is unsafe because the opcode has to be valid, and the value type must
    // match. We have checked those, so we're good.
    let ioctl_type =
        unsafe { Updater::<DMA_HEAP_IOC_ALLOC_OPCODE, dma_heap_allocation_data>::new(data) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl,
    // and to implement it properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_type) }.map_err(<Errno as Into<io::Error>>::into)
}

pub(crate) fn dma_heap_alloc(fd: BorrowedFd<'_>, len: usize) -> io::Result<OwnedFd> {
    let mut data = dma_heap_allocation_data {
        len: len as u64,
        fd_flags: OFlags::union(OFlags::CLOEXEC, OFlags::RDWR).bits(),
        ..dma_heap_allocation_data::default()
    };

    dma_heap_alloc_ioctl(fd, &mut data)?;

    // SAFETY: This function is unsafe because the file descriptor might not be valid, might
    // have been closed, or we might not be the sole owners of it. However, they are all
    // mitigated by the fact that the kernel has just given us that file descriptor so it's
    // valid, we are the exclusive owner of that fd, and we haven't closed it either.
    let fd = unsafe {
        #[allow(
            clippy::cast_possible_wrap,
            reason = "The sign doesn't matter, the fd is an opaque value anyway."
        )]
        OwnedFd::from_raw_fd(data.fd as RawFd)
    };

    Ok(fd)
}
