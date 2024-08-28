use std::{
    io,
    path::{Path, PathBuf},
};

use bevy_utils::tracing::info;
use harmony_modloader_api as api;
use sha2::{Digest, Sha256};

use super::{
    schedule::{LoadedSchedules, SchedulingError},
    Feature,
};

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub struct LoadedMod {
    pub(super) manifest_hash: api::FileHash,
    module: wasmer::Module,
    features: Vec<Feature>,
    schedules: LoadedSchedules,
}

impl PartialEq for LoadedMod {
    fn eq(&self, other: &Self) -> bool {
        self.manifest_hash == other.manifest_hash
    }
}

impl LoadedMod {
    /// Load a mod from a path. The path can be either:
    /// - a directory containing ".wasm" and ".manifest" files
    /// - any mod file as long as it has siblings with matching names
    pub async fn try_from_path<P>(path: P) -> LoadedModResult
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        info!("Loading mod from path: {:?}", path);

        // Either files are like this: "modname/.wasm" or "modname.wasm"
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_owned()
            .into_string()
            .unwrap();

        let directory = if file_name.is_empty() || file_name.starts_with(".") {
            path
        } else {
            path.parent()
                .ok_or(LoadingError::FileNotFound(path.to_owned(), None))?
        };

        let package_name = file_name.split('.').next().unwrap().to_owned();

        let manifest_path = directory.join(format!("{}.manifest", package_name));
        let manifest_bytes = async_fs::read(&manifest_path)
            .await
            .map_err(|err| LoadingError::FileNotFound(manifest_path, Some(err)))?;

        let wasm_path = directory.join(format!("{}.wasm", package_name));
        let wasm_bytes = async_fs::read(&wasm_path)
            .await
            .map_err(|err| LoadingError::FileNotFound(wasm_path, Some(err)))?;

        Self::try_from_bytes(manifest_bytes, wasm_bytes).await
    }

    async fn try_from_bytes(manifest_bytes: Vec<u8>, wasm_bytes: Vec<u8>) -> LoadedModResult {
        let manifest: api::ModManifest =
            bitcode::decode(&manifest_bytes).map_err(|_| LoadingError::InvalidManifest)?;

        let wasm_hash = api::FileHash::from_sha256(Sha256::digest(&wasm_bytes).into());
        if wasm_hash != manifest.wasm_hash {
            return Err(LoadingError::MissmatchingDependencies);
        }

        let manifest_hash = api::FileHash::from_sha256(Sha256::digest(&manifest_bytes).into());

        let store = wasmer::Store::default();
        let module = wasmer::Module::new(&store, wasm_bytes).map_err(LoadingError::InvalidWasm)?;

        let features = manifest
            .features
            .iter()
            .map(Feature::from_descriptor)
            .collect();

        let mut schedules = LoadedSchedules::new();
        for (feature_id, feature) in manifest.features.iter().enumerate() {
            for descriptor in &feature.descriptors {
                schedules.add_from_descriptor(feature_id, descriptor)?;
            }
        }

        Ok(Self {
            manifest_hash,
            module,
            features,
            schedules,
        })
    }
}

pub type LoadedModResult = Result<LoadedMod, LoadingError>;

// These fields are read by a debug macro
#[allow(dead_code)]
#[derive(Debug)]
pub enum LoadingError {
    FileNotFound(PathBuf, Option<io::Error>),
    InvalidManifest,
    InvalidWasm(wasmer::CompileError),
    MissmatchingDependencies,
    InvalidSchedule(api::OwnedStableId),
    SchedulingError(SchedulingError),
}
