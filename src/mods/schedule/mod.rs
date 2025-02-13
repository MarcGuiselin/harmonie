// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub enum SchedulingError {
    SystemDeclaredTwice(common::SystemId),
    Cycles {
        named_set: Option<String>,
        cycles: Vec<Cycle>,
    },
    EmptyAnonymousSet,
}

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub struct Cycle(pub Vec<common::SystemId>);
