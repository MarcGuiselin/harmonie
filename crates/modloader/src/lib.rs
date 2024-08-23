use bevy_app::{App, Plugin};

mod file_watcher;

mod mods;

mod schedule;

pub struct ModloaderPlugin;

impl Plugin for ModloaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((mods::ModPlugin, file_watcher::FileWatcherPlugin));
    }
}
