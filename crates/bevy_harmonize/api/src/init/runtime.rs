use crate::ecs::system::BoxedSystem;

use super::Harmony;

struct Runtime {
    /// Systems ordered by their declaration in the manifest
    systems: Vec<BoxedSystem>,
}

static mut RUNTIME: Option<Runtime> = None;

/// Initializes the internal execution runtime
#[doc(hidden)]
#[cfg(feature = "wasm_runtime")]
pub fn __internal_initialize_runtime(harmony: Harmony) {
    let systems = harmony
        .features
        .into_iter()
        .flat_map(|f| f.boxed_systems)
        .collect::<Vec<_>>();

    // SAFETY: This is a single-threaded environment
    unsafe {
        RUNTIME = Some(Runtime { systems });
    }
}

#[doc(hidden)]
#[cfg(not(feature = "wasm_runtime"))]
pub fn __internal_initialize_runtime(_: Harmony) {
    unreachable!()
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
    #[allow(static_mut_refs)]
    let runtime = RUNTIME.as_mut().expect("Runtime not initialized");
    let system = &mut runtime.systems[index];
    system.run(());
}
