mod function_system;
mod params;
mod system;
mod system_config;
mod system_param;

use std::any::TypeId;

pub use function_system::FunctionSystem;
pub use params::*;
pub use system::{BoxedSystem, System};
pub use system_config::IntoSystemConfigs;
pub use system_param::{Params, SystemParam};

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid system with input `{In}` and output `{Out}`",
    label = "invalid system"
)]
pub trait IntoSystem<In, Out, Marker>: Sized {
    /// The type of [`System`] that this instance converts into.
    type System: System<In = In, Out = Out>;

    /// Turns this value into its corresponding [`System`].
    fn into_system(self) -> Self::System;

    /// Get the [`TypeId`] of the [`System`] produced after calling [`into_system`](`IntoSystem::into_system`).
    #[inline]
    fn system_type_id(&self) -> TypeId {
        TypeId::of::<Self::System>()
    }
}

/// Wrapper type to mark a [`SystemParam`] as an input.
pub struct In<In>(pub In);

impl<T> std::ops::Deref for In<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for In<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
