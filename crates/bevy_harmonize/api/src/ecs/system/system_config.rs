use super::IntoSystem;
use bevy_utils_proc_macros::all_tuples;
use common::*;
use const_panic::concat_panic;
use const_vec::ConstVec;

/// Similar in role to bevy's IntoSystemConfigs trait
#[const_trait]
pub trait IntoSystemConfigs<Marker>
where
    Self: Sized,
{
    fn into_configs(self) -> SystemConfigs;

    fn chain(self) -> SystemConfigs {
        let mut config = self.into_configs();

        match config.chain {
            Chain::None => {
                config.chain = Chain::All;
                config
            }
            Chain::All => concat_panic!("Systems are already chained",),
            Chain::Impossible => concat_panic!("You can only chain a tuple of systems",),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemConfigs {
    systems: ConstVec<fn() -> System<'static>, 123>,
    chain: Chain,
    // ... before: ConstVec<fn() -> SystemSet<'static>, 16>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Chain {
    None,
    All,
    Impossible,
}

impl const IntoSystemConfigs<()> for SystemConfigs {
    fn into_configs(self) -> SystemConfigs {
        self
    }
}

impl SystemConfigs {
    pub(crate) fn into_schedule(self) -> Schedule<'static> {
        let SystemConfigs {
            systems,
            chain: chained,
        } = self;

        let systems: Vec<System<'static>> = systems
            .into_slice()
            .iter()
            .map(|getter| (getter)())
            .collect();

        let mut constraints = Vec::new();
        if chained == Chain::All {
            // Chaining the systems [a, b, c] is equivalent to the constaints "a before b" + "b before c"
            for systems in systems.windows(2) {
                constraints.push(Constraint::Order {
                    before: SystemSet::Anonymous(vec![systems[0].id]),
                    after: SystemSet::Anonymous(vec![systems[1].id]),
                });
            }
        }

        common::Schedule {
            systems,
            constraints,
        }
    }
}

impl<Marker, F> const IntoSystemConfigs<Marker> for F
where
    F: 'static + Sized + IntoSystem<(), (), Marker> + Copy,
{
    fn into_configs(self) -> SystemConfigs {
        SystemConfigs {
            systems: ConstVec::from_slice(&[F::into_metadata]),
            chain: Chain::None,
        }
    }
}

#[doc(hidden)]
pub struct DescriptorTupleMarker;

macro_rules! impl_system_collection {
    ($(($param: ident, $sys: ident)),*) => {
        /// Implement IntoSystemConfigs for all possible sets
        impl<$($param, $sys),*> const IntoSystemConfigs<(DescriptorTupleMarker, $($param,)*)> for ($($sys,)*)
        where
            $($sys: ~const IntoSystemConfigs<$param> + Copy),*
        {
            #[allow(non_snake_case)]
            fn into_configs(self) -> SystemConfigs {
                let ($($sys,)*) = self;
                let mut systems = ConstVec::new();
                $(
                    let add = $sys.into_configs();
                    systems.extend(add.systems);
                )*
                SystemConfigs{
                    systems,
                    chain: Chain::None,
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

    #[test]
    fn into_config() {
        fn system1() {}
        fn system2(mut _commands: Commands) {}
        let system_set = (system1, system2);

        let system_config = system_set.into_configs();
        assert_eq!(system_config.chain, Chain::None);

        let Schedule {
            systems,
            constraints,
        } = system_config.into_schedule();
        assert_eq!(
            systems,
            [
                make_system(system1, vec![]),
                make_system(system2, vec![Param::Command]),
            ]
        );
        assert_eq!(constraints, []);
    }

    #[test]
    fn chaining() {
        fn system1() {}
        fn system2() {}
        fn system3() {}
        let system_set = (system1, system2, system3).chain();

        let system_config = system_set.into_configs();
        assert_eq!(system_config.chain, Chain::All);

        let Schedule { constraints, .. } = system_config.into_schedule();
        assert_eq!(
            constraints,
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
