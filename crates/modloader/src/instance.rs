use std::{path::Path, sync::Arc};

use bevy_ecs::world::World;
use bevy_ecs_macros::Resource;
use bevy_tasks::Task;
use bevy_utils::HashMap;
use harmony_modloader_api as api;

#[derive(PartialEq, Eq, Debug)]
pub struct ModId(usize);

/// Holds an vec of mods indexed by [ModId]
#[derive(Resource, Default)]
pub struct Mods {
    loading: Vec<Task<LoadedMod>>,
    mods: Vec<Mod>,
}

impl Mods {
    fn load_mod(_path: &Path) {
        unimplemented!()
    }
}

pub struct LoadedMod {
    // TODO
}

/// Maps systems to mod ids
///
/// This is used mostly by the executor to know which mod to invoke when a system is called
#[derive(Resource, Default)]
pub struct SystemMods(Arc<HashMap<api::SystemId, ModId>>);

enum Mod {
    Loaded {
        // TODO
        instances: Vec<ModInstance>,
        world: World,
    },
    Unloaded {
        // TODO
        // Not sure if this is a good pattern
    },
}

// Holds one wasmer instance of a loaded mod and memory
struct ModInstance {
    // TODO
}
