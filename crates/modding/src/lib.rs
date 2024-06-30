pub mod ecs;
pub mod init;

pub mod prelude {
    pub use crate::init::{
        Commands, Component, Feature, Harmony, NewFeature, StableId, Start, Update,
    };
}
