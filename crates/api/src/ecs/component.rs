use bevy_reflect::Typed;

/// Similar to bevy's Component
pub trait Component
where
    Self: Typed + bitcode::Encode + Decode,
{
}

impl<C> Component for C where C: Typed + bitcode::Encode + Decode {}

pub trait Decode
where
    Self: for<'a> bitcode::Decode<'a>,
{
}
impl<T> Decode for T where T: for<'a> bitcode::Decode<'a> {}
