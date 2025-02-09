use bitcode::{Decode, Encode};

use crate::StableId;

/// A serializable version of [`TypeInfo`]
#[derive(Encode, Decode, PartialEq, Debug)]
pub enum TypeSignature<'a> {
    Struct {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        fields: Vec<FieldSignature<'a>>,
    },
    TupleStruct {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        fields: Vec<StableId<'a>>,
    },
    Tuple {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        fields: Vec<StableId<'a>>,
    },
    List {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        item_ty: StableId<'a>,
    },
    Array {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        item_ty: StableId<'a>,
        capacity: usize,
    },
    Map {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        key_ty: StableId<'a>,
        value_ty: StableId<'a>,
    },
    Set {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        value_ty: StableId<'a>,
    },
    Enum {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
        variants: Vec<VariantSignature<'a>>,
    },
    Opaque {
        ty: StableId<'a>,
        generics: Vec<GenericSignature<'a>>,
    },
}

/// A serializable version of [`bevy_reflect::GenericInfo`]
#[derive(Encode, Decode, PartialEq, Debug)]
pub enum GenericSignature<'a> {
    Type(StableId<'a>),
    Const(StableId<'a>),
}

/// A serializable version of [`bevy_reflect::NamedField`]
#[derive(Encode, Decode, PartialEq, Debug)]
pub struct FieldSignature<'a> {
    pub name: &'a str,
    pub ty: StableId<'a>,
}

/// A serializable version of [`bevy_reflect::VariantInfo`]
#[derive(Encode, Decode, PartialEq, Debug)]
pub enum VariantSignature<'a> {
    Struct {
        name: &'a str,
        fields: Vec<FieldSignature<'a>>,
    },
    Tuple {
        name: &'a str,
        fields: Vec<StableId<'a>>,
    },
    Unit {
        name: &'a str,
    },
}
