use async_fs::{self, DirEntry, ReadDir};
use bevy_utils::{info, tracing::info};
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

async fn aaa<E>(entries:&mut  ReadDir) -> Result<Option<DirEntry>, E> where 
E: rancor::Source {
    let a = entries.try_next();
    info!("Stuck");
    let a = a.await;
    info!("Not stuck");
    let a = a.into_error();
    a
}

pub async fn copy_dir<P, Q, E>(from: P, to: Q) -> Result<(), E>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    E: rancor::Source,
{
    let from = from.as_ref();
    let to = to.as_ref();

    create_dir_all_empty(to).await?;

    let mut dirs = vec![from.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        //info!("dir: {:?}, dirs: {:?}", &dir, &dirs);

        let mut entries = read_dir(&dir).await?;

        while let Some(entry) = aaa(&mut entries).await? {
            //info!("aaaa");
            let from_path = entry.path();
            let to_path = to.join(from_path.strip_prefix(from).unwrap());
            info!("{:?} -> {:?}", &from_path, &to_path);
            if entry.file_type().await.into_error()?.is_dir() {
                //info!("dddd");
                async_fs::create_dir(&to_path).await.into_error()?;
                dirs.push(from_path);
            } else {
                //info!("eeee");
                async_fs::copy(&from_path, &to_path)
                    .await
                    .into_with_trace(|| {
                        format!("Failed to copy file: {:?} -> {:?}", from_path, to_path)
                    })?;
            }
            //info!("bbbb");
        }
        //info!("cccc");
    }

    Ok(())
}
