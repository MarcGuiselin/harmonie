use super::StableId;

pub trait Resource
where
    Self: StableId + bitcode::Encode + Default,
{
}
