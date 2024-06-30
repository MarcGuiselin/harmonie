use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug)]
struct StableId<'a> {
    crate_name: &'a str,
    version: &'a str,
    name: &'a str,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SystemId(u32);
impl SystemId {
    pub fn new(size: usize) -> SystemId {
        SystemId(size as _)
    }

    pub fn value(&self) -> usize {
        self.0 as _
    }
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum Descriptor {
    System(SystemDescriptor),
    Set {
        systems: Vec<SystemId>,
        conditions: Vec<SystemId>,
    },
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SystemDescriptor {
    id: SystemId,
    params: Vec<SystemParams>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum SystemParams {
    Command,
    // TODO: Query, Res, ResMut, etc
}
