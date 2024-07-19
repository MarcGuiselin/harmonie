use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let debug = cfg!(debug_assertions);
    let directory = std::env::current_dir()?;
    let packages = vec!["the_cube".into()];
    harmony_modloader_build::build(!debug, directory, packages).await?;

    Ok(())
}
