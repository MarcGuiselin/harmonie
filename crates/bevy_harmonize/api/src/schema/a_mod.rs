use bevy_reflect::Typed;
use common::ScheduleLabel;

use crate::ecs::{system::IntoSystemConfigs, Resource};

use super::Schema;

#[derive(Debug, Clone, Copy)]
pub struct Mod {
    schema: Schema,
}

impl Mod {
    pub const fn new(name: &'static str) -> Self {
        let mut schema = Schema::new();
        schema.name = Some(name);
        Self { schema }
    }

    pub const fn into_schema(self) -> Schema {
        self.schema
    }

    pub const fn register_type<T>(&mut self) -> &mut Self
    where
        T: Typed,
    {
        self.schema.types.push(T::type_info);
        self
    }

    pub const fn add_resource<R>(&mut self) -> &mut Self
    where
        R: Resource,
    {
        self.schema
            .resources
            .push((R::get_stable_id, R::default_value_as_buffer));
        self
    }

    pub const fn add_systems<Marker>(
        &mut self,
        schedule: impl ~const ScheduleLabel,
        systems: impl ~const IntoSystemConfigs<Marker>,
    ) -> &mut Self {
        const fn schedule_id_getter_from_value<T>(_schedule: T) -> fn() -> common::StableId<'static>
        where
            T: ScheduleLabel,
        {
            T::get_stable_id
        }
        let getter = schedule_id_getter_from_value(schedule);
        let system_configs = IntoSystemConfigs::into_configs(systems);
        self.schema.schedules.push((getter, system_configs));
        self
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_reflect::Reflect;
    use bitcode::Encode;
    use common::{HasStableId, Start, Update};

    #[test]
    fn name() {
        const SCHEMA: Schema = Mod::new("A custom name").into_schema();
        assert_eq!(SCHEMA.name, Some("A custom name"));
    }

    #[test]
    fn add_resource() {
        #[derive(Debug, Reflect, Encode)]
        struct TestResource(u32);
        impl Default for TestResource {
            fn default() -> Self {
                Self(123)
            }
        }
        impl Resource for TestResource {}
        impl HasStableId for TestResource {
            const CRATE_NAME: &'static str = "";
            const VERSION: &'static str = "";
            const NAME: &'static str = "TestResource";
        }

        const SCHEMA: Schema = Mod::new("Test add_resource")
            .add_resource::<TestResource>()
            .into_schema();

        let Schema {
            types, resources, ..
        } = SCHEMA;

        // TODO: test registering type
        assert_eq!(types.len(), 0);

        assert_eq!(resources.len(), 1);
        let (stable_id, default_value) = resources[0];
        assert_eq!(stable_id().name, "TestResource");
        assert_eq!(default_value(), vec![4, 123]);
    }

    #[test]
    fn register_type() {
        #[derive(Debug, Reflect)]
        struct TestType;

        const SCHEMA: Schema = Mod::new("Test register_type")
            .register_type::<TestType>()
            .into_schema();

        let Schema { types, .. } = SCHEMA;
        assert_eq!(types.len(), 1);
        let test_type = types[0]();
        assert_eq!(test_type.type_id(), std::any::TypeId::of::<TestType>());
    }

    #[test]
    fn add_systems() {
        fn system1() {}
        fn system2() {}
        fn system3() {}

        const SCHEMA: Schema = Mod::new("Test register_type")
            .add_systems(Start, system1)
            .add_systems(Update, (system1, system3).chain())
            .into_schema();

        let Schema {
            types, schedules, ..
        } = SCHEMA;

        // TODO: test registering nested types
        assert_eq!(types.len(), 0);

        assert_eq!(schedules.len(), 2);
        assert_eq!(schedules[0].0(), Start::get_stable_id());
        assert_eq!(schedules[0].1.into_schedule().systems.len(), 1);
        assert_eq!(schedules[1].0(), Update::get_stable_id());
        assert_eq!(schedules[1].1.into_schedule().systems.len(), 2);
    }
}
