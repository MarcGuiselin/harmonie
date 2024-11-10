use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(
                1000 / 60,
            ))),
            bevy_harmonize::ModloaderPlugin,
            LogPlugin::default(),
        ))
        .run();
}
