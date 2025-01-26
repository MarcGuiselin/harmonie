mod function_system;
mod params;
mod system;
mod system_config;
mod system_param;

use std::any::TypeId;

pub use function_system::FunctionSystem;
pub use params::*;
pub use system::{BoxedSystem, ConstParams, System};
pub use system_config::IntoSystemConfigs;
pub use system_param::SystemParam;

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid system with input `{In}` and output `{Out}`",
    label = "invalid system"
)]
#[const_trait]
pub trait IntoSystem<In, Out, Marker>: Sized {
    /// The type of [`System`] that this instance converts into.
    type System: System<In = In, Out = Out>;

    /// Turns this value into its corresponding [`System`].
    fn into_system(this: Self) -> Self::System;

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

#[cfg(test)]
mod tests {
    use common::Param;

    use super::*;
    use crate::prelude::Commands;

    #[test]
    fn simple_system() {
        static mut RAN: bool = false;

        fn sys() {
            unsafe {
                RAN = true;
            }
        }

        let mut system = IntoSystem::into_system(sys);
        assert_eq!(
            system.name(),
            "bevy_harmonize_api::ecs::system::tests::simple_system::sys"
        );
        assert_eq!(system.params().into_slice(), &[]);

        system.run(());
        assert!(unsafe { RAN }, "system did not run");
    }

    #[test]
    fn system_with_param() {
        fn sys(mut _commands: Commands) {}

        let system = IntoSystem::into_system(sys);
        assert_eq!(system.params().into_slice(), vec![Param::Command]);
    }
}
