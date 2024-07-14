use crate::ecs::system::BoxedSystem;

use super::Harmony;

struct Runtime {
    /// Systems ordered by their declaration in the manifest
    systems: Vec<BoxedSystem>,
}

static mut RUNTIME: Option<Runtime> = None;

/// Initializes the internal execution runtime
#[doc(hidden)]
pub fn __internal_initialize_runtime(harmony: Harmony) {
    let systems = harmony
        .features
        .into_iter()
        .flat_map(|feature| feature.descriptors.into_iter())
        .flat_map(|(_, descriptor)| descriptor.systems.into_iter())
        .map(|(_, system)| system)
        .collect();

    // SAFETY: This is a single-threaded environment
    unsafe {
        RUNTIME = Some(Runtime { systems });
    }
}

/// Runs a system by its index
///
/// # Safety
/// Caller must ensure that the:
/// - runtime has already been initialized
/// - system index is valid
/// - system index is not already running
pub unsafe fn __internal_run_system(index: usize) {
    // SAFETY: This is a single-threaded environment
    let runtime = RUNTIME.as_mut().expect("Runtime not initialized");
    let system = &mut runtime.systems[index];
    system.run(());
}
