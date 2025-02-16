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
}
