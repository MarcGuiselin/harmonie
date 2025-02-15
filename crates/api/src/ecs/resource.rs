use bevy_reflect::{serde::TypedReflectSerializer, GetTypeRegistration, TypeRegistry, Typed};

pub trait Resource
where
    Self: Typed,
{
    fn default_value_as_buffer() -> Vec<u8>;
}

impl<R> Resource for R
where
    R: Typed + GetTypeRegistration + Default,
{
    fn default_value_as_buffer() -> Vec<u8> {
        let mut registry = TypeRegistry::default();
        registry.register::<R>();

        let default = Self::default();
        let serializer = TypedReflectSerializer::new(&default, &registry);

        bitcode::serialize(&serializer).unwrap()
    }
}
