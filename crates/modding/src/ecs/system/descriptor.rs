use super::{BoxedSystem, IntoSystem, System};
use bevy_utils::all_tuples;
use harmony_modding_api as api;

/// Describes a feature's systems and what sets these belong to in order for the modloader know how to schedule and query data for them
pub struct Descriptors {
    systems: Vec<(api::SystemDescriptor, BoxedSystem)>,
    sets: Vec<api::SetDescriptor>,
}

impl Descriptors {
    pub fn empty() -> Self {
        Self {
            systems: vec![],
            sets: vec![],
        }
    }

    pub fn push(&mut self, descriptors: &mut Self) {
        self.systems.append(&mut descriptors.systems);
        self.sets.append(&mut descriptors.sets);
    }

    pub fn append(&mut self, descriptors: &mut Vec<Self>) {
        descriptors.iter_mut().for_each(|descriptors| {
            self.push(descriptors);
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
                descriptors.append(&mut vec![$($sys.into_descriptors()),*]);

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
        let mut descriptors = Descriptors::empty();

        let id = api::SystemId::from_type::<F::System>();
        let system = IntoSystem::into_system(self);
        let params = system.param_descriptors();
        let executor = Box::new(system);
        descriptors
            .systems
            .push((api::SystemDescriptor { id, params }, executor));

        descriptors
    }
}
