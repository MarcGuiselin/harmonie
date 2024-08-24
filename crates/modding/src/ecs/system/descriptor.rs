use super::{BoxedSystem, IntoSystem, System};
use bevy_utils_proc_macros::all_tuples;
use harmony_modloader_api as api;

/// Describes a feature's systems and what sets these belong to in order for the modloader know how to schedule and query data for them
pub struct Descriptors {
    pub(crate) systems: Vec<(api::SystemDescriptor, BoxedSystem)>,
    pub(crate) sets: Vec<api::SetDescriptor>,
}

impl std::fmt::Debug for Descriptors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Descriptors")
            .field(
                "systems",
                &self
                    .systems
                    .iter()
                    .map(|(desc, _)| desc)
                    .collect::<Vec<_>>(),
            )
            .field("sets", &self.sets)
            .finish()
    }
}

impl Descriptors {
    pub fn empty() -> Self {
        Self {
            systems: vec![],
            sets: vec![],
        }
    }

    pub fn push(&mut self, mut descriptors: Self) {
        self.systems.append(&mut descriptors.systems);
        self.sets.append(&mut descriptors.sets);
    }

    pub fn append_set(&mut self, set_descriptors: Vec<Self>) {
        let mut systems = Vec::new();
        for descriptors in set_descriptors {
            // The last set descriptor must describe that set (and not it's dependencies)
            descriptors
                .sets
                .last()
                .map(|set| systems.extend(set.systems.iter()));

            self.push(descriptors);
        }
        self.sets.push(api::SetDescriptor { systems });
    }
}

/// Similar in role to bevy's IntoSystemConfigs trait
pub trait IntoDescriptors<Marker> {
    fn into_descriptors(self) -> Descriptors;
}

#[doc(hidden)]
pub struct DescriptorTupleMarker;

macro_rules! impl_system_collection {
    ($(($param: ident, $sys: ident)),*) => {
        /// Implement IntoSystemDescriptors for all possible sets
        impl<$($param, $sys),*> IntoDescriptors<(DescriptorTupleMarker, $($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoDescriptors<$param>),*
        {
            #[allow(non_snake_case)]
            fn into_descriptors(self) -> Descriptors {
                let mut descriptors = Descriptors::empty();

                let ($($sys,)*) = self;
                descriptors.append_set(vec![$($sys.into_descriptors()),*]);

                descriptors
            }
        }
    }
}

all_tuples!(impl_system_collection, 1, 20, P, S);

impl<Marker, F> IntoDescriptors<Marker> for F
where
    F: 'static + IntoSystem<(), (), Marker>,
{
    fn into_descriptors(self) -> Descriptors {
        let id = api::SystemId::from_type::<F::System>();
        let system = IntoSystem::into_system(self);
        let params = system.param_descriptors();
        let executor = Box::new(system);

        Descriptors {
            systems: vec![(api::SystemDescriptor { id, params }, executor)],
            sets: vec![api::SetDescriptor { systems: vec![id] }],
        }
    }
}
