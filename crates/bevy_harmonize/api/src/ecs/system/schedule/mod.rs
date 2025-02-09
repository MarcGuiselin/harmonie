use super::IntoSystem;
use bevy_utils_proc_macros::all_tuples;
use common::*;

/// Similar in role to bevy's IntoSystemConfigs trait
pub trait IntoSchedule<Marker>
where
    Self: Sized,
{
    fn into_configs() -> Schedule<'static>;
}

mod chain;
pub use chain::Chained;

/// An const trait that automatically applies to all [`IntoSchedule`] types
///
/// This allows us to keep [`IntoSchedule`] non-const, while allowing users to do things like chain() in const contexts
#[const_trait]
pub trait ConstrainSchedule<Marker>
where
    Self: Sized + Copy,
{
    fn chain(self) -> Chained<Marker, Self> {
        Chained::new()
    }
}

impl<Marker, T> const ConstrainSchedule<Marker> for T where T: IntoSchedule<Marker> + Copy {}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct FunctionMarker;

impl<Marker, F> IntoSchedule<(FunctionMarker, Marker)> for F
where
    F: 'static + Sized + IntoSystem<(), (), Marker> + Copy,
{
    fn into_configs() -> Schedule<'static> {
        Schedule {
            systems: vec![F::into_metadata()],
            constraints: Vec::new(),
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct DescriptorTupleMarker;

macro_rules! impl_system_collection {
    ($(($param: ident, $sys: ident)),*) => {
        /// Implement IntoSchedule for all possible sets
        impl<$($param, $sys),*> IntoSchedule<(DescriptorTupleMarker, $($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoSchedule<$param> + Copy),*
        {
            #[allow(non_snake_case)]
            fn into_configs() -> Schedule<'static> {
                let mut systems = Vec::new();
                let mut constraints = Vec::new();
                $(
                    let add = $sys::into_configs();
                    systems.extend(add.systems);
                    constraints.extend(add.constraints);
                )*
                Schedule{
                    systems,
                    constraints
                }
            }
        }
    }
}

all_tuples!(impl_system_collection, 1, 20, P, S);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Commands;

    fn get_system_id<Marker>(system: impl IntoSystem<(), (), Marker>) -> SystemId {
        SystemId::from_type(IntoSystem::get_type_id(&system))
    }

    fn make_system<Marker, F>(system: F, params: Vec<Param<'static>>) -> System<'static>
    where
        F: IntoSystem<(), (), Marker>,
    {
        System {
            id: get_system_id(system),
            name: std::any::type_name::<F::System>(),
            params,
        }
    }

    fn into_configs<T, Marker>(_systems: T) -> Schedule<'static>
    where
        T: IntoSchedule<Marker>,
    {
        T::into_configs()
    }

    #[test]
    fn into_config() {
        fn system1() {}
        fn system2(mut _commands: Commands) {}
        let system_set = (system1, system2);

        let schedule = into_configs(system_set);
        assert_eq!(
            schedule,
            Schedule {
                systems: vec![
                    make_system(system1, vec![]),
                    make_system(system2, vec![Param::Command]),
                ],
                constraints: vec![]
            }
        );
    }

    #[test]
    fn chaining() {
        fn system1() {}
        fn system2() {}
        fn system3() {}
        let system_set = (system1, system2, system3).chain();

        let schedule = into_configs(system_set);
        assert_eq!(
            schedule.constraints,
            [
                Constraint::Order {
                    before: SystemSet::Anonymous(vec![get_system_id(system1)]),
                    after: SystemSet::Anonymous(vec![get_system_id(system2)])
                },
                Constraint::Order {
                    before: SystemSet::Anonymous(vec![get_system_id(system2)]),
                    after: SystemSet::Anonymous(vec![get_system_id(system3)])
                }
            ]
        );
    }
}
