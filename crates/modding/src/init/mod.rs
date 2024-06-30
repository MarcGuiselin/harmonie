use crate::ecs::{system::IntoDescriptors, StableId, StableIdWithData};
use harmony_modding_api::Descriptor;

pub struct Harmony<V = ()>
where
    V: HarmonyVisitor,
{
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
    systems: Vec<StableIdWithData<Vec<Descriptor>>>,
}

pub trait ScheduleLabel
where
    Self: StableId,
{
}

pub trait Resource
where
    Self: StableId + bitcode::Encode + Default,
{
}

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
        systems: impl IntoDescriptors<M>,
    ) -> &mut Self {
        self.systems
            .push(StableIdWithData::new::<S>(systems.into_descriptors()));
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
