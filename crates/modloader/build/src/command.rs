use async_process::Command;
use std::{ffi::OsStr, path::Path, str};

pub struct CargoCommand {
    pub inner: Command,
}

impl CargoCommand {
    pub fn new(kind: &str) -> Self {
        let mut command = Command::new("cargo");
        command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."));
        command.arg(kind);
        Self { inner: command }
    }

    pub fn packages<I, S>(&mut self, names: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for name in names {
            self.inner.arg("-p");
            self.inner.arg(name.as_ref());
        }
        self
    }
}
