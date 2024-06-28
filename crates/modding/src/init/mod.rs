use bevy_ecs::{
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::SystemParam,
};
use bitcode::Encode;

pub struct Harmony<V: HarmonyVisitor = ()> {
    visitor: V,
}

impl<V: HarmonyVisitor> Harmony<V> {
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }

    pub fn add_translations(&mut self, path: &str) -> &mut Self {
        self.visitor.add_translations(path);
        self
    }

    pub fn add_feature<F: Feature>(&mut self, feature: F) -> &mut Self {
        self.visitor.add_feature(feature);
        self
    }
}

/// Implementation of a visitor pattern for [`Harmony`]
///
/// The init function of a mod serves several purposes, including initializing the system execution runtime.
pub trait HarmonyVisitor {
    fn new() -> Self;

    fn add_translations(&mut self, path: &str) -> &mut Self;

    fn add_feature<F: Feature>(&mut self, feature: F) -> &mut Self;
}

/// A default implementation so user can write out the init function without defining the generic type of [`Browser`]
impl HarmonyVisitor for () {
    fn new() -> Self {
        unreachable!()
    }

    fn add_translations(&mut self, _path: &str) -> &mut Self {
        unreachable!()
    }

    fn add_feature<F: Feature>(&mut self, _feature: F) -> &mut Self {
        unreachable!()
    }
}

pub trait Feature: StableId {
    fn build(&self, feature: &mut NewFeature);
}

pub struct NewFeature {
    name: &'static str,
    resources: Vec<StableIdWithData<Vec<u8>>>,
    systems: Vec<StableIdWithData<SystemConfigs>>,
}

struct StableIdWithData<T> {
    crate_name: &'static str,
    version: &'static str,
    name: &'static str,
    data: T,
}

impl<T> StableIdWithData<T> {
    fn new<S: StableId>(data: T) -> StableIdWithData<T> {
        StableIdWithData {
            crate_name: S::CRATE_NAME,
            version: S::VERSION,
            name: S::NAME,
            data,
        }
    }
}

/// A id shared between mods, used to identify objects defined in the manifest
pub trait StableId {
    const CRATE_NAME: &'static str;
    const VERSION: &'static str;
    const NAME: &'static str;
}

pub trait ScheduleLabel: StableId {}

pub trait Resource: StableId + bitcode::Encode + Default {}

impl NewFeature {
    pub fn set_name(&mut self, name: &'static str) -> &mut Self {
        self.name = name;
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources
            .push(StableIdWithData::new::<R>(bitcode::encode(&resource)));
        self
    }

    pub fn add_systems<S: ScheduleLabel, M>(
        &mut self,
        _schedule: S,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        self.systems
            .push(StableIdWithData::new::<S>(systems.into_configs()));
        self
    }
}

pub struct Start;
impl StableId for Start {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Start";
}
impl ScheduleLabel for Start {}

pub struct Update;
impl StableId for Update {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Update";
}
impl ScheduleLabel for Update {}

#[derive(SystemParam)]
pub struct Commands;

/// Similar to bevy_ecs::system::commands::Commands
impl Commands {
    pub fn spawn_empty(&mut self) -> EntityCommands {
        #[link(wasm_import_module = "harmony_mod")]
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

    pub fn id(&self) -> Entity {
        Entity(self.0)
    }
}

/// Similar to bevy's Entity
#[derive(Debug)]
pub struct Entity(u32);

/// Similar to bevy's Component
pub trait Component: StableId + Encode + Decode {
    fn get_local_component_id() -> u32;
}

pub trait Decode: for<'a> bitcode::Decode<'a> {}
impl<T> Decode for T where T: for<'a> bitcode::Decode<'a> {}
