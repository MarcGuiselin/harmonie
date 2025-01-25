use super::{system_param::SystemParamItem, ConstParams, In, IntoSystem, System, SystemParam};
use bevy_utils_proc_macros::all_tuples;
use std::marker::PhantomData;

/// The [`System`] counter part of an ordinary function.
///
/// You get this by calling [`IntoSystem::into_system`]  on a function that only accepts
/// [`SystemParam`]s. The output of the system becomes the functions return type, while the input
/// becomes the functions [`In`] tagged parameter or `()` if no such parameter exists.
///
/// [`FunctionSystem`] must be `.initialized` before they can be run.
///
/// The [`Clone`] implementation for [`FunctionSystem`] returns a new instance which
/// is NOT initialized. The cloned system must also be `.initialized` before it can be run.
pub struct FunctionSystem<Marker, F>
where
    F: SystemParamFunction<Marker>,
{
    func: F,
    param_state: <F::Param as SystemParam>::State,
    params: ConstParams,
    name: &'static str,
    // NOTE: PhantomData<fn()-> T> gives this safe Send/Sync impls
    marker: PhantomData<fn() -> Marker>,
}

macro_rules! impl_system_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<Out, Func: Send + Sync + 'static, $($param: SystemParam),*> SystemParamFunction<fn($($param,)*) -> Out> for Func
        where
        for <'a> &'a mut Func:
                FnMut($($param),*) -> Out +
                FnMut($(SystemParamItem<$param>),*) -> Out, Out: 'static
        {
            type In = ();
            type Out = Out;
            type Param = ($($param,)*);
            #[inline]
            fn run(&mut self, _input: (), param_value: SystemParamItem<($($param,)*)>) -> Out {
                // Yes, this is strange, but `rustc` fails to compile this impl
                // without using this function. It fails to recognize that `func`
                // is a function, potentially because of the multiple impls of `FnMut`
                #[allow(clippy::too_many_arguments)]
                fn call_inner<Out, $($param,)*>(
                    mut f: impl FnMut($($param,)*)->Out,
                    $($param: $param,)*
                )->Out{
                    f($($param,)*)
                }
                let ($($param,)*) = param_value;
                call_inner(self, $($param),*)
            }
        }

        #[allow(non_snake_case)]
        impl<Input, Out, Func: Send + Sync + 'static, $($param: SystemParam),*> SystemParamFunction<fn(In<Input>, $($param,)*) -> Out> for Func
        where
        for <'a> &'a mut Func:
                FnMut(In<Input>, $($param),*) -> Out +
                FnMut(In<Input>, $(SystemParamItem<$param>),*) -> Out, Out: 'static
        {
            type In = Input;
            type Out = Out;
            type Param = ($($param,)*);
            #[inline]
            fn run(&mut self, input: Input, param_value: SystemParamItem< ($($param,)*)>) -> Out {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<Input, Out, $($param,)*>(
                    mut f: impl FnMut(In<Input>, $($param,)*)->Out,
                    input: In<Input>,
                    $($param: $param,)*
                )->Out{
                    f(input, $($param,)*)
                }
                let ($($param,)*) = param_value;
                call_inner(self, In(input), $($param),*)
            }
        }
    };
}

// Note that we rely on the highest impl to be <= the highest order of the tuple impls
// of `SystemParam` created.
all_tuples!(impl_system_function, 0, 0, F);

/// A marker type used to distinguish regular function systems from exclusive function systems.
#[doc(hidden)]
pub struct IsFunctionSystem;

impl<Marker, F> const IntoSystem<F::In, F::Out, (IsFunctionSystem, Marker)> for F
where
    Marker: 'static,
    F: SystemParamFunction<Marker>,
    <F as SystemParamFunction<Marker>>::Param: ~const SystemParam,
{
    type System = FunctionSystem<Marker, F>;
    fn into_system(func: Self) -> Self::System {
        FunctionSystem {
            func,
            param_state: F::Param::init_state(),
            params: F::Param::get_descriptors(),
            name: std::any::type_name::<F>(),
            marker: PhantomData,
        }
    }
}

impl<Marker, F> System for FunctionSystem<Marker, F>
where
    Marker: 'static,
    F: SystemParamFunction<Marker>,
{
    type In = F::In;
    type Out = F::Out;

    #[inline]
    fn name(&self) -> &'static str {
        self.name
    }

    #[inline]
    fn run(&mut self, input: Self::In) -> Self::Out {
        let params = F::Param::get_param(&mut self.param_state);
        let out = self.func.run(input, params);
        out
    }

    #[inline]
    fn params(&self) -> ConstParams {
        self.params.clone()
    }
}

/// A trait implemented for all functions that can be used as [`System`]s.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid system",
    label = "invalid system"
)]
#[const_trait]
pub trait SystemParamFunction<Marker>
where
    Self: Send + Sync + 'static,
{
    /// The input type to this system. See [`System::In`].
    type In;

    /// The return type of this system. See [`System::Out`].
    type Out;

    /// The [`SystemParam`]/s used by this system.
    type Param: SystemParam;

    /// Executes this system once. See [`System::run`]
    fn run(&mut self, input: Self::In, param_value: SystemParamItem<Self::Param>) -> Self::Out;
}
