use async_channel::Receiver;
use bevy_app::{App, Plugin, PostUpdate, PreStartup};
use bevy_ecs::prelude::*;
use bevy_ecs_macros::Resource;
use bevy_utils::{
    default,
    tracing::{error, warn},
};
use futures_lite::future::block_on;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{fs, path::Path};

use crate::mods::Mods;

const MOD_BUILD_DIRS: &[&str] = &[
    "./target/harmony-build/debug",
    "./target/harmony-build/release",
];

pub(crate) struct FileWatcherPlugin;

impl Plugin for FileWatcherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileWatcher>()
            .add_systems(PreStartup, load_all_mods)
            .add_systems(PostUpdate, update_mods);
    }
}

#[derive(Resource)]
struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event>>,
}

impl Default for FileWatcher {
    fn default() -> Self {
        let mut ret = Self::new();

        for path in MOD_BUILD_DIRS {
            let path = Path::new(path);
            let _ = fs::create_dir_all(path);
            ret.watch(path).expect("Failed to watch path");
        }

        ret
    }
}

impl FileWatcher {
    fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        let watcher = RecommendedWatcher::new(
            move |res| {
                block_on(sender.send(res)).expect("Watch event send failure.");
            },
            default(),
        )
        .expect("Failed to create filesystem watcher.");

        Self { watcher, receiver }
    }

    /// Watch for changes recursively at the provided path.
    fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.watcher
            .watch(path.as_ref(), RecursiveMode::NonRecursive)
    }
}

fn load_all_mods(mut mods: ResMut<Mods>) {
    for path in MOD_BUILD_DIRS {
        let _ = fs::create_dir_all(&path);
        let paths = fs::read_dir(path).unwrap();
        for path in paths {
            let path = path.unwrap().path();

            match path.extension() {
                Some(ext) if ext == "wasm" => {
                    mods.load_from_path(path);
                }
                _ => {}
            }
        }
    }
}

fn update_mods(mut mods: ResMut<Mods>, file_watcher: Res<FileWatcher>) {
    while let Ok(event) = file_watcher.receiver.try_recv() {
        match event {
            Ok(Event { kind, paths, .. }) => {
                for path in paths {
                    // We only load wasm files for now
                    match path.extension() {
                        Some(ext) if ext == "wasm" => match kind {
                            EventKind::Create(_) => mods.load_from_path(path),
                            _ => warn!(
                                "Ignored event: {:?} {:?}",
                                path.file_name().unwrap_or_default(),
                                kind
                            ),
                        },
                        _ => {}
                    }
                }
            }
            Err(err) => error!("file watcher error: {:?}", err),
        }
    }
}
