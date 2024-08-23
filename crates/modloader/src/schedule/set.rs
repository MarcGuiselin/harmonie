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
    pub fn new(
        indices: &api::SetIndices,
        systems: &Vec<api::SystemDescriptor>,
        preceeding_sets: &Vec<SystemSet>,
    ) -> Self {
        Self(match indices {
            api::SetIndices::System(index) => {
                let mut id = HashSet::with_capacity(1);
                id.insert(systems[*index].id);
                id
            }
            api::SetIndices::Sets(indices) => {
                let mut id = HashSet::with_capacity(indices.len());
                for set_index in indices {
                    // SetDescriptors for sets only ever include the sets defined before them
                    for system_index in preceeding_sets[*set_index].0.iter() {
                        id.insert(*system_index);
                    }
                }
                id
            }
        })
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
