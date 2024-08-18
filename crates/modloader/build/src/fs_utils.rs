use async_fs;
use futures_lite::stream::StreamExt;
use std::{io::Result, path::Path};

pub async fn remove_dir_contents<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut dir_entries = async_fs::read_dir(path).await?;

    while let Some(entry) = dir_entries.next().await {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type().await?.is_dir() {
            async_fs::remove_dir_all(path).await?;
        } else {
            async_fs::remove_file(path).await?;
        }
    }
    Ok(())
}
