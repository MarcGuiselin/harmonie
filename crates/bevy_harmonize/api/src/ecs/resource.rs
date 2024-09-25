pub trait Resource
where
    Self: common::HasStableId + bitcode::Encode + Default,
{
}
