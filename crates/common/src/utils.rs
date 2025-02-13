use std::ops::Range;

/// Valid only for 32-bit wasm!
#[derive(Debug, Clone, Copy)]
pub struct WasmPointer<'a> {
    #[cfg(not(target_arch = "wasm32"))]
    ptr: u32,
    #[cfg(target_arch = "wasm32")]
    pub ptr: u32,
    #[cfg(not(target_arch = "wasm32"))]
    len: u32,
    #[cfg(target_arch = "wasm32")]
    pub len: u32,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl From<u64> for WasmPointer<'_> {
    fn from(value: u64) -> Self {
        let ptr = value as u32;
        let len = (value >> 32) as u32;
        Self::new(ptr, len)
    }
}

impl Into<u64> for WasmPointer<'_> {
    fn into(self) -> u64 {
        (self.len as u64) << 32 | self.ptr as u64
    }
}

impl Into<Range<u64>> for WasmPointer<'_> {
    fn into(self) -> Range<u64> {
        let start = self.ptr as u64;
        let end = start + self.len as u64;
        start..end
    }
}

impl<'a> WasmPointer<'a> {
    fn new(ptr: u32, len: u32) -> Self {
        Self {
            ptr,
            len,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn from_vec(vec: &'a Vec<u8>) -> Self {
        let ptr = vec.as_ptr();
        let len = vec.len();
        Self::new(ptr as _, len as _)
    }
}
