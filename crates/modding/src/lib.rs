#![allow(dead_code)] // TODO: remove

pub mod ecs;
pub mod init;
pub(crate) mod utils;

pub mod prelude {
    pub use crate::ecs::{system::Commands, Component};
    pub use crate::init::{Feature, FeatureBuilder, Harmony};
    pub use harmony_modloader_api::{HasStableId, Start, Update};
}
