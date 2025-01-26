use super::{BoxedSystem, IntoSystem, System};
use bevy_utils_proc_macros::all_tuples;

/// Similar in role to bevy's IntoSystemConfigs trait
pub trait IntoSystemConfigs<Marker>
where
    Self: Sized,
{
    fn add_to_schedule(this: Self, schedule: &mut common::Schedule);

    fn add_to_boxed_systems(this: Self, boxed_systems: &mut Vec<BoxedSystem>);
}

impl<Marker, F> IntoSystemConfigs<Marker> for F
where
    F: 'static + Sized + IntoSystem<(), (), Marker>,
{
    fn add_to_schedule(this: Self, schedule: &mut common::Schedule) {
        let id = common::SystemId::from_type::<F::System>();
        let system = IntoSystem::into_system(this);
        let params = system.params();

        schedule.systems.push(common::System { id, params });
    }

    fn add_to_boxed_systems(this: Self, boxed_systems: &mut Vec<BoxedSystem>) {
        boxed_systems.push(Box::new(F::into_system(this)));
    }
}

#[doc(hidden)]
pub struct DescriptorTupleMarker;

macro_rules! impl_system_collection {
    ($(($param: ident, $sys: ident)),*) => {
        /// Implement IntoSystemDescriptors for all possible sets
        impl<$($param, $sys),*> IntoSystemConfigs<(DescriptorTupleMarker, $($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoSystemConfigs<$param>),*
        {
            #[allow(non_snake_case)]
            fn add_to_schedule(this: Self, schedule: &mut common::Schedule) {
                let ($($sys,)*) = this;
                $(
                    $sys::add_to_schedule($sys, schedule);
                )*
            }

            #[allow(non_snake_case)]
            fn add_to_boxed_systems(this: Self, boxed_systems: &mut Vec<BoxedSystem>) {
                let ($($sys,)*) = this;
                $(
                    $sys::add_to_boxed_systems($sys, boxed_systems);
                )*
            }
        }
    }
}

all_tuples!(impl_system_collection, 1, 20, P, S);
