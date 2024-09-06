use std::hash::{Hash, Hasher};

use bevy_utils::{HashMap, HashSet};
use harmony_modloader_api::{self as api, HasStableId, Start, Update};
use petgraph::prelude::*;

use crate::mods::SchedulingError;

use super::LoadingError;

type Dag = DiGraphMap<Node, ()>;

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub struct LoadedSchedules(HashMap<api::OwnedStableId, LoadedSchedule>);

impl LoadedSchedules {
    pub fn try_from_schedule_descriptors<'a>(
        descriptors: &Vec<api::ScheduleDescriptor<'a>>,
    ) -> Result<Self, LoadingError> {
        let mut schedules = HashMap::default();

        // Allow only the default schedules for now
        schedules.insert(Start.get_owned_stable_id(), Vec::new());
        schedules.insert(Update.get_owned_stable_id(), Vec::new());

        // Group together schedules with the same schedule id
        for descriptor in descriptors {
            let schedule_id = descriptor.id.to_owned();
            schedules
                .get_mut(&schedule_id)
                .ok_or(LoadingError::InvalidSchedule(schedule_id))?
                .push(&descriptor.schedule);
        }

        let mut inner = HashMap::default();
        for (id, schedules) in schedules.into_iter().filter(|(_, v)| !v.is_empty()) {
            if !schedules.is_empty() {
                let loaded = LoadedSchedule::try_from_schedules(&schedules[..])
                    .map_err(LoadingError::SchedulingError)?;
                inner.insert(id, loaded);
            }
        }

        Ok(Self(inner))
    }
}

#[derive(Debug, Default)]
pub struct LoadedSchedule {
    // TODO
}

impl LoadedSchedule {
    pub fn try_from_schedules(schedules: &[&api::Schedule]) -> Result<Self, SchedulingError> {
        let mut builder = Builder::default();

        // Populate the dependency graph nodes
        for schedule in schedules {
            for system in schedule.systems.iter() {
                builder.dependency.add_node(Node::System(system.id));
            }
        }

        // Add constraints to the dependency graph
        for schedule in schedules {
            for constraint in schedule.constraints.iter() {
                builder.add_constraint(constraint)?;
            }
        }

        Ok(builder.build())
    }
}

#[derive(Default)]
struct Builder {
    dependency: Dag,
    sets: HashMap<SystemSet, usize>,
}

impl Builder {
    fn add_constraint(&mut self, constraint: &api::Constraint) -> Result<(), SchedulingError> {
        match constraint {
            api::Constraint::Before { a, b } => {
                let (_, end_a) = self.populate_set_nodes(a)?;
                let (start_b, _) = self.populate_set_nodes(b)?;

                // The last node of a must run before the first node of b
                self.dependency.add_edge(end_a, start_b, ());
            }
            api::Constraint::Condition { set, condition } => {
                let condition = Node::System(*condition);
                let (start_set, _) = self.populate_set_nodes(set)?;

                // The condition must run before the first node of the set
                self.dependency.add_edge(condition, start_set, ());
            }
            api::Constraint::Includes { parent_name, set } => {
                let parent = SystemSet::Named(parent_name.to_owned());
                let (start_parent, end_parent) = self.populate_set_nodes_inner(parent);
                let (start_set, end_set) = self.populate_set_nodes(set)?;

                // The child set must run within the parent set
                // So the first node of the child set must run after the first node of the parent set
                // And the last node of the child set must run before the last node of the parent set
                self.dependency.add_edge(start_parent, start_set, ());
                self.dependency.add_edge(end_set, end_parent, ());
            }
        }
        Ok(())
    }

    /// For a given set, resolves the start and end nodes
    fn populate_set_nodes(
        &mut self,
        set: &api::SystemSet,
    ) -> Result<(Node, Node), SchedulingError> {
        match set {
            api::SystemSet::Anonymous(systems) => match systems.len() {
                0 => Err(SchedulingError::EmptyAnonymousSet),
                1 => {
                    let id = Node::System(systems[0]);
                    self.dependency.add_node(id);
                    Ok((id, id))
                }
                _ => {
                    let mut set = HashSet::new();
                    for system in systems {
                        set.insert(*system);
                    }
                    let set = SystemSet::Anonymous(set);
                    Ok(self.populate_set_nodes_inner(set))
                }
            },
            api::SystemSet::Named(name) => {
                let set = SystemSet::Named(name.to_owned());
                Ok(self.populate_set_nodes_inner(set))
            }
        }
    }

    fn populate_set_nodes_inner(&mut self, set: SystemSet) -> (Node, Node) {
        let id = self.sets.get(&set).map(|id| *id).unwrap_or_else(|| {
            let id = self.sets.len();

            // Create a before and after node for the anonymous set
            self.dependency.add_node(Node::SetStart(id));
            self.dependency.add_node(Node::SetEnd(id));

            // If this is an anonymous set, link its dependencies
            if let SystemSet::Anonymous(systems) = &set {
                for system in systems {
                    self.dependency.add_node(Node::System(*system));
                    self.dependency
                        .add_edge(Node::SetStart(id), Node::System(*system), ());
                    self.dependency
                        .add_edge(Node::System(*system), Node::SetEnd(id), ());
                }
            }

            self.sets.insert(set, id);
            id
        });
        (Node::SetStart(id), Node::SetEnd(id))
    }

    fn build(self) -> LoadedSchedule {
        unimplemented!()
    }
}

#[derive(PartialEq, Eq)]
pub enum SystemSet {
    Anonymous(HashSet<api::SystemId>),
    Named(api::OwnedStableId),
}

impl Hash for SystemSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Anonymous(systems) => {
                // Note: order will remain the same regardless of id insertion order
                for system in systems {
                    system.hash(state);
                }
            }
            Self::Named(id) => id.hash(state),
        }
    }
}

struct System {
    params: Vec<api::ParamDescriptor>,
    run_conditions: Vec<api::SystemId>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
enum Node {
    System(api::SystemId),
    SetStart(usize),
    SetEnd(usize),
}
