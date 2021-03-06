use staticvec::StaticVec;
use uefi::table::boot::{MemoryDescriptor, MemoryType};

#[derive(Clone, Copy)]
pub struct FreeFrame {
    start: u32,
    len: u32,
}

pub struct FrameAllocator {
    map: StaticVec<FreeFrame, 64>,
}

impl FrameAllocator {
    pub fn from_uefi(mem_iter: impl Iterator<Item = &'static MemoryDescriptor>) -> Self {
        let mut allocator = FrameAllocator {
            map: StaticVec::new(),
        };
        for mem in mem_iter {
            assert!(mem.page_count > 0);
            if let MemoryType::CONVENTIONAL | MemoryType::BOOT_SERVICES_CODE = mem.ty {
                let item = FreeFrame {
                    start: (mem.phys_start >> 12) as u32,
                    len: mem.page_count as u32,
                };
                allocator.push(item);
            }
        }
        allocator.compat();
        allocator.debug();
        allocator
    }
    pub fn alloc(&mut self, count: u32) -> usize {
        for i in (0..self.map.len()).rev() {
            if self.map[i].len >= count {
                self.map[i].len -= count;
                let result = self.map[i].start + self.map[i].len;
                if self.map[i].len == 0 {
                    self.map.pop();
                }
                return (result as usize) << 12;
            }
        }
        panic!("Frame alloc failed")
    }
    pub fn dealloc(&mut self, addr: usize, len: u32) {
        self.map.push(FreeFrame {
            start: (addr >> 12) as u32,
            len,
        });
        self.compat();
    }
    fn push(&mut self, item: FreeFrame) {
        if !self.map.is_full() || self.compat() {
            self.map.push(item)
        } else {
            panic!("FrameAllocator overflow!")
        }
    }
    fn compat(&mut self) -> bool {
        self.map.sort_unstable_by(|a, b| a.start.cmp(&b.start));
        if self.map.len() > 1 {
            let mut slow = 0;
            for fast in 1..self.map.len() {
                if self.map[slow].start + self.map[slow].len == self.map[fast].start {
                    self.map[slow].len += self.map[fast].len;
                } else {
                    assert!(self.map[slow].start + self.map[slow].len < self.map[fast].start);
                    slow += 1;
                    if slow != fast {
                        self.map[slow] = self.map[fast];
                    }
                }
            }
            unsafe {
                self.map.set_len(slow + 1);
            }
        }
        !self.map.is_full()
    }
    fn debug(&self) {
        debug!("Usable pages:");
        for i in self.map.iter() {
            debug!("Page: 0x{:06X} -- 0x{:06X}", i.start, i.start + i.len)
        }
    }

    pub fn size(&self) -> usize {
        let frames: u32 = self.map.iter().map(|f| f.len).sum();
        (frames as usize) * 4096
    }
}
