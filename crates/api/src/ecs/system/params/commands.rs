use bevy_reflect::FromReflect;

use crate::{
    ecs::system::{system_param::Params, SystemParam},
    runtime::{ffi_set_component, ffi_spawn_empty, serialize},
};

pub struct Commands;
impl SystemParam for Commands {
    type State = ();
    type Item<'state> = Commands;

    fn init_state() -> Self::State {
        ()
    }

    fn get_param<'state>(_state: &'state mut Self::State) -> Self::Item<'state> {
        Commands
    }

    fn get_metadata() -> Params {
        vec![common::Param::Command]
    }
}

/// Similar to bevy_ecs::system::commands::Commands
impl Commands {
    pub fn spawn_empty(&mut self) -> EntityCommands {
        let id = ffi_spawn_empty();
        EntityCommands(id)
    }
}

pub struct EntityCommands(u32);

impl EntityCommands {
    // TODO: replace with insert<T: Bundle>(&mut self, bundle: T)
    pub fn insert_component(&mut self, component: impl FromReflect) -> &mut Self {
        let type_short_name = component.reflect_short_type_path();
        let crate_name = component.reflect_crate_name().unwrap_or("unknown");
        let value = serialize(&component);
        ffi_set_component(self.0, type_short_name, crate_name, &value);
        self
    }

    pub fn id(&self) -> Entity {
        Entity(self.0)
    }
}

/// Similar to bevy's Entity
#[derive(Debug)]
pub struct Entity(u32);
