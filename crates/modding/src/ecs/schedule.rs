use super::StableId;

pub trait ScheduleLabel
where
    Self: StableId,
{
}

pub struct Start;
impl StableId for Start {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Start";
}
impl ScheduleLabel for Start {}

pub struct Update;
impl StableId for Update {
    const CRATE_NAME: &'static str = "core";
    const VERSION: &'static str = "v0.0.0";
    const NAME: &'static str = "Update";
}
impl ScheduleLabel for Update {}
