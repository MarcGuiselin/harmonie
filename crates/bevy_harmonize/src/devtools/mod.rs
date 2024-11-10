use std::{
    io::Result,
    path::{Path, PathBuf},
};

use async_channel::Receiver;
use bevy_app::{App, Plugin, PostUpdate, PreStartup};
use bevy_ecs::system::ResMut;
use bevy_ecs_macros::Resource;
use bevy_tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy_utils::tracing::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::mods::Mods;

const PATH: &str = "C:/Users/Marc/Documents/Projects/Harmony/harmony/mods";

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
    compute: Option<Task<Result<BuildResult>>>,

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

        let path = Path::new(PATH);
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

struct BuildResult {
    paths: Vec<PathBuf>,
}

fn update_build(mut task: ResMut<BuildTask>, mut mods: ResMut<Mods>) {
    // Check on the active build task
    if let Some(compute) = &mut task.compute {
        match block_on(poll_once(compute)) {
            Some(Ok(BuildResult { paths })) => {
                for path in paths {
                    mods.load_from_path(path);
                }
            }
            Some(Err(err)) => error!("Error when building mods {}", err),
            None => {}
        }
        task.compute = None;
    }

    // Initialize a new task when the previous one is finished
    if task.trigger_build.try_recv().is_ok() && task.compute.is_none() {
        task.compute
            .replace(AsyncComputeTaskPool::get().spawn(build_all()));
    }
}

async fn build_all() -> Result<BuildResult> {
    warn!("Rebuilding!");

    Ok(BuildResult { paths: Vec::new() })
}
