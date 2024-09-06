use bevy_utils::HashMap;
use harmony_modloader_api as api;

use super::{schedule::LoadedSchedules, LoadingError};

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub struct LoadedFeature {
    pub name: String,
    pub resources: HashMap<api::OwnedStableId, Vec<u8>>,
    pub schedules: LoadedSchedules,
}

impl LoadedFeature {
    pub fn try_from_descriptor<'a>(
        descriptor: &api::FeatureDescriptor<'a>,
    ) -> Result<Self, LoadingError> {
        let mut schedules = LoadedSchedules::new();
        for schedule in descriptor.schedules.iter() {
            schedules.add_from_descriptor(schedule)?;
        }

        Ok(Self {
            name: descriptor.name.to_owned(),
            resources: descriptor
                .resources
                .iter()
                .map(|(id, bytes)| (id.to_owned(), bytes.to_owned()))
                .collect(),
            schedules,
        })
    }
}
