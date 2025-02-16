use bevy_reflect::{PartialReflect, Typed};

use crate::runtime::serialize;

pub trait Resource
where
    Self: Sized + Typed + PartialReflect,
{
    fn default_value() -> Self;

    fn default_value_as_buffer() -> Vec<u8> {
        let value = Self::default_value();
        serialize(&value)
    }
}

impl<R> Resource for R
where
    R: Sized + Typed + PartialReflect + Default,
{
    fn default_value() -> Self {
        Self::default()
    }
}
