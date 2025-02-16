use std::{
    cell::OnceCell,
    ops::{Deref, DerefMut},
};

use common::StableId;

use crate::{
    ecs::{
        system::{system_param::Params, SystemParam},
        Resource,
    },
    runtime::{
        deserialize, ffi_get_local_type_id, ffi_get_resource, ffi_set_resource, serialize,
        LocalTypeId,
    },
};

pub struct ResMut<'w, T>
where
    T: Resource,
{
    type_id: &'w LocalTypeId,
    changed: bool,
    value: OnceCell<T>,
}

impl<'a, T> SystemParam for ResMut<'a, T>
where
    T: Resource,
{
    type State = LocalTypeId;
    type Item<'state> = ResMut<'state, T>;

    fn init_state() -> Self::State {
        let id = StableId::from_typed::<T>();
        ffi_get_local_type_id(&id)
    }

    fn get_param<'state>(state: &'state mut Self::State) -> Self::Item<'state> {
        ResMut {
            type_id: state,
            changed: false,
            value: OnceCell::new(),
        }
    }

    fn get_metadata() -> Params {
        vec![common::Param::Res {
            mutable: false,
            id: StableId::from_typed::<T>(),
        }]
    }
}

impl<'w, T> Deref for ResMut<'w, T>
where
    T: Resource,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value.get_or_init(|| {
            let bytes = ffi_get_resource(self.type_id);
            deserialize(&bytes)
        })
    }
}

impl<'w, T> AsRef<T> for ResMut<'w, T>
where
    T: Resource,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'w, T> DerefMut for ResMut<'w, T>
where
    T: Resource,
{
    #[inline]
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        self.value.get_or_init(|| {
            let bytes = ffi_get_resource(self.type_id);
            deserialize(&bytes)
        });
        self.value.get_mut().unwrap()
    }
}

impl<'w, T> AsMut<T> for ResMut<'w, T>
where
    T: Resource,
{
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'w, T> Drop for ResMut<'w, T>
where
    T: Resource,
{
    fn drop(&mut self) {
        if self.changed {
            let value = self.value.get().unwrap();
            let buffer = serialize(value);
            ffi_set_resource(self.type_id, &buffer);
        }
    }
}
