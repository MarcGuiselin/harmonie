#![allow(dead_code)] // TODO: remove

pub mod ecs;
pub mod init;

pub mod prelude {
    pub use crate::ecs::{system::Commands, Component, StableId};
    pub use crate::init::{Feature, Harmony, NewFeature, Start, Update};
}
