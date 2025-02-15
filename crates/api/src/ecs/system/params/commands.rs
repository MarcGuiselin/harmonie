use crate::{
    ecs::{
        system::{system_param::Params, SystemParam},
        Reflected,
    },
    runtime::serialize,
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
        #[link(wasm_import_module = "bevy_harmonize")]
        extern "C" {
            fn command_spawn_empty() -> u32;
        }

        let id = unsafe { command_spawn_empty() };
        EntityCommands(id)
    }
}

pub struct EntityCommands(u32);

impl EntityCommands {
    // TODO: replace with insert<T: Bundle>(&mut self, bundle: T)
    pub fn insert_component(&mut self, component: impl Reflected) -> &mut Self {
        #[link(wasm_import_module = "bevy_harmonize")]
        extern "C" {
            fn entity_insert_component(
                entity_id: u32,
                type_short_name_ptr: u32,
                type_short_name_len: u32,
                type_crate_name_ptr: u32,
                type_crate_name_len: u32,
                buffer_ptr: u32,
                buffer_len: u32,
            );
        }

        let type_short_name = component.reflect_short_type_path();
        let crate_name = component.reflect_crate_name().unwrap_or("unknown");

        let component_buffer = serialize(&component);

        unsafe {
            entity_insert_component(
                self.0,
                type_short_name.as_ptr() as _,
                type_short_name.len() as _,
                crate_name.as_ptr() as _,
                crate_name.len() as _,
                component_buffer.as_ptr() as _,
                component_buffer.len() as _,
            );
        }
        self
    }

    pub fn id(&self) -> Entity {
        Entity(self.0)
    }
}

/// Similar to bevy's Entity
#[derive(Debug)]
pub struct Entity(u32);
