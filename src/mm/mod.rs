mod page;

use alloc::alloc::{GlobalAlloc, Layout};
use core::iter::ExactSizeIterator;
use page::FrameAllocator;
use spin::Mutex;
use uefi::table::boot::MemoryDescriptor;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{mapper::OffsetPageTable, PageTable};

pub fn init(
    map: impl ExactSizeIterator<Item = &'static MemoryDescriptor> + Clone,
) -> impl ExactSizeIterator<Item = &'static MemoryDescriptor> + Clone {
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::from_uefi(map.clone()));
    let (frame, _flags) = Cr3::read();
    let phys = frame.start_address().as_u64();
    let page_table = unsafe { &mut *(phys as *mut PageTable) };
    let page_table = unsafe { OffsetPageTable::new(page_table, x86_64::VirtAddr::zero()) };
    *PAGE_TABLE_MAPPER.lock() = Some(page_table);
    debug!("Allocator is initialized!");
    map
}

#[global_allocator]
static ALLOCATOR: PageAllocator = PageAllocator;

static PAGE_TABLE_MAPPER: Mutex<Option<OffsetPageTable>> = Mutex::new(None);

static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator>> = Mutex::new(None);

pub struct PageAllocator;

pub fn size() -> usize {
    let mut allocator = FRAME_ALLOCATOR.lock();
    allocator
        .as_mut()
        .expect("Allocator not initialized")
        .size()
}

unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut count = (layout.size() / 4096) as u32;
        if layout.size() % 4096 > 0 {
            count += 1;
        }
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator
            .as_mut()
            .expect("Allocator not initialized")
            .alloc(count) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        assert!((ptr as usize).trailing_zeros() >= 12);
        let mut count = (layout.size() / 4096) as u32;
        if layout.size() % 4096 > 0 {
            count += 1;
        }
        let mut allocator = FRAME_ALLOCATOR.lock();
        allocator
            .as_mut()
            .expect("Allocator not initialized")
            .dealloc(ptr as usize, count);
    }
}
