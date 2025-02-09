#![allow(dead_code)] // TODO: remove
#![feature(const_type_id)]
#![feature(const_type_name)]

#[cfg(all(feature = "generate_manifest", feature = "wasm_runtime"))]
compile_error!(
    "Features \"generate_manifest\" and \"wasm_runtime\" cannot be enabled at the same time"
);

pub mod ecs;
pub mod schema;
pub(crate) mod utils;

pub mod prelude {
    pub use bevy_reflect::prelude::*;
    pub use bevy_reflect_derive::*;
    pub use bitcode::{Decode, Encode};

    pub use crate::ecs::{system::Commands, Component, Resource};
    pub use crate::schema::{Mod, Schema};

    // Schedules
    pub use common::{Start, Update};
}
