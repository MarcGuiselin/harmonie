use crate::HasStableId;

pub trait ScheduleLabel
where
    Self: HasStableId,
{
}

pub struct Start;
impl HasStableId for Start {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Start";
}
impl ScheduleLabel for Start {}

pub struct Update;
impl HasStableId for Update {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Update";
}
impl ScheduleLabel for Update {}
