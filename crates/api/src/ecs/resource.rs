use bevy_reflect::{FromReflect, GetTypeRegistration, Typed};

use crate::runtime::serialize;

pub trait Resource
where
    Self: Sized + Typed + FromReflect + GetTypeRegistration,
{
    fn default_value() -> Self;

    fn default_value_as_buffer() -> Vec<u8> {
        let value = Self::default_value();
        serialize(&value)
    }
}

impl<R> Resource for R
where
    R: Sized + Typed + FromReflect + GetTypeRegistration + Default,
{
    fn default_value() -> Self {
        Self::default()
    }
}
