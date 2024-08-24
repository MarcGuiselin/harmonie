use std::{
    fmt,
    hash::{Hash, Hasher},
};

use bevy_utils::HashSet;
use harmony_modloader_api as api;

#[derive(PartialEq, Eq)]
pub(crate) struct SystemSet(HashSet<api::SystemId>);

impl fmt::Debug for SystemSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SystemSet[")?;
        for (i, id) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{:x}\"", id.get_raw())?;
        }
        write!(f, "]")
    }
}

impl SystemSet {
    pub fn new(systems: &Vec<api::SystemId>) -> Self {
        let mut id = HashSet::new();
        for system in systems {
            id.insert(*system);
        }
        Self(id)
    }
}

impl Hash for SystemSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Note: order will remain the same regardless of id insertion order
        for index in self.0.iter() {
            index.hash(state);
        }
    }
}
