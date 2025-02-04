use async_fs;
use futures_lite::stream::StreamExt;
use rancor::ResultExt;
use std::path::{Path, PathBuf};

pub async fn read_dir<P, E>(path: P) -> Result<async_fs::ReadDir, E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let path = path.as_ref();
    async_fs::read_dir(path)
        .await
        .into_with_trace(|| format!("Failed to read dir: {:?}", path))
}

pub async fn remove_dir_all<P, E>(path: P) -> Result<(), E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let path = path.as_ref();
    async_fs::remove_dir_all(path)
        .await
        .into_with_trace(|| format!("Failed to remove dir: {:?}", path))
}

pub async fn remove_file<P, E>(path: P) -> Result<(), E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let path = path.as_ref();
    async_fs::remove_file(path)
        .await
        .into_with_trace(|| format!("Failed to remove file: {:?}", path))
}

pub async fn create_dir_all<P, E>(path: P) -> Result<(), E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let path = path.as_ref();
    match async_fs::create_dir_all(path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e).into_with_trace(|| format!("Failed to create dir: {:?}", path))
            }
        }
    }
}

pub async fn rename<P, Q, E>(from: P, to: Q) -> Result<(), E>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    E: rancor::Source,
{
    let from = from.as_ref();
    let to = to.as_ref();
    async_fs::rename(from, to)
        .await
        .into_with_trace(|| format!("Failed to rename file: {:?} -> {:?}", from, to))
}

pub async fn read<P, E>(path: P) -> Result<Vec<u8>, E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let path = path.as_ref();
    async_fs::read(path)
        .await
        .into_with_trace(|| format!("Failed to read file: {:?}", path))
}

pub async fn write<P, C, E>(path: P, contents: C) -> Result<(), E>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
    E: rancor::Source,
{
    let path = path.as_ref();
    async_fs::write(path, contents)
        .await
        .into_with_trace(|| format!("Failed to write to file: {:?}", path))
}

pub async fn empty_dir<P, E>(path: P) -> Result<(), E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let mut entries = read_dir(path).await?;

    while let Some(entry) = entries.try_next().await.into_error()? {
        let path = entry.path();
        if entry.file_type().await.into_error()?.is_dir() {
            remove_dir_all(path).await?;
        } else {
            remove_file(path).await?;
        }
    }
    Ok(())
}

/// Iterates through a directory's descendents, deleting those for whom the condition yields true
pub async fn empty_dir_conditional<P, C, E>(path: P, condition: C) -> Result<(), E>
where
    P: AsRef<Path>,
    C: Fn(&Path) -> bool,
    E: rancor::Source,
{
    let mut entries = read_dir(path).await?;

    while let Some(entry) = entries.try_next().await.into_error()? {
        let path = entry.path();
        if condition(&path) {
            if entry.file_type().await.into_error()?.is_dir() {
                remove_dir_all(path).await?;
            } else {
                remove_file(path).await?;
            }
        }
    }
    Ok(())
}

pub async fn create_dir_all_empty<P, E>(path: P) -> Result<(), E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let path = path.as_ref();
    create_dir_all(path).await?;
    empty_dir(path).await?;
    Ok(())
}

pub async fn list_files_in_dir<P, E>(path: P) -> Result<Vec<PathBuf>, E>
where
    P: AsRef<Path>,
    E: rancor::Source,
{
    let mut files = Vec::new();
    let mut dirs = vec![path.as_ref().to_path_buf()];

    while let Some(dir) = dirs.pop() {
        let mut entries = read_dir(&dir).await?;
        while let Some(entry) = entries.try_next().await.into_error()? {
            let path = entry.path();
            if entry.file_type().await.into_error()?.is_dir() {
                dirs.push(path);
            } else {
                files.push(path);
            }
        }
    }

    Ok(files)
}
