use harmony_modloader_api as api;

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub enum SchedulingError {
    SystemDeclaredTwice(api::SystemId),
    Cycles {
        named_set: Option<String>,
        cycles: Vec<Cycle>,
    },
    EmptyAnonymousSet,
}

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub struct Cycle(pub Vec<api::SystemId>);
