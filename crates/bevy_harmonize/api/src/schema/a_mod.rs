use bevy_reflect::Typed;

use crate::ecs::Resource;

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
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_reflect::Reflect;
    use bitcode::Encode;
    use common::HasStableId;

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

        let Schema { resources, .. } = SCHEMA;
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
}
