use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub struct WasmPointer {
    pub ptr: u32,
    pub len: u32,
}

impl Into<WasmPointer> for u64 {
    fn into(self) -> WasmPointer {
        let ptr = self as u32;
        let len = (self >> 32) as u32;
        WasmPointer::new(ptr, len)
    }
}

impl Into<u64> for WasmPointer {
    fn into(self) -> u64 {
        (self.len as u64) << 32 | self.ptr as u64
    }
}

impl Into<Range<u64>> for WasmPointer {
    fn into(self) -> Range<u64> {
        let start = self.ptr as u64;
        let end = start + self.len as u64;
        start..end
    }
}

impl WasmPointer {
    pub fn new(ptr: u32, len: u32) -> Self {
        Self { ptr, len }
    }

    /// SAFETY: user must ensure sure that value is not mutated or dropped
    // TODO: Make safe, by keeping a lifetime
    pub unsafe fn from_vec(vec: &Vec<u8>) -> Self {
        Self {
            ptr: vec.as_ptr() as u32,
            len: vec.len() as u32,
        }
    }
}
