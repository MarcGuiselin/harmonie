use std::path::Path;

use bevy_utils::tracing::warn;

pub struct LoadedMod {
    // TODO
}

impl LoadedMod {
    pub async fn try_from_path<P>(path: P) -> LoadedModResult
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_owned();
        let bytes = async_fs::read(&path)
            .await
            .map_err(|_| LoadingError::FileNotFound)?;

        warn!("Load mod: {:?}\n   Bytes: {}", path, bytes.len());

        Ok(Self {})
    }
}

pub type LoadedModResult = Result<LoadedMod, LoadingError>;

pub enum LoadingError {
    FileNotFound,
}
