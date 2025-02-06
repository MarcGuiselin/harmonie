use super::{BoxedSystem, IntoSystem};
use bevy_utils_proc_macros::all_tuples;

pub struct SystemConfig(common::Schedule<'static>);

/// Similar in role to bevy's IntoSystemConfigs trait
pub trait IntoSystemConfigs<Marker>
where
    Self: Sized,
{
    fn into_configs(self) -> SystemConfig;

    fn add_to_boxed_systems(self, boxed_systems: &mut Vec<BoxedSystem>);
}

impl<Marker, F> IntoSystemConfigs<Marker> for F
where
    F: 'static + Sized + IntoSystem<(), (), Marker>,
{
    fn into_configs(self) -> SystemConfig {
        SystemConfig(common::Schedule {
            systems: vec![F::into_metadata()],
            constraints: Vec::new(),
        })
    }

    fn add_to_boxed_systems(self, boxed_systems: &mut Vec<BoxedSystem>) {
        boxed_systems.push(Box::new(F::into_system(self)));
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
            fn into_configs(self) -> SystemConfig {
                let ($($sys,)*) = self;
                let mut systems = Vec::new();
                let mut constraints = Vec::new();
                $(
                    let add = $sys.into_configs();
                    systems.extend(add.0.systems);
                    constraints.extend(add.0.constraints);
                )*
                SystemConfig(common::Schedule {
                    systems,
                    constraints,
                })
            }

            #[allow(non_snake_case)]
            fn add_to_boxed_systems(self, boxed_systems: &mut Vec<BoxedSystem>) {
                let ($($sys,)*) = self;
                $(
                    $sys::add_to_boxed_systems($sys, boxed_systems);
                )*
            }
        }
    }
}

all_tuples!(impl_system_collection, 1, 20, P, S);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Commands;

    #[test]
    fn into_config() {
        fn system1() {}
        fn system2(mut _commands: Commands) {}
        let system_set = (system1, system2);

        let config = system_set.into_configs();
        let systems = config.0.systems;
        assert_eq!(systems.len(), 2);
        assert_eq!(systems[0].params, Vec::new());
        assert_eq!(systems[1].params, vec![common::Param::Command]);
    }

    #[test]
    fn add_to_boxed_systems() {
        static mut COUNT: u8 = 0;

        fn system1() {
            unsafe {
                COUNT += 1;
            }
        }
        fn system2(mut _commands: Commands) {
            unsafe {
                COUNT += 2;
            }
        }
        let system_set = (system1, system2);

        let mut boxed_systems = Vec::new();
        system_set.add_to_boxed_systems(&mut boxed_systems);
        assert_eq!(boxed_systems.len(), 2);
        for system in boxed_systems.iter_mut() {
            system.run(());
        }
        assert_eq!(unsafe { COUNT }, 3);
    }
}
