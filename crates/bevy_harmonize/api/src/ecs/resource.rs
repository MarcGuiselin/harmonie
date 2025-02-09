use bevy_reflect::Typed;
use bitcode::Encode;

pub trait Resource
where
    Self: Typed,
{
    fn default_value_as_buffer() -> Vec<u8>;
}

impl<R> Resource for R
where
    R: Typed + Encode + Default,
{
    fn default_value_as_buffer() -> Vec<u8> {
        bitcode::encode(&Self::default())
    }
}
