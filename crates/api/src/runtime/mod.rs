use bevy_reflect::{
    serde::{TypedReflectDeserializer, TypedReflectSerializer},
    FromReflect, GetTypeRegistration, PartialReflect, TypePath, TypeRegistry,
};
use serde::de::DeserializeSeed;

mod ffi;
pub(crate) use ffi::*;

struct Runtime {
    registry: TypeRegistry,
}

static mut RUNTIME: Option<Runtime> = None;

#[allow(unused)]
impl Runtime {
    fn new() -> Self {
        Self {
            registry: TypeRegistry::new(),
        }
    }

    /// SAFETY: Caller must ensure no references to the runtime is already borrowed
    unsafe fn init() -> &'static mut Self {
        RUNTIME = Some(Self::new());
        #[allow(static_mut_refs)]
        RUNTIME.as_mut().unwrap_unchecked()
    }

    /// SAFETY: Caller must ensure no references to the runtime is already borrowed
    unsafe fn get() -> &'static Self {
        #[allow(static_mut_refs)]
        if let Some(runtime) = RUNTIME.as_ref() {
            runtime
        } else {
            Self::init()
        }
    }

    /// SAFETY: Caller must ensure no references to the runtime is already borrowed
    unsafe fn get_mut() -> &'static mut Self {
        #[allow(static_mut_refs)]
        if let Some(runtime) = RUNTIME.as_mut() {
            runtime
        } else {
            Self::init()
        }
    }
}

pub(crate) fn serialize(value: &dyn PartialReflect) -> Vec<u8> {
    #[cfg(test)]
    // Tests are multithreaded, so we can't guarantee only one reference to the static
    let runtime = Runtime::new();
    #[cfg(not(test))]
    // SAFETY: This should be the only borrowed reference to the static
    let runtime = unsafe { Runtime::get() };

    let registry = &runtime.registry;
    let serializer = TypedReflectSerializer::new(value, registry);
    bitcode::serialize(&serializer).unwrap()
}

pub(crate) fn deserialize<T>(mut bytes: &[u8]) -> T
where
    T: FromReflect + TypePath + GetTypeRegistration,
{
    #[cfg(test)]
    // Tests are multithreaded, so we can't guarantee only one reference to the static
    let mut runtime = Runtime::new();
    #[cfg(not(test))]
    // SAFETY: This should be the only borrowed reference to the static
    let runtime = unsafe { Runtime::get_mut() };

    let registry = &mut runtime.registry;
    registry.register::<T>();

    let mut decoder = bitcode::SerdeDecoder::Unspecified { length: 1 };
    let bitcode_deserializer = bitcode::DecoderWrapper {
        decoder: &mut decoder,
        input: &mut bytes,
    };

    let reflect_deserializer = TypedReflectDeserializer::of::<T>(registry);
    let boxed = reflect_deserializer
        .deserialize(bitcode_deserializer)
        .unwrap();

    assert!(bytes.is_empty(), "Expected EOF");

    T::from_reflect(boxed.as_partial_reflect()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_reflect::Reflect;

    #[test]
    fn serde() {
        #[derive(Reflect, PartialEq, Debug)]
        struct Struct {
            string: String,
            vec: Vec<Enum>,
        }

        #[derive(Reflect, PartialEq, Debug)]
        enum Enum {
            A(u32),
            B,
        }

        let original = Struct {
            string: "hello world".into(),
            vec: vec![Enum::A(123), Enum::B],
        };

        let serialized = serialize(&original);
        let deserialized = deserialize(&serialized);

        assert_eq!(original, deserialized);
    }
}
