use bevy_reflect::{serde::TypedReflectSerializer, PartialReflect, TypeRegistry};

mod ffi;
pub(crate) use ffi::*;

static RUNTIME: Option<Runtime> = None;

struct Runtime {
    registry: TypeRegistry,
}

pub(crate) fn serialize<'a>(value: &'a dyn PartialReflect) -> Vec<u8> {
    let runtime = RUNTIME.as_ref().unwrap();
    let serializer = TypedReflectSerializer::new(value, &runtime.registry);
    bitcode::serialize(&serializer).unwrap()
}
