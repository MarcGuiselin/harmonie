use std::any::TypeId;

use super::Params;

#[diagnostic::on_unimplemented(message = "`{Self}` is not a system", label = "invalid system")]
pub trait System
where
    Self: Send + Sync + 'static,
{
    /// The system's input. See [`In`](crate::system::In) for
    /// [`FunctionSystem`](crate::system::FunctionSystem)s.
    type In;

    /// The system's output.
    type Out;

    /// Returns the system's name.
    fn name(&self) -> &'static str;

    /// Returns the [`TypeId`] of the underlying system type.
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Runs the system with the given input
    fn run(&mut self, input: Self::In) -> Self::Out;

    /// List of [`common::Param`]s that this system has.
    fn params(&self) -> Params;
}

/// A convenience type alias for a boxed [`System`] trait object.
pub type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;
