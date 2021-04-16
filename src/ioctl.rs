use nix::ioctl_readwrite;

const DMA_HEAP_IOC_MAGIC: u8 = b'H';
const DMA_HEAP_IOC_ALLOC: u8 = 0;

#[derive(Default)]
#[repr(C)]
pub struct dma_heap_allocation_data {
    pub len: u64,
    pub fd: u32,
    pub fd_flags: u32,
    pub heap_flags: u64,
}

ioctl_readwrite!(
    dma_heap_alloc,
    DMA_HEAP_IOC_MAGIC,
    DMA_HEAP_IOC_ALLOC,
    dma_heap_allocation_data
);
