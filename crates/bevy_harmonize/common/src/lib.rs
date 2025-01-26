#![feature(const_trait_impl)]
#![feature(const_type_name)]

use bitcode::{Decode, Encode};
use std::{
    any::TypeId,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
};

mod schedule;
pub use schedule::*;

mod identifiers;
pub use identifiers::*;

mod utils;
pub use utils::*;

pub type StableIdGetter = fn() -> StableId<'static>;

/// Identify structs
#[derive(Encode, Decode, PartialEq, Eq, Hash, Clone, Copy)]
pub struct StableId<'a> {
    pub crate_name: &'a str,
    pub version: &'a str,
    pub name: &'a str,
}

impl<'a> StableId<'a> {
    // Not ideal, but simply taking [this advice](https://stackoverflow.com/questions/72105604/implement-toowned-for-user-defined-types#answer-72106272:~:text=If%20you%20just%20want%20to%20be%20able%20to%20call%20a%20.to_owned()%20method%20on%20a%20DataRef%2C%20don%27t%20bother%20with%20the%20ToOwned%20trait%3B%20just%20write%20an%20inherent%20(non%2Dtrait)%20method.)
    pub fn to_owned(&self) -> OwnedStableId {
        OwnedStableId {
            crate_name: self.crate_name.to_owned(),
            version: self.version.to_owned(),
            name: self.name.to_owned(),
        }
    }
}

impl<'a> fmt::Debug for StableId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StableId(\"{}::{} {}\")",
            self.crate_name, self.name, self.version
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct OwnedStableId {
    pub crate_name: String,
    pub version: String,
    pub name: String,
}

impl fmt::Debug for OwnedStableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OwnedStableId(\"{}::{} {}\")",
            self.crate_name, self.name, self.version
        )
    }
}

/// A id shared between mods, used to identify objects defined in the manifest
#[const_trait]
pub trait HasStableId {
    const CRATE_NAME: &'static str;
    const VERSION: &'static str;
    const NAME: &'static str;

    #[inline]
    fn get_stable_id() -> StableId<'static> {
        StableId {
            crate_name: Self::CRATE_NAME,
            version: Self::VERSION,
            name: Self::NAME,
        }
    }
}

/// Identify systems
#[derive(Encode, Decode, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct SystemId(u64);

impl SystemId {
    pub fn of<T: ?Sized + 'static>() -> Self {
        Self::from_type(TypeId::of::<T>())
    }

    pub fn from_type(id: TypeId) -> Self {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let result = hasher.finish();

        Self(result)
    }

    pub fn get_raw(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SystemId(\"{:x}\")", self.0)
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum Param<'a> {
    Command,
    Res(StableId<'a>),
    // TODO: Query, Res, etc
}

impl Param<'_> {
    pub fn to_owned(&self) -> OwnedParam {
        match self {
            Param::Command => OwnedParam::Command,
            Param::Res(stable_id) => OwnedParam::Res(stable_id.to_owned()),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum OwnedParam {
    Command,
    Res(OwnedStableId),
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct FeatureDescriptor<'a> {
    pub name: &'a str,
    pub resources: Vec<(StableId<'a>, Vec<u8>)>,
    pub schedules: Vec<schedule::ScheduleDescriptor<'a>>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ModManifest<'a> {
    pub wasm_hash: FileHash,
    pub features: Vec<FeatureDescriptor<'a>>,
}

#[derive(Encode, Decode, PartialEq)]
pub struct FileHash([u8; 16]);

impl std::fmt::Debug for FileHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileHash(\"")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, "\")")?;
        Ok(())
    }
}

impl FileHash {
    pub fn empty() -> Self {
        Self([0; 16])
    }

    pub fn from_sha256(bytes: [u8; 32]) -> Self {
        let mut hash = [0; 16];
        hash.copy_from_slice(&bytes[..16]);
        Self(hash)
    }
}
