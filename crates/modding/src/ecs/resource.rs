use harmony_modloader_api as api;

pub trait Resource
where
    Self: api::HasStableId + bitcode::Encode + Default,
{
}
