use bitcode::{Decode, Encode};
use std::{
    any::TypeId,
    hash::{DefaultHasher, Hash, Hasher},
};

mod utils;
pub use utils::*;

/// Identify structs
#[derive(Encode, Decode, PartialEq, Debug, Hash)]
pub struct StableId<'a> {
    pub crate_name: &'a str,
    pub version: &'a str,
    pub name: &'a str,
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
pub enum SetIndices {
    System(usize),
    Sets(Vec<usize>),
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct SetDescriptor {
    pub indices: SetIndices,
    // TODO: run conditions, order, etc
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum ParamDescriptor {
    Command,
    // TODO: Query, Res, ResMut, etc
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ScheduleDescriptor<'a> {
    pub id: StableId<'a>,
    pub systems: Vec<SystemDescriptor>,
    pub sets: Vec<SetDescriptor>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct FeatureDescriptor<'a> {
    pub name: &'a str,
    pub resources: Vec<(StableId<'a>, Vec<u8>)>,
    pub descriptors: Vec<ScheduleDescriptor<'a>>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ModManifest<'a> {
    pub features: Vec<FeatureDescriptor<'a>>,
}
