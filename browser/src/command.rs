use std::{path::Path, str};
use tokio::process::Command;

pub struct CargoCommand {
    pub command: Command,
}

impl CargoCommand {
    pub fn new(kind: &str) -> Self {
        let mut command = Command::new("cargo");
        command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."));
        command.arg(kind);
        Self { command }
    }

    pub fn packages(&mut self, names: &[&str]) -> &mut Self {
        self.command
            .args(names.iter().flat_map(|package_name| ["-p", package_name]));
        self
    }
}
