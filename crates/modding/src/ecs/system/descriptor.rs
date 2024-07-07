use super::{BoxedSystem, IntoSystem, System};
use bevy_utils_proc_macros::all_tuples;
use harmony_modding_api as api;

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

    pub fn push(&mut self, descriptors: Self) {
        let Self {
            mut systems,
            mut sets,
        } = descriptors;

        for set in sets.iter_mut() {
            match &mut set.indices {
                api::SetIndices::System(index) => {
                    *index += self.systems.len();
                }
                api::SetIndices::Sets(indexes) => {
                    for index in indexes.iter_mut() {
                        *index += self.sets.len();
                    }
                }
            };
        }

        self.systems.append(&mut systems);
        self.sets.append(&mut sets);
    }

    pub fn append_set(&mut self, descriptors: Vec<Self>) {
        let mut offset = 0;
        let sets = descriptors
            .iter()
            .filter_map(|descriptors| {
                if descriptors.sets.is_empty() {
                    None
                } else {
                    offset += descriptors.sets.len();
                    Some(offset - 1)
                }
            })
            .collect();

        descriptors.into_iter().for_each(|descriptors| {
            self.push(descriptors);
        });
        self.sets.push(api::SetDescriptor {
            indices: api::SetIndices::Sets(sets),
        });
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

                // This is where I planned to add to sets

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
            sets: vec![api::SetDescriptor {
                indices: api::SetIndices::System(0),
            }],
        }
    }
}
