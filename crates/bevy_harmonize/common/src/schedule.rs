use super::*;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ScheduleDescriptor<'a> {
    pub id: StableId<'a>,
    pub schedule: Schedule<'a>,
}

/// Describes how to create a schedule
#[derive(Encode, Decode, PartialEq, Debug, Default)]
pub struct Schedule<'a> {
    pub systems: Vec<System>,
    pub constraints: Vec<Constraint<'a>>,
}

/// Constraints that define the order of systems in the schedule
///
/// These must always be checked for validity before being loaded by the modloader
#[derive(Encode, Decode, PartialEq, Debug)]
pub enum Constraint<'a> {
    /// One system set needs to run before another system set
    Order {
        before: SystemSet<'a>,
        after: SystemSet<'a>,
    },
    /// System set needs to run only if the condition is met
    Condition {
        set: SystemSet<'a>,
        condition: SystemId,
    },
    /// A system set is included in a named set
    Includes {
        parent_name: StableId<'a>,
        set: SystemSet<'a>,
    },
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct System {
    pub id: SystemId,
    pub params: Vec<ParamDescriptor>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub enum SystemSet<'a> {
    Anonymous(Vec<SystemId>),
    Named(StableId<'a>),
}
