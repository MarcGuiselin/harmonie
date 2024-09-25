use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*, tasks::AsyncComputeTaskPool};
use std::{io::Result, time::Duration};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1))),
            bevy_harmonize::ModloaderPlugin,
            LogPlugin::default(),
        ))
        .add_systems(Startup, build_mods)
        .run();
}

fn build_mods() {
    async fn build() -> Result<()> {
        let debug = cfg!(debug_assertions);
        let directory = std::env::current_dir()?;
        let packages = vec!["the_cube".into()];
        bevy_harmonize_build::build(!debug, directory, packages).await?;

        Ok(())
    }

    AsyncComputeTaskPool::get()
        .spawn(async {
            if let Err(e) = build().await {
                error!("Failed to build mods: {:?}", e);
            }
        })
        .detach();
}
