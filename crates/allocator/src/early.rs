use crate::BaseAllocator;

use super::{AllocError, AllocResult, PageAllocator, ByteAllocator};
use core::alloc::Layout;
use core::mem::size_of;
use core::ptr::NonNull;

const PAGE_SIZE: usize = 0x1000;

pub struct EarlyAllocator {
    start: usize,
    len: usize,
    byte_ava: usize,
    byte_count: usize,
    page_ava: usize,
    page_count: usize,
}

impl EarlyAllocator {
    /// Creates a new empty `SlabByteAllocator`.
    pub const fn new() -> Self {
        Self { 
            start: 0,
            len: 0,  
            byte_ava: 0,
            byte_count: 0,
            page_ava: 0,
            page_count: 0,
        }
    }
    
}

impl ByteAllocator for EarlyAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let align = layout.align();
        if !align.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let size = (layout.size() + align - 1) & (!align + 1);
        let byte_ava = (self.byte_ava + align - 1) & (!align + 1);
        if byte_ava + size > self.page_ava {
            return Err(AllocError::NoMemory);
        }
        self.byte_ava = byte_ava + size;
        self.byte_count += 1;
        unsafe { Ok(NonNull::new_unchecked(byte_ava as *mut u8)) }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, _layout: Layout) {
        let pos = pos.as_ptr() as usize;
        if pos > self.start && pos < self.byte_ava {
            self.byte_count -= 1;
            if self.byte_count <= 0 {
                self.byte_ava = self.start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.len
    }

    fn used_bytes(&self) -> usize {
        self.byte_ava - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_ava - self.byte_ava
    }
}

impl BaseAllocator for EarlyAllocator {
    fn init(&mut self, start: usize, size: usize) {
        let end = (start + size) & (!size_of::<usize>() + 1);
        let start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        
        assert!(start <= end);

        self.start = start;
        self.len = end - start;
        self.byte_ava = start;
        self.byte_count = 0;
        self.page_ava = end;
        self.page_count = 0;
        
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::NoMemory) // unsupported
    }
}
impl PageAllocator for EarlyAllocator {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        if num_pages == 0 {
            return Err(AllocError::InvalidParam);
        }
        let size = (num_pages * PAGE_SIZE + align_pow2 - 1) & (!align_pow2 + 1);
        let page_ava = (self.page_ava - size) & (!align_pow2 + 1);
        if page_ava < self.byte_ava {
            return Err(AllocError::NoMemory);
        }
        self.page_ava = page_ava;
        self.page_count += 1;
        Ok(page_ava)
    }

    fn dealloc_pages(&mut self, pos: usize, _num_pages: usize) {
        if pos > self.page_ava && pos < self.start + self.len {
            self.page_count -= 1;
            if self.page_count <= 0 {
                self.page_ava = self.len + self.start;
            }
        }
        
    }

    fn total_pages(&self) -> usize {
        self.len / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.start + self.len - self.page_ava) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.page_ava - self.byte_ava) / PAGE_SIZE
    }
}