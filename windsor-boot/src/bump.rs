pub struct BumpAllocator {
    ptr: *mut u8,
    len: usize,
    used: usize,
}

impl BumpAllocator {
    pub unsafe fn new(ptr: *mut u8, len: usize) -> Self {
        Self {
            ptr, len, used: 0
        }
    }
}

pub struct BumpAllocation<T> {
    mem: *mut T,
    len: usize,
}

impl<T> alloc::SliceWrapper<T> for BumpAllocation<T> {
    fn slice(& self) -> & [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.mem, self.len)
        }
    }
}

impl<T> alloc::SliceWrapperMut<T> for BumpAllocation<T> {
    fn slice_mut(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.mem, self.len)
        }
    }
}

impl<T: Default> Default for BumpAllocation<T> {
    fn default() -> Self {
        Self {
            mem: core::ptr::null_mut(),
            len: 0,
        }
    }
}

impl<T: Default> alloc::Allocator<T> for BumpAllocator {
    type AllocatedMemory = BumpAllocation<T>;
    fn free_cell(&mut self, _data: Self::AllocatedMemory) {
        // Do nothing
    }

    fn alloc_cell(&mut self, len: usize) -> Self::AllocatedMemory {
        let mem = self.ptr as *mut T;
        let size = len * core::mem::size_of::<T>();
        self.used += size;

        if self.used >= self.len {
            panic!("Unable to allocate memory");
        }

        unsafe {
            self.ptr = self.ptr.add(size);
        }

        Self::AllocatedMemory {
            mem,
            len,
        }
    }
}

