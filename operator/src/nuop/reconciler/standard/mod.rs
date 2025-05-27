use super::{controller::controller as reconciler_controller, util::get_script_config};
use kube::Client;
use std::{collections::HashSet, path::PathBuf};
use tokio::task::JoinHandle;
use tracing::error;

pub fn get_standard_controllers(client: &Client, scripts: &[PathBuf]) -> Vec<JoinHandle<()>> {
    let mut processed_kinds = HashSet::new();
    scripts
        .iter()
        .filter_map(|script| {
            get_script_config(script)
                .map_err(|e| {
                    error!("Failed to get script config for {:?}: {:?}", script, e);
                })
                .ok()
                .and_then(|config| {
                    if processed_kinds.insert(config.kind.clone()) {
                        Some((script, config))
                    } else {
                        error!(
                            "Duplicate kind found for script {:?}: {:?}",
                            &script, &config.kind
                        );
                        None
                    }
                })
        })
        .map(|(script, config)| {
            tokio::spawn(reconciler_controller(
                client.clone(),
                config,
                script.clone(),
            ))
        })
        .collect()
}
