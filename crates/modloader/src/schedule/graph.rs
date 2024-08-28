use std::hash::{Hash, Hasher};

use bevy_utils::{HashMap, HashSet};
use harmony_modloader_api::api;
use petgraph::prelude::*;

type Dag = DiGraphMap<Node, ()>;

#[derive(Debug, Default)]
pub struct ScheduleGraph {
    /// Directed acyclic graph of the dependency (which systems/sets have to run before which other systems/sets)
    dependency: Dag,
}

impl ScheduleGraph {
    pub fn try_from_graph(graph: &api::Schedule) -> Result<Self, SchedulingError> {
        let mut builder = Builder::new(&graph);

        // Add constraints to the dependency graph
        for constraint in graph.constraints.iter() {
            builder.add_constraint(constraint)?;
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
    fn new(graph: &api::Schedule) -> Self {
        let mut builder = Self::default();

        // Populate the dependency graph nodes
        for systems in graph.systems.iter() {
            builder.dependency.add_node(Node::System(systems.id));
        }

        builder
    }

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

    fn build(self) -> ScheduleGraph {
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

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub enum SchedulingError {
    SystemDeclaredTwice(api::SystemId),
    Cycles {
        named_set: Option<String>,
        scc_with_cycles: Vec<Vec<Node>>,
    },
    EmptyAnonymousSet,
}
