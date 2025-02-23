// Compile with: `RUSTFLAGS="" cargo build --target wasm32-unknown-unknown -p wasmtest -Zbuild-std`

#![no_std]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

struct MyResource {
    a: u32,
    b: u32,
}

extern "C" {
    // While llvm does support multiple address spaces (via the address-space attribute), unfortunately support is
    // not coming to rust any time soon.
    //
    // So in order to support densly storing components in shared memory (similar to bevy), we need to manually modify
    // the compiled wasm (wat to make things easier). Specifically, we need to add address space indexes to the `i32.store`
    // and `i32.load` instructions that access/modify components.
    //
    // This presents a problem though: From looking at wat, how do we know what instructions correspond to what components?
    //
    // The mod must already ask the modloader for offsets in order to find components in memory. We can leverage this to
    // easily find the instructions that store/load components: Codegen passes unique ids to this function. These
    // ids are not component ids, since the address space for a compoent might change based on the system. Then we
    // convert the binary to wat, replacing these ids with the stable component index as determined by the manifest,
    // and correcting the address space of all instructions that use the output pointer from this function.
    //
    // See:
    // - https://clang.llvm.org/docs/AttributeReference.html#address-space
    // - https://bytecodealliance.zulipchat.com/#narrow/stream/223391-wasm/topic/Multi-memory.20.2F.20shared.20memory
    // - https://www.reddit.com/r/rust/comments/dlj2bh/can_i_use_the_address_space_indices_like_clangs/

    pub fn __resource_offset(id: i32) -> *mut u8;
}

struct ResMut<T> {
    ptr: *mut u8,
    phantom: PhantomData<T>,
}

impl<T> ResMut<T> {
    #[inline(always)]
    fn new(id: i32) -> Self {
        Self {
            ptr: unsafe { __resource_offset(id) },
            phantom: PhantomData,
        }
    }
}

impl<T> Deref for ResMut<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.cast() }
    }
}

impl<T> DerefMut for ResMut<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr.cast() }
    }
}

#[no_mangle]
pub unsafe extern "C" fn run() {
    let resource = ResMut::new(1234);
    system(resource)
}

fn system(mut resource: ResMut<MyResource>) {
    resource.a -= 11;
    resource.b += 12 + resource.a;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
