use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct EntityCommandInsert<'a> {
    entity_id: u32,
    // Use box for single copy instead of double (skip copy in modloader)
    component: Box<[EncodedComponent<'a>]>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct EncodedComponent<'a> {
    id: &'a str,
    value: Box<[u8]>,
}
