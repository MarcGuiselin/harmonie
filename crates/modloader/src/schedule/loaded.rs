use bevy_utils::{HashMap, HashSet};
use harmony_modloader_api::{self as api, HasStableId, Start, Update};

use crate::mods::LoadingError;

use super::SystemSet;

#[derive(Debug)]
pub struct LoadedSchedules(HashMap<api::OwnedStableId, LoadedSchedule>);

impl LoadedSchedules {
    pub fn new() -> Self {
        let mut schedules = HashMap::default();

        // Allow only the default schedules for now
        schedules.insert(Start.get_owned_stable_id(), LoadedSchedule::default());
        schedules.insert(Update.get_owned_stable_id(), LoadedSchedule::default());

        Self(schedules)
    }

    pub fn add_from_descriptor<'a>(
        &mut self,
        feature_id: usize,
        descriptor: &api::ScheduleDescriptor<'a>,
    ) -> Result<(), LoadingError> {
        let schedule_id = descriptor.id.to_owned();
        let schedule = self
            .0
            .get_mut(&schedule_id)
            .ok_or(LoadingError::InvalidSchedule(schedule_id))?;

        schedule.add_from_descriptor(feature_id, descriptor)
    }
}

/// Essentially the same as bevy ecs's `ScheduleGraph`
#[derive(Debug, Default)]
struct LoadedSchedule {
    /// List of systems in the schedule
    systems: HashMap<api::SystemId, SystemNode>,
    /// List of sets in the schedule
    sets: HashSet<SystemSet>,
}

#[derive(PartialEq, Eq, Debug)]
struct SystemNode {
    params: Vec<api::ParamDescriptor>,
    feature_id: usize,
    // TODO: conditions (system_conditions in ScheduleGraph)
}

impl LoadedSchedule {
    fn add_from_descriptor<'a>(
        &mut self,
        feature_id: usize,
        descriptor: &api::ScheduleDescriptor<'a>,
    ) -> Result<(), LoadingError> {
        // Generate and insert SystemSets
        let mut relative_sets: Vec<SystemSet> = Vec::with_capacity(descriptor.sets.len());
        for api::SetDescriptor { indices } in &descriptor.sets {
            relative_sets.push(SystemSet::new(indices, &descriptor.systems, &relative_sets));
        }
        for set in relative_sets {
            self.sets.insert(set);
        }

        for api::SystemDescriptor { id, params } in &descriptor.systems {
            // TODO: we only need to store systems that belong to this mod
            let replaced = self.systems.insert(
                *id,
                SystemNode {
                    params: params.clone(),
                    feature_id,
                },
            );
            assert_eq!(replaced, None);
        }

        // TODO: generate graph

        Ok(())
    }
}
