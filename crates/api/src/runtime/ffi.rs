use common::StableId;

pub(crate) fn ffi_spawn_empty() -> u32 {
    unsafe { spawn_empty() }
}

pub(crate) fn ffi_set_component(entity_id: u32, type_id: &StableId, value: &Vec<u8>) {
    unsafe {
        set_component(
            entity_id,
            type_id.name.as_ptr() as _,
            type_id.name.len() as _,
            type_id.crate_name.as_ptr() as _,
            type_id.crate_name.len() as _,
            value.as_ptr() as _,
            value.len() as _,
        );
    }
}

pub struct LocalTypeId(u32);

pub(crate) fn ffi_get_local_type_id(type_id: &StableId) -> LocalTypeId {
    LocalTypeId(unsafe {
        get_local_type_id(
            type_id.name.as_ptr() as _,
            type_id.name.len() as _,
            type_id.crate_name.as_ptr() as _,
            type_id.crate_name.len() as _,
        )
    })
}

pub(crate) fn ffi_set_resource(type_id: &LocalTypeId, buffer: &Vec<u8>) {
    unsafe {
        set_resource(type_id.0, buffer.as_ptr() as _, buffer.len() as _);
    }
}

pub(crate) fn ffi_get_resource(type_id: &LocalTypeId) -> Vec<u8> {
    let size = unsafe { buffer_resource(type_id.0) };

    let mut buffer = Vec::with_capacity(size as _);
    unsafe { write_buffer_to(buffer.as_mut_ptr() as _) };
    buffer
}

#[link(wasm_import_module = "bevy_harmonize")]
extern "C" {
    fn spawn_empty() -> u32;
    fn set_component(
        entity_id: u32,
        type_short_name_ptr: u32,
        type_short_name_len: u32,
        type_crate_name_ptr: u32,
        type_crate_name_len: u32,
        buffer_ptr: u32,
        buffer_len: u32,
    );
    fn get_local_type_id(
        type_short_name_ptr: u32,
        type_short_name_len: u32,
        type_crate_name_ptr: u32,
        type_crate_name_len: u32,
    ) -> u32;
    fn set_resource(local_type_id: u32, buffer_ptr: u32, buffer_len: u32);
    fn buffer_resource(local_type_id: u32) -> u32;
    fn write_buffer_to(ptr: u32);
}
