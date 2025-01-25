use const_vec::ConstVec;

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

    /// Runs the system with the given input
    fn run(&mut self, input: Self::In) -> Self::Out;

    /// List of [`common::ParamDescriptor`]s that this system has.
    fn param_descriptors(&self) -> ParamDescriptors;
}

pub type ParamDescriptors = ConstVec<common::Param<'static>, 64>;

/// A convenience type alias for a boxed [`System`] trait object.
pub type BoxedSystem<In = (), Out = ()> = Box<dyn System<In = In, Out = Out>>;
