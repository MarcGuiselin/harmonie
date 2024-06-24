pub struct Slice {
    ptr: *const u8,
    len: usize,
}

impl Into<Slice> for u64 {
    fn into(self) -> Slice {
        let ptr = self as *const u8;
        let len = (self >> 32) as usize;
        Slice::new(ptr, len)
    }
}

impl Into<u64> for Slice {
    fn into(self) -> u64 {
        (self.len as u64) << 32 | self.ptr as u64
    }
}

impl Slice {
    pub fn new(ptr: *const u8, len: usize) -> Self {
        Self { ptr, len }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}
