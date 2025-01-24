use bevy_app::{App, Plugin};

mod devtools;

mod mods;

pub struct ModloaderPlugin;

impl Plugin for ModloaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((mods::ModPlugin, devtools::DevtoolsPlugin));
    }
}

pub mod prelude {
    pub use crate::ModloaderPlugin;
}
