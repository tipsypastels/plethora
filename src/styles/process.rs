use crate::{stuff::STUFF, themes::Theme};
use anyhow::{ensure, Result};
use camino::Utf8Path;
use std::process::Stdio;
use tokio::process::{Child, Command};

#[derive(Debug)]
pub struct Process {
    _child: Option<Child>,
}

impl Process {
    pub async fn new(binary: &Utf8Path, theme: &Theme) -> Result<Self> {
        let mut command = Command::new(binary.as_str());

        let input = theme.tailwind_input_path();
        let output = theme.tailwind_output_path();
        let config = theme.tailwind_config_path();

        command
            .kill_on_drop(true)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(["--config", config.as_str()])
            .args(["--input", input.as_str()])
            .args(["--output", output.as_str()])
            .arg("--minify");

        let _child = if STUFF.reload {
            command.arg("--watch");
            Some(command.spawn()?)
        } else {
            let status = command.status().await?;
            let success = status.success();

            ensure!(success, "tailwind {} failed, status {status}", theme.slug());
            None
        };

        Ok(Self { _child })
    }
}
