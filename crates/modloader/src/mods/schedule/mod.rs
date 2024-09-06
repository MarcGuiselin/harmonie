use harmony_modloader_api as api;

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub enum SchedulingError {
    SystemDeclaredTwice(api::SystemId),
    Cycles {
        named_set: Option<String>,
        scc_with_cycles: (),
    },
    EmptyAnonymousSet,
}
