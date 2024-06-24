use std::any::Any;

use serde::de::value;

trait Signal {
    const NAME: &'static str;
}

#[derive(Clone)]
struct A;
impl Signal for A {
    const NAME: &'static str = "A";
}

#[derive(Clone)]
struct B;
impl Signal for B {
    const NAME: &'static str = "B";
}

#[derive(Clone)]
struct C;
impl Signal for C {
    const NAME: &'static str = "C";
}

#[derive(Clone)]
struct D;
impl Signal for D {
    const NAME: &'static str = "D";
}

fn produce_nothing_from_a(_a: A) {}

fn produce_c_from_a(_a: A) -> C {
    C
}

fn produce_c_and_d_from_a_and_b(_ab: (A, B)) -> (C, D) {
    (C, D)
}

struct AnySignal<'v> {
    name: &'static str,
    value: &'v dyn Any,
}

/// A trait implemented for all signals
pub trait SignalGroup<'v>
where
    Self: Sized + Clone + 'v,
{
    fn breakup(self) -> Vec<AnySignal<'v>>;
    fn combine(other: Vec<AnySignal>) -> Option<Self>;
    fn input_names() -> Vec<&'static str>;
}

impl<'v> SignalGroup<'v> for () {
    fn breakup(self) -> Vec<AnySignal<'v>> {
        vec![]
    }

    fn combine(other: Vec<AnySignal>) -> Option<Self> {
        if other.is_empty() {
            Some(())
        } else {
            None
        }
    }

    fn input_names() -> Vec<&'static str> {
        vec![]
    }
}

impl<'v, P0> SignalGroup<'v> for P0
where
    P0: Signal + Clone + 'v,
{
    fn breakup(self) -> Vec<AnySignal<'v>> {
        let value: &dyn Any = &self;
        vec![AnySignal {
            name: P0::NAME,
            value,
        }]
    }

    fn combine(other: Vec<AnySignal>) -> Option<Self> {
        if other.len() == 1 && other[0].name == P0::NAME {
            Some(other[0].value.downcast_ref::<P0>().unwrap().clone())
        } else {
            None
        }
    }

    fn input_names() -> Vec<&'static str> {
        vec![P0::NAME]
    }
}

impl<'v, P0, P1> SignalGroup<'v> for (P0, P1)
where
    P0: Signal + Clone + 'v,
    P1: Signal + Clone + 'v,
{
    fn breakup(self) -> Vec<AnySignal<'v>> {
        vec![
            AnySignal {
                name: P0::NAME,
                value: &self.0,
            },
            AnySignal {
                name: P1::NAME,
                value: &self.1,
            },
        ]
    }

    fn combine(other: Vec<AnySignal>) -> Option<Self> {
        if other.len() == 2 && other[0].name == P0::NAME && other[1].name == P1::NAME {
            Some((
                other[0].value.downcast_ref::<P0>().unwrap().clone(),
                other[1].value.downcast_ref::<P1>().unwrap().clone(),
            ))
        } else {
            None
        }
    }

    fn input_names() -> Vec<&'static str> {
        vec![P0::NAME, P1::NAME]
    }
}

// struct Params;
//
// /// A trait implemented for all functions that can be used as [`Transformer`]s.
// ///
// /// Basically copied from Bevy's `SystemParamFunction`
// pub trait TransformerFunction {
//     fn input_names(&self) -> Vec<&'static str>;
//     //fn run(&self, input: Vec<Params>) -> Vec<Params>;
// }
//
// impl<p1, p2> Transformer for fn(A) -> C {
//     fn input_names(&self) -> Vec<&'static str> {
//         vec![A::NAME]
//     }
// }

fn call_transformer<'v, Input, Output>(
    f: fn(Input) -> Output,
    input: Vec<AnySignal>,
) -> Vec<AnySignal<'v>>
where
    Input: SignalGroup<'v>,
    Output: SignalGroup<'v>,
{
    let input = Input::combine(input).unwrap();
    let output = f(input);
    output.breakup()
}

fn main() {
    let test1 = call_transformer(
        produce_nothing_from_a,
        vec![AnySignal {
            name: A::NAME,
            value: &A,
        }],
    );
}
