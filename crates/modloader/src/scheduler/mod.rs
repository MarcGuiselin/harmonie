use bevy_ecs_macros::Resource;
use bevy_utils::HashMap;
use harmony_modloader_api::{self as api, HasStableId, Start, Update};

mod builder;

use crate::ModId;

#[derive(Resource)]
pub struct ModSchedules(HashMap<api::StableId<'static>, Schedule>);

impl Default for ModSchedules {
    fn default() -> Self {
        let mut schedules = HashMap::default();

        // Allow only the default schedules for now
        schedules.insert(Start.get_stable_id(), Schedule::default());
        schedules.insert(Update.get_stable_id(), Schedule::default());

        Self(schedules)
    }
}

/// Our mod schedules are initiated and executed differently than bevy's.
///
/// Unlike bevy, [`Schedule`] does not contain the graph, since it is
/// rebuilt and discarded every time mods are changed (using
/// [`ScheduleBuilder`](builder::ScheduleBuilder)).
///
/// It also doesn't contain an executor, since that is handled by a
/// dedicated system.
///
/// Thus [`Schedule`] here is more similar to bevy ecs's `SystemSchedule`.
#[derive(Default)]
pub struct Schedule {
    systems: Vec<ScheduledSystem>,
    sets: Vec<ScheduledSet>,
}

/// System in the Schedule with associated metadata
pub struct ScheduledSystem {
    id: api::SystemId,
    /// The mod which provides this system
    mod_id: ModId,
    /// Number of systems that the system immediately depends on.
    dependencies: usize,
    /// List of systems that immediately depend on the system.
    dependents: Vec<usize>,
    // TODO: conditions
    // TODO: sets_with_conditions_of_systems
}

/// Set in the Schedule with associated metadata
pub struct ScheduledSet {
    systems: Vec<usize>,
    // TODO: conditions
    // TODO: systems_in_sets_with_conditions
}
