use std::marker::PhantomData;

use common::{Constraint, SystemSet};

use super::IntoSchedule;

#[derive(Clone, Copy)]
pub struct Chained<Marker, T>(PhantomData<(Marker, T)>);

#[inline]
#[track_caller]
pub(crate) const fn new<Marker, T>() -> Chained<Marker, T>
where
    T: IntoSchedule<Marker>,
{
    if !T::IS_CHAINABLE {
        panic!("It is not possible to chain these systems")
    }

    Chained(PhantomData)
}

impl<Marker, T> IntoSchedule<()> for Chained<Marker, T>
where
    T: IntoSchedule<Marker> + Copy,
{
    fn into_configs() -> common::Schedule<'static> {
        let mut schedule = T::into_configs();

        for systems in schedule.systems.windows(2) {
            schedule.constraints.push(Constraint::Order {
                before: SystemSet::Anonymous(vec![systems[0].id]),
                after: SystemSet::Anonymous(vec![systems[1].id]),
            });
        }

        schedule
    }
}
