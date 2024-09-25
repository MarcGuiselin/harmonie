use crate::ecs::{system::*, Resource};

pub trait Feature: common::HasStableId {
    fn build(&self, feature: &mut FeatureBuilder);
}

pub struct FeatureBuilder {
    pub(crate) name: &'static str,
    pub(crate) resources: Vec<(common::StableId<'static>, Vec<u8>)>,
    #[cfg(feature = "wasm_runtime")]
    pub(crate) boxed_systems: Vec<BoxedSystem>,
    #[cfg(feature = "generate_manifest")]
    pub(crate) schedules: Vec<common::ScheduleDescriptor<'static>>,
}

impl Default for FeatureBuilder {
    fn default() -> Self {
        Self {
            name: "",
            resources: Vec::new(),
            #[cfg(feature = "wasm_runtime")]
            boxed_systems: Vec::new(),
            #[cfg(feature = "generate_manifest")]
            schedules: Vec::new(),
        }
    }
}

impl std::fmt::Debug for FeatureBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("FeatureBuilder");
        debug.field("name", &self.name);
        debug.field("resources", &self.resources);

        #[cfg(feature = "generate_manifest")]
        debug.field("schedules", &self.schedules);

        debug.finish()
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

    pub fn add_systems<S: common::ScheduleLabel, M>(
        &mut self,
        schedule: S,
        systems: impl IntoSystemConfigs<M>,
    ) -> &mut Self {
        let _schedule = schedule;
        let _systems = systems;

        #[cfg(feature = "generate_manifest")]
        {
            let id = _schedule.get_stable_id();
            let descriptor = crate::utils::find_mut_or_push(
                &mut self.schedules,
                |s| s.id == id,
                || common::ScheduleDescriptor {
                    id,
                    schedule: Default::default(),
                },
            );

            IntoSystemConfigs::add_to_schedule(_systems, &mut descriptor.schedule);
        }
        #[cfg(feature = "wasm_runtime")]
        {
            IntoSystemConfigs::add_to_boxed_systems(_systems, &mut self.boxed_systems);
        }
        self
    }
}
