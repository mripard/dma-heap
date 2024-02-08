use nix::ioctl_readwrite;

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
    dma_heap_alloc,
    DMA_HEAP_IOC_MAGIC,
    DMA_HEAP_IOC_ALLOC,
    dma_heap_allocation_data
);
