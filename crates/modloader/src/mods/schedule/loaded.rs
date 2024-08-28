use bevy_utils::HashMap;
use harmony_modloader_api::{self as api, HasStableId, Start, Update};

use crate::mods::LoadingError;

use super::ScheduleGraph;

#[derive(Debug)]
pub struct LoadedSchedules(HashMap<api::OwnedStableId, Vec<FeatureSchedule>>);

impl LoadedSchedules {
    pub fn new() -> Self {
        let mut schedules = HashMap::default();

        // Allow only the default schedules for now
        schedules.insert(Start.get_owned_stable_id(), Vec::new());
        schedules.insert(Update.get_owned_stable_id(), Vec::new());

        Self(schedules)
    }

    pub fn add_from_descriptor<'a>(
        &mut self,
        feature_id: usize,
        descriptor: &api::ScheduleDescriptor<'a>,
    ) -> Result<(), LoadingError> {
        let schedule_id = descriptor.id.to_owned();
        let schedules = self
            .0
            .get_mut(&schedule_id)
            .ok_or(LoadingError::InvalidSchedule(schedule_id))?;

        schedules.push(FeatureSchedule::try_from_descriptor(
            feature_id, descriptor,
        )?);

        Ok(())
    }
}

/// Essentially the same as bevy ecs's `ScheduleGraph`
#[derive(Debug)]
struct FeatureSchedule {
    feature_id: usize,
    /// Directed acyclic graph of the dependency (which systems/sets have to run before which other systems/sets)
    schedule: ScheduleGraph,
}

impl FeatureSchedule {
    fn try_from_descriptor<'a>(
        feature_id: usize,
        descriptor: &api::ScheduleDescriptor<'a>,
    ) -> Result<Self, LoadingError> {
        let schedule = ScheduleGraph::try_from_graph(&descriptor.schedule)
            .map_err(LoadingError::SchedulingError)?;

        Ok(Self {
            feature_id,
            schedule,
        })
    }
}
