use bevy_ecs::system::IntoSystem;
use bevy_utils::all_tuples;
use harmony_modding_api::Descriptor;

/// Similar in role to bevy's IntoSystemConfigs trait
pub trait IntoDescriptors<Marker> {
    fn into_descriptors(self) -> Vec<Descriptor>;
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
            fn into_descriptors(self) -> Vec<Descriptor> {
                let ($($sys,)*) = self;
                vec![$($sys.into_descriptors()),*]
                    .into_iter()
                    .flatten()
                    .collect()
            }
        }
    }
}

all_tuples!(impl_system_collection, 1, 20, P, S);

impl<Marker, F> IntoDescriptors<Marker> for F
where
    F: IntoSystem<(), (), Marker>,
{
    fn into_descriptors(&self) -> Vec<Self> {
        vec![]
    }
}
