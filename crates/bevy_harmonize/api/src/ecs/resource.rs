pub trait Resource
where
    Self: common::HasStableId + bitcode::Encode + Default,
{
    fn default_value_as_buffer() -> Vec<u8> {
        bitcode::encode(&Self::default())
    }
}
