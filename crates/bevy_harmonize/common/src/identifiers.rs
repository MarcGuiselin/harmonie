use crate::HasStableId;

#[const_trait]
pub trait ScheduleLabel
where
    Self: ~const HasStableId + Copy,
{
}

#[derive(Clone, Copy)]
pub struct Start;
impl const HasStableId for Start {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Start";
}
impl const ScheduleLabel for Start {}

#[derive(Clone, Copy)]
pub struct Update;
impl const HasStableId for Update {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Update";
}
impl const ScheduleLabel for Update {}
