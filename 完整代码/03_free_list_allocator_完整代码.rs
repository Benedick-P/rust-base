//! # 空闲链表分配器
//!
//! 在 bump 分配器的基础上，实现一个支持内存回收的空闲链表（Free-List）分配器。
//!
//! ## 工作原理
//!
//! 空闲链表分配器用一条链表记录所有已释放的内存块。
//! 分配时，它先在链表中寻找合适的块（首次适配策略）；
//! 若找不到，再从未使用区域分配。
//! 释放时，把该块插入链表头部。
//!
//! ```text
//! free_list -> [block A: 64B] -> [block B: 128B] -> [block C: 32B] -> null
//! ```
//!
//! 每个空闲块的开头存放一个 `FreeBlock` 结构（包含块大小和 next 指针）。
//!
//! ## 任务
//!
//! 实现 `FreeListAllocator` 的 `alloc` 与 `dealloc` 方法：
//!
//! ### alloc
//! 1. 遍历 free_list，找到第一个 `size >= layout.size()` 且对齐满足要求的块（首次适配）
//! 2. 若找到，把它从链表中摘除并返回
//! 3. 若没找到，就从 `bump` 区域分配（与 bump 分配器相同）
//!
//! ### dealloc
//! 1. 在释放的块上写入 `FreeBlock` 头部信息
//! 2. 把它插入 free_list 头部
//!
//! ## 关键概念
//!
//! - 侵入式链表
//! - `*mut T` 的读写：`ptr.write(val)` / `ptr.read()`
//! - 内存对齐检查

#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::Ordering;

/// 空闲块头部，存放在每个空闲内存块的开头
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

pub struct FreeListAllocator {
    heap_start: usize,
    heap_end: usize,
    /// Bump 指针：未分配区域从这里开始
    bump_next: core::sync::atomic::AtomicUsize,
    /// 空闲链表头（测试中由 Mutex 保护，其他情况用 UnsafeCell）
    #[cfg(test)]
    free_list: std::sync::Mutex<*mut FreeBlock>,
    #[cfg(not(test))]
    free_list: core::cell::UnsafeCell<*mut FreeBlock>,
}

#[cfg(test)]
unsafe impl Send for FreeListAllocator {}
#[cfg(test)]
unsafe impl Sync for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Send for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Sync for FreeListAllocator {}

impl FreeListAllocator {
    /// # 安全性
    /// `heap_start..heap_end` 必须是合法、可读可写的内存区域。
    pub unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            bump_next: core::sync::atomic::AtomicUsize::new(heap_start),
            #[cfg(test)]
            free_list: std::sync::Mutex::new(null_mut()),
            #[cfg(not(test))]
            free_list: core::cell::UnsafeCell::new(null_mut()),
        }
    }

    #[cfg(test)]
    fn free_list_head(&self) -> *mut FreeBlock {
        *self.free_list.lock().unwrap()
    }

    #[cfg(test)]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        *self.free_list.lock().unwrap() = head;
    }

    #[cfg(not(test))]
    fn free_list_head(&self) -> *mut FreeBlock {
        unsafe { *self.free_list.get() }
    }

    #[cfg(not(test))]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        unsafe { *self.free_list.get() = head }
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 确保块至少足以容纳一个 FreeBlock 头部（供将来 dealloc 使用）
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());
        let align = layout.align().max(core::mem::align_of::<FreeBlock>());

        // 第 1 步：遍历 free_list，首次适配
        let mut prev: *mut FreeBlock = null_mut();
        let mut curr = self.free_list_head();

        while !curr.is_null() {
            let block_size = (*curr).size;
            let addr = curr as usize;

            if block_size >= size && addr % align == 0 {
                // 从链表中摘除 curr
                let next = (*curr).next;
                if prev.is_null() {
                    self.set_free_list_head(next);
                } else {
                    (*prev).next = next;
                }
                return curr as *mut u8;
            }

            prev = curr;
            curr = (*curr).next;
        }

        // 第 2 步：free_list 中没有合适的块，从 bump 区域分配
        let mut next = self.bump_next.load(Ordering::SeqCst);
        loop {
            let aligned = match next.checked_add(align - 1) {
                Some(v) => v & !(align - 1),
                None => return null_mut(),
            };

            let end = match aligned.checked_add(size) {
                Some(v) => v,
                None => return null_mut(),
            };

            if end > self.heap_end {
                return null_mut();
            }

            match self
                .bump_next
                .compare_exchange(next, end, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return aligned as *mut u8,
                Err(actual) => next = actual,
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());

        // 1. 把 ptr 转换为 *mut FreeBlock
        let block = ptr as *mut FreeBlock;

        // 2. 写入 FreeBlock { size, next: 当前链表头 }
        let head = self.free_list_head();
        block.write(FreeBlock { size, next: head });

        // 3. 把 free_list 头更新为 ptr
        self.set_free_list_head(block);
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    const HEAP_SIZE: usize = 4096;

    fn make_allocator() -> (FreeListAllocator, Vec<u8>) {
        let mut heap = vec![0u8; HEAP_SIZE];
        let start = heap.as_mut_ptr() as usize;
        let alloc = unsafe { FreeListAllocator::new(start, start + HEAP_SIZE) };
        (alloc, heap)
    }

    #[test]
    fn test_alloc_basic() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(32, 8).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_alloc_alignment() {
        let (alloc, _heap) = make_allocator();
        for align in [1, 2, 4, 8, 16] {
            let layout = Layout::from_size_align(8, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null());
            assert_eq!(ptr as usize % align, 0, "align={align}");
        }
    }

    #[test]
    fn test_dealloc_and_reuse() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();

        let p1 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null());

        // 释放之后，下一次分配应复用同一个块
        unsafe { alloc.dealloc(p1, layout) };
        let p2 = unsafe { alloc.alloc(layout) };
        assert!(!p2.is_null());
        assert_eq!(p1, p2, "should reuse the freed block");
    }

    #[test]
    fn test_multiple_alloc_dealloc() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(128, 8).unwrap();

        let p1 = unsafe { alloc.alloc(layout) };
        let p2 = unsafe { alloc.alloc(layout) };
        let p3 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null() && !p2.is_null() && !p3.is_null());

        unsafe { alloc.dealloc(p2, layout) };
        unsafe { alloc.dealloc(p1, layout) };

        let q1 = unsafe { alloc.alloc(layout) };
        let q2 = unsafe { alloc.alloc(layout) };
        assert!(!q1.is_null() && !q2.is_null());
    }

    #[test]
    fn test_oom() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE + 1, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when exceeding heap");
    }
}
