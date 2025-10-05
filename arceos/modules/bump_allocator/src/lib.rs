#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator, AllocResult, AllocError};
use core::alloc::Layout;
use core::ptr::NonNull;

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
    page_count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
            page_count: 0,
        }
    }

    /// 向上对齐
    fn align_up(&self, v: usize, align: usize) -> usize {
        (v + align - 1) & !(align - 1)
    }

    /// 向下对齐
    fn align_down(&self, v: usize, align: usize) -> usize {
        v &!(align - 1)
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize){
        self.start = start;
        self.end = (start + size) / PAGE_SIZE * PAGE_SIZE; // 对齐到页边界
        self.b_pos = start;
        self.p_pos = start + size;
    }
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        unimplemented!() // 不支持
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        let b_pos = self.align_up(self.b_pos, align); // 起始位置对齐
        if b_pos + size <= self.p_pos { // 检查空间是否足够
            self.b_pos = b_pos + size;
            self.count += 1;
            Ok(NonNull::new(b_pos as *mut u8).unwrap())
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.count -= 1;
        if self.count == 0 { // 没有分配时释放空间
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.p_pos - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let align = 1 << align_pow2;
        let p_pos = self.align_down(self.p_pos - num_pages * PAGE_SIZE, align); // 起始位置对齐
        if p_pos >= self.b_pos { // 检查空间是否足够
            self.p_pos = p_pos;
            self.page_count += num_pages;
            Ok(p_pos)
        } else {
            Err(AllocError::NoMemory)
        }
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.page_count -= num_pages;
        if self.page_count == 0 { // 没有分配时释放空间
            self.p_pos = self.end;
        }
    }

    fn total_pages(&self) -> usize {
        (self.end - self.b_pos) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }
}
