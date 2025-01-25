use const_vec::ConstVec;

use bevy_reflect::{TypeInfo, Typed};
use common::StableIdGetter;

use crate::ecs::Resource;

#[derive(Debug)]
pub struct Schema {
    types: ConstVec<fn() -> &'static TypeInfo, 1024>,
    resources: ConstVec<(StableIdGetter, fn() -> Vec<u8>), 128>,
}

impl Schema {
    pub const fn new() -> Self {
        Self {
            types: ConstVec::new(),
            resources: ConstVec::new(),
        }
    }

    pub const fn register_type<T>(mut self) -> Self
    where
        T: Typed,
    {
        self.types.push(T::type_info);
        self
    }

    pub const fn add_resource<R>(mut self) -> Self
    where
        R: Resource,
    {
        self.resources
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
    fn schema_check_size() {
        // Assume any size over 1MB is too big
        assert_eq!(size_of::<Schema>() < 1024 * 1024, true);
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

        const SCHEMA: Schema = Schema::new().add_resource::<TestResource>();

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

        const SCHEMA: Schema = Schema::new().register_type::<TestType>();

        let Schema { types, .. } = SCHEMA;
        assert_eq!(types.len(), 1);
        let test_type = types[0]();
        assert_eq!(test_type.type_id(), std::any::TypeId::of::<TestType>());
    }
}
