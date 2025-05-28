use crate::nuop::reconciler::config::Config;
use anyhow::{Context, Result};
use std::{path::PathBuf, process::Command};
use tracing::debug;

pub fn get_script_config(script: &PathBuf) -> Result<Config> {
    let output = Command::new(script)
        .arg("config")
        .output()
        .with_context(|| format!("Failed to execute script: {:?}", script))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Script execution failed with status: {}",
            output.status
        ));
    }

    let config_str =
        String::from_utf8(output.stdout).context("Failed to parse script output as UTF-8")?;

    debug!("Config: {:?}", config_str);

    let config: Config = serde_yaml::from_str(&config_str)
        .context("Failed to deserialize script output into Config")?;

    Ok(config)
}
