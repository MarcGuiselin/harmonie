use std::hash::{Hash, Hasher};

use crate::ModId;

use bevy_utils::{tracing::error, HashMap, HashSet};
use harmony_modloader_api::{self as api, SetDescriptor};
use thiserror::Error;

use super::Schedule;

/// Essentially the same as bevy ecs's `ScheduleGraph`
pub struct ScheduleBuilder {
    /// List of systems in the schedule
    systems: HashMap<api::SystemId, SystemNode>,
    /// List of sets in the schedule
    sets: HashSet<SystemSet>,
}

#[derive(PartialEq, Eq, Debug)]
struct SystemNode {
    params: Vec<api::ParamDescriptor>,
    /// The mod which provides this system
    mod_id: ModId,
    // TODO: conditions (system_conditions in ScheduleGraph)
}

#[derive(PartialEq, Eq)]
struct SystemSet(HashSet<api::SystemId>);

impl SystemSet {
    fn new(
        indices: api::SetIndices,
        systems: &Vec<api::SystemDescriptor>,
        preceeding_sets: &Vec<SystemSet>,
    ) -> Self {
        Self(match indices {
            api::SetIndices::System(index) => {
                let mut id = HashSet::with_capacity(1);
                id.insert(systems[index].id());
                id
            }
            api::SetIndices::Sets(indices) => {
                let mut id = HashSet::with_capacity(indices.len());
                for set_index in indices {
                    // SetDescriptors for sets only ever include the sets defined before them
                    for system_index in preceeding_sets[set_index].0.iter() {
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

impl ScheduleBuilder {
    /// Add a collection of systems and sets (using indexes relative to those)
    ///
    /// These should come from a [`ScheduleDescriptor`](api::ScheduleDescriptor)
    ///
    /// Similar to bevy ecs's `ScheduleGraph::process_configs`
    pub fn add_systems_and_relative_sets(
        &mut self,
        systems: Vec<api::SystemDescriptor>,
        sets: Vec<api::SetDescriptor>,
        mod_id: ModId,
    ) {
        // Generate and insert SystemSets
        let mut relative_sets: Vec<SystemSet> = Vec::with_capacity(sets.len());
        for SetDescriptor { indices } in sets {
            relative_sets.push(SystemSet::new(indices, &systems, &relative_sets));
        }
        for set in relative_sets {
            self.sets.insert(set);
        }

        for system in systems {
            // Mods may define systems from other mods in their own manifest
            // These can safely be ignored
            if let api::SystemDescriptor::Internal { id, params } = system {
                let replaced = self.systems.insert(id, SystemNode { mod_id, params });
                assert_eq!(replaced, None);
            }
        }
        // TODO: apply_collective_conditions

        // Mod systems do not support chaining
        // So ignore_deferred = false and chained = false
    }

    /// Build a [`Schedule`] optimized for scheduler access from the [`ScheduleGraph`].
    ///
    /// This method also
    /// - checks for dependency or hierarchy cycles
    /// - checks for system access conflicts and reports ambiguities
    pub fn build_schedule(self) -> Result<Schedule, ScheduleBuildError> {
        // check hierarchy for cycles
        self.hierarchy.topsort =
            self.topsort_graph(&self.hierarchy.graph, ReportCycles::Hierarchy)?;

        let hier_results = check_graph(&self.hierarchy.graph, &self.hierarchy.topsort);
        self.optionally_check_hierarchy_conflicts(&hier_results.transitive_edges, schedule_label)?;

        // remove redundant edges
        self.hierarchy.graph = hier_results.transitive_reduction;

        // check dependencies for cycles
        self.dependency.topsort =
            self.topsort_graph(&self.dependency.graph, ReportCycles::Dependency)?;

        // check for systems or system sets depending on sets they belong to
        let dep_results = check_graph(&self.dependency.graph, &self.dependency.topsort);
        self.check_for_cross_dependencies(&dep_results, &hier_results.connected)?;

        // map all system sets to their systems
        // go in reverse topological order (bottom-up) for efficiency
        let (set_systems, set_system_bitsets) =
            self.map_sets_to_systems(&self.hierarchy.topsort, &self.hierarchy.graph);
        self.check_order_but_intersect(&dep_results.connected, &set_system_bitsets)?;

        // check that there are no edges to system-type sets that have multiple instances
        self.check_system_type_set_ambiguity(&set_systems)?;

        let mut dependency_flattened = self.get_dependency_flattened(&set_systems);

        // modify graph with auto sync points
        if self.settings.auto_insert_apply_deferred {
            dependency_flattened = self.auto_insert_apply_deferred(&mut dependency_flattened)?;
        }

        // topsort
        let mut dependency_flattened_dag = Dag {
            topsort: self.topsort_graph(&dependency_flattened, ReportCycles::Dependency)?,
            graph: dependency_flattened,
        };

        let flat_results = check_graph(
            &dependency_flattened_dag.graph,
            &dependency_flattened_dag.topsort,
        );

        // remove redundant edges
        dependency_flattened_dag.graph = flat_results.transitive_reduction;

        // flatten: combine `in_set` with `ambiguous_with` information
        let ambiguous_with_flattened = self.get_ambiguous_with_flattened(&set_systems);

        // check for conflicts
        let conflicting_systems = self.get_conflicting_systems(
            &flat_results.disconnected,
            &ambiguous_with_flattened,
            ignored_ambiguities,
        );
        self.optionally_check_conflicts(&conflicting_systems, components, schedule_label)?;
        self.conflicting_systems = conflicting_systems;

        // build the schedule
        Ok(self.build_schedule_inner(dependency_flattened_dag, hier_results.reachable))
    }
}

/// Values returned by [`ScheduleBuilder::add_systems_and_relative_sets`]
struct AddSystemsAndSetsResult {
    /// All nodes contained inside this `process_configs` call's [`NodeConfigs`] hierarchy
    nodes: Vec<NodeId>,
    /// True if and only if all nodes are "densely chained", meaning that all nested nodes
    /// are linearly chained (as if `after` system ordering had been applied between each node)
    /// in the order they are defined
    densely_chained: bool,
}

/// Category of errors encountered during schedule construction.
#[derive(Error, Debug)]
pub enum ScheduleBuildError {
    /// A system set contains itself.
    #[error("System set `{0}` contains itself.")]
    HierarchyLoop(String),
    /// The hierarchy of system sets contains a cycle.
    #[error("System set hierarchy contains cycle(s).\n{0}")]
    HierarchyCycle(String),
    /// The hierarchy of system sets contains redundant edges.
    ///
    /// This error is disabled by default, but can be opted-in using [`ScheduleBuildSettings`].
    #[error("System set hierarchy contains redundant edges.\n{0}")]
    HierarchyRedundancy(String),
    /// A system (set) has been told to run before itself.
    #[error("System set `{0}` depends on itself.")]
    DependencyLoop(String),
    /// The dependency graph contains a cycle.
    #[error("System dependencies contain cycle(s).\n{0}")]
    DependencyCycle(String),
    /// Tried to order a system (set) relative to a system set it belongs to.
    #[error("`{0}` and `{1}` have both `in_set` and `before`-`after` relationships (these might be transitive). This combination is unsolvable as a system cannot run before or after a set it belongs to.")]
    CrossDependency(String, String),
    /// Tried to order system sets that share systems.
    #[error("`{0}` and `{1}` have a `before`-`after` relationship (which may be transitive) but share systems.")]
    SetsHaveOrderButIntersect(String, String),
    /// Tried to order a system (set) relative to all instances of some system function.
    #[error("Tried to order against `{0}` in a schedule that has more than one `{0}` instance. `{0}` is a `SystemTypeSet` and cannot be used for ordering if ambiguous. Use a different set without this restriction.")]
    SystemTypeSetAmbiguity(String),
    /// Systems with conflicting access have indeterminate run order.
    ///
    /// This error is disabled by default, but can be opted-in using [`ScheduleBuildSettings`].
    #[error("Systems with conflicting access have indeterminate run order.\n{0}")]
    Ambiguity(String),
    /// Tried to run a schedule before all of its systems have been initialized.
    #[error("Systems in schedule have not been initialized.")]
    Uninitialized,
}
