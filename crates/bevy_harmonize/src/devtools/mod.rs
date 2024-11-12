use std::path::{Path, PathBuf};

use async_channel::Receiver;
use bevy_app::{App, Plugin, PostUpdate, PreStartup};
use bevy_ecs::system::ResMut;
use bevy_ecs_macros::Resource;
use bevy_harmonize_build::build;
use bevy_tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_utils::tracing::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::mods::Mods;

const MOD_DIR: &str = "./mods";
const CARGO_DIR: &str = ".";

pub(crate) struct DevtoolsPlugin;

impl Plugin for DevtoolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildTask>()
            .add_systems(PreStartup, update_build)
            .add_systems(PostUpdate, update_build);
    }
}

#[derive(Resource)]
struct BuildTask {
    compute: Option<Task<Result<Vec<PathBuf>, rancor::Error>>>,

    /// Indicates that there were one or more file changes
    trigger_build: Receiver<()>,

    _watcher: RecommendedWatcher,
}

impl Default for BuildTask {
    fn default() -> Self {
        let (sender, receiver) = async_channel::bounded(1);

        // Rebuild at least once on startup
        sender.try_send(()).unwrap();

        let mut watcher = RecommendedWatcher::new(
            move |_| {
                let _ = sender.try_send(());
            },
            Default::default(),
        )
        .expect("Failed to create filesystem watcher.");

        let path = Path::new(MOD_DIR);
        watcher
            .watch(path, RecursiveMode::Recursive)
            .expect("Failed to watch path");

        Self {
            compute: None,
            trigger_build: receiver,
            _watcher: watcher,
        }
    }
}

fn update_build(mut task: ResMut<BuildTask>, mut mods: ResMut<Mods>) {
    // Check on the active build task
    if let Some(compute) = &mut task.compute {
        match block_on(poll_once(compute)) {
            Some(Ok(files)) => {
                for file in files {
                    mods.load_from_path(&file);
                }
            }
            Some(Err(err)) => error!("Error when building mods {}", err),
            None => {}
        }
        task.compute = None;
    }

    // Initialize a new task when the previous one is finished
    if task.trigger_build.try_recv().is_ok() && task.compute.is_none() {
        let debug = cfg!(debug_assertions);
        let mods_directory = Path::new(MOD_DIR).to_path_buf();
        let cargo_directory = Path::new(CARGO_DIR).to_path_buf();

        let future = build::<rancor::Error>(!debug, mods_directory, cargo_directory);
        task.compute
            .replace(AsyncComputeTaskPool::get().spawn(future));
    }
}
