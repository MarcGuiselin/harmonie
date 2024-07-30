use bevy::{app::ScheduleRunnerPlugin, prelude::*, tasks::AsyncComputeTaskPool};
use std::{io::Result, time::Duration};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1))),
            harmony_modloader::ModloaderPlugin,
        ))
        .add_systems(Startup, build_mods)
        .run();
}

fn build_mods() {
    async fn build() -> Result<()> {
        let debug = cfg!(debug_assertions);
        let directory = std::env::current_dir()?;
        let packages = vec!["the_cube".into()];
        harmony_modloader_build::build(!debug, directory, packages).await?;

        Ok(())
    }

    AsyncComputeTaskPool::get()
        .spawn(async {
            build().await.expect("Failed to build mods");
            println!("Mods built successfully!");
        })
        .detach();
}
