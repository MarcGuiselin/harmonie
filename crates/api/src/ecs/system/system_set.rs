use super::IntoSystem;
use bevy_reflect::Typed;
use bevy_utils_proc_macros::all_tuples;
use common::*;

/// Similar in role to bevy's IntoSystemConfigs trait
pub trait IntoSystemSet<Marker> {
    fn into_system_set() -> SystemSet<'static>;

    fn into_systems_vec() -> Vec<System<'static>> {
        Vec::new()
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemMarker;

// Implement for anonymous functions
impl<Marker, F> IntoSystemSet<(SystemMarker, Marker)> for F
where
    F: 'static + IntoSystem<(), (), Marker> + Copy,
{
    fn into_system_set() -> SystemSet<'static> {
        SystemSet::Anonymous(vec![SystemId::of::<F::System>()])
    }

    fn into_systems_vec() -> Vec<System<'static>> {
        vec![F::into_metadata()]
    }
}

impl<T> IntoSystemSet<()> for T
where
    T: Typed,
{
    fn into_system_set() -> SystemSet<'static> {
        SystemSet::Named(StableId::from_typed::<T>())
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct TupleMarker;

macro_rules! impl_system_collection {
    ($(($param: ident, $sys: ident)),*) => {
        /// Implement IntoSystemSet for all possible sets
        impl<$($param, $sys),*> IntoSystemSet<(TupleMarker, $($param,)*)> for ($($sys,)*)
        where
            $($sys: IntoSystemSet<$param> + Copy),*
        {
            fn into_system_set() -> SystemSet<'static> {
                let mut anonymous_systems = Vec::new();
                $(
                    match $sys::into_system_set() {
                        SystemSet::Anonymous(systems) => {
                            anonymous_systems.extend(systems);
                        }
                        _ => {
                            panic!("A system set must be Anonymous or Named, but not a mix of both. For example (anonymous_system, NamedSystemSet) is an invalid system set.");
                        }
                    };
                )*

                SystemSet::Anonymous(anonymous_systems)
            }

            #[allow(non_snake_case)]
            fn into_systems_vec() -> Vec<System<'static>> {
                let mut systems = Vec::new();
                $(
                    systems.extend($sys::into_systems_vec());
                )*
                systems
            }
        }
    }
}

all_tuples!(impl_system_collection, 1, 20, P, S);

#[cfg(test)]
mod tests {
    use bevy_reflect::Reflect;

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

    fn into_system_set<T, Marker>(_systems: T) -> SystemSet<'static>
    where
        T: IntoSystemSet<Marker>,
    {
        T::into_system_set()
    }

    fn into_systems_vec<T, Marker>(_systems: T) -> Vec<System<'static>>
    where
        T: IntoSystemSet<Marker>,
    {
        T::into_systems_vec()
    }

    #[test]
    fn anonymous_system_into_system_set() {
        fn system(mut _commands: Commands) {}
        let system_set = system;

        assert_eq!(
            into_system_set(system_set),
            SystemSet::Anonymous(vec![get_system_id(system)])
        );
        assert_eq!(
            into_systems_vec(system_set),
            vec![make_system(system, vec![Param::Command])]
        );
    }

    #[test]
    fn named_into_system_set() {
        #[derive(Reflect, Clone, Copy)]
        struct NamedSet;
        let system_set = NamedSet;

        assert_eq!(
            into_system_set(system_set),
            SystemSet::Named(StableId::from_typed::<NamedSet>())
        );
        assert_eq!(into_systems_vec(system_set), Vec::new());
    }

    #[test]
    fn system_tuple_into_system_set() {
        fn system1() {}
        fn system2(mut _commands: Commands) {}
        let system_set = (system1, system2);

        assert_eq!(
            into_system_set(system_set),
            SystemSet::Anonymous(vec![get_system_id(system1), get_system_id(system2),])
        );
        assert_eq!(
            into_systems_vec(system_set),
            vec![
                make_system(system1, vec![]),
                make_system(system2, vec![Param::Command]),
            ]
        );
    }

    #[test]
    fn invalid_system_set() {
        fn anonymous_system() {}
        #[derive(Reflect, Clone, Copy)]
        struct NamedSet;
        let system_set = (anonymous_system, NamedSet);

        // This is not a valid system set
        let result = std::panic::catch_unwind(|| {
            let _ = into_system_set(system_set);
        });
        assert!(result.is_err());

        // But it is a valid array of systems
        assert_eq!(
            into_systems_vec(system_set),
            vec![make_system(anonymous_system, vec![])]
        );
    }
}
