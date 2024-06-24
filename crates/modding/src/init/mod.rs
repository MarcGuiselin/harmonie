use bevy_ecs::schedule::{IntoSystemConfigs, SystemConfigs};

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

pub trait Feature {
    fn build(&self, feature: &mut NewFeature);
}

pub struct NewFeature {
    name: &'static str,
    resources: Vec<(&'static str, Vec<u8>)>,
    systems: Vec<(&'static str, SystemConfigs)>,
}

pub trait HasStableId {
    const STABLE_ID: &'static str;
}

pub trait ScheduleLabel: HasStableId {
    fn id(&self) -> &'static str {
        Self::STABLE_ID
    }
}

pub trait Resource: HasStableId + bitcode::Encode + Default {}

impl NewFeature {
    pub fn set_name(&mut self, name: &'static str) -> &mut Self {
        self.name = name;
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources
            .push((R::STABLE_ID, bitcode::encode(&resource)));
        self
    }

    pub fn add_systems<S: ScheduleLabel, M>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        self.systems.push((schedule.id(), systems.into_configs()));
        self
    }
}

pub struct Start;

impl HasStableId for Start {
    const STABLE_ID: &'static str = "start";
}
impl ScheduleLabel for Start {}

pub struct Update;
impl HasStableId for Update {
    const STABLE_ID: &'static str = "update";
}
impl ScheduleLabel for Update {}

pub struct Commands<'w, 's> {
    queue: InternalQueue<'s>,
    entities: &'w Entities,
}
