use std::path::Path;

use bevy_utils::tracing::warn;

pub struct LoadingMod {
    // TODO
}

impl LoadingMod {
    pub async fn try_from_path<P>(path: P) -> LoadingModResult
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_owned();
        let bytes = async_fs::read(&path)
            .await
            .map_err(|_| LoadingModError::FileNotFound)?;

        warn!("Load mod: {:?}\n   Bytes: {}", path, bytes.len());

        Ok(Self {})
    }
}

pub type LoadingModResult = Result<LoadingMod, LoadingModError>;

pub enum LoadingModError {
    FileNotFound,
}
