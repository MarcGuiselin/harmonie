use crate::ecs::{
    system::{Descriptors, IntoDescriptors},
    Resource,
};
use harmony_modloader_api as api;

pub trait Feature: api::HasStableId {
    fn build(&self, feature: &mut FeatureBuilder);
}

pub struct FeatureBuilder {
    pub(crate) name: &'static str,
    pub(crate) resources: Vec<(api::StableId<'static>, Vec<u8>)>,
    pub(crate) descriptors: Vec<(api::StableId<'static>, Descriptors)>,
}

impl std::fmt::Debug for FeatureBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureBuilder")
            .field("name", &self.name)
            .field("resources", &self.resources)
            .field("descriptors", &self.descriptors)
            .finish()
    }
}

impl FeatureBuilder {
    pub fn set_name(&mut self, name: &'static str) -> &mut Self {
        self.name = name;
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources
            .push((resource.get_stable_id(), bitcode::encode(&resource)));
        self
    }

    pub fn add_systems<S: api::ScheduleLabel, M>(
        &mut self,
        schedule: S,
        systems: impl IntoDescriptors<M>,
    ) -> &mut Self {
        let schedule = schedule.get_stable_id();
        let descriptors = IntoDescriptors::into_descriptors(systems);

        self.add_descriptor(schedule, descriptors)
    }

    fn add_descriptor(
        &mut self,
        schedule: api::StableId<'static>,
        descriptors: Descriptors,
    ) -> &mut Self {
        if let Some((_, desc)) = self.descriptors.iter_mut().find(|(id, _)| *id == schedule) {
            desc.push(descriptors);
        } else {
            self.descriptors.push((schedule, descriptors));
        }
        self
    }
}
