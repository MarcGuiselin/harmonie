use bevy_utils::HashMap;
use harmony_modloader_api as api;

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub struct Feature {
    pub name: String,
    pub resources: HashMap<api::OwnedStableId, Vec<u8>>,
}

impl Feature {
    pub fn from_descriptor<'a>(descriptor: &api::FeatureDescriptor<'a>) -> Self {
        Self {
            name: descriptor.name.to_owned(),
            resources: descriptor
                .resources
                .iter()
                .map(|(id, bytes)| (id.to_owned(), bytes.to_owned()))
                .collect(),
        }
    }
}
