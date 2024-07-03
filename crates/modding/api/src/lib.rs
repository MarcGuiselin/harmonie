use bitcode::{Decode, Encode};
use std::{
    any::TypeId,
    hash::{DefaultHasher, Hash, Hasher},
};

/// Identify structs
#[derive(Encode, Decode, PartialEq, Debug, Hash)]
struct StableId<'a> {
    crate_name: &'a str,
    version: &'a str,
    name: &'a str,
}

/// Identify systems
#[derive(Encode, Decode, PartialEq, Debug, Hash)]
pub struct SystemId(u64);

impl SystemId {
    pub fn from_type<T: ?Sized + 'static>() -> Self {
        let type_id = TypeId::of::<T>();

        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        let result = hasher.finish();

        Self(result)
    }
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum ExecutionDescriptor {
    System(SystemDescriptor),
    Set {
        systems: Vec<SystemId>,
        conditions: Vec<SystemId>,
    },
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SystemDescriptor {
    pub id: SystemId,
    pub params: Vec<ParamDescriptor>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SetDescriptor {
    pub systems: Vec<SystemId>,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum ParamDescriptor {
    Command,
    // TODO: Query, Res, ResMut, etc
}
