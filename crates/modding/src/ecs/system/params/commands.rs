use crate::ecs::{
    system::{ParamDescriptors, SystemParam},
    Component,
};
use harmony_modloader_api as api;

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

    fn get_descriptors() -> ParamDescriptors {
        vec![api::ParamDescriptor::Command]
    }
}

/// Similar to bevy_ecs::system::commands::Commands
impl Commands {
    #[cfg(feature = "wasm_runtime")]
    pub fn spawn_empty(&mut self) -> EntityCommands {
        #[link(wasm_import_module = "harmony_mod")]
        extern "C" {
            fn command_spawn_empty() -> u32;
        }

        let id = unsafe { command_spawn_empty() };
        EntityCommands(id)
    }

    #[cfg(not(feature = "wasm_runtime"))]
    pub fn spawn_empty(&mut self) -> EntityCommands {
        unreachable!()
    }
}

pub struct EntityCommands(u32);

impl EntityCommands {
    // TODO: replace with insert<T: Bundle>(&mut self, bundle: T)
    #[cfg(feature = "wasm_runtime")]
    pub fn insert_component<T: Component>(&mut self, component: T) -> &mut Self {
        #[link(wasm_import_module = "harmony_mod")]
        extern "C" {
            fn entity_insert_component(
                entity_id: u32,
                local_component_id: u32,
                component_buffer_ptr: u32,
                component_buffer_len: u32,
            );
        }

        let component_buffer = bitcode::encode(&component);
        unsafe {
            entity_insert_component(
                self.0,
                <T as Component>::get_local_component_id(),
                component_buffer.as_ptr() as _,
                component_buffer.len() as _,
            );
        }
        self
    }

    #[cfg(not(feature = "wasm_runtime"))]
    pub fn insert_component<T: Component>(&mut self, _: T) -> &mut Self {
        unreachable!()
    }

    pub fn id(&self) -> Entity {
        Entity(self.0)
    }
}

/// Similar to bevy's Entity
#[derive(Debug)]
pub struct Entity(u32);
