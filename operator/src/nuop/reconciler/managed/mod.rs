use super::controller::controller as reconciler_controller;
use crate::nuop::manager::Mapping;
use crate::nuop::reconciler::util::get_script_config;
use kube::Client;
use std::fs::File;
use std::io::BufReader;
use std::{collections::HashSet, path::PathBuf};
use tokio::task::JoinHandle;
use tracing::{error, warn};

pub fn get_managed_controllers(
    client: &Client,
    mappings: &[PathBuf],
    scripts: &[PathBuf],
) -> Vec<JoinHandle<()>> {
    let mut processed_kinds = HashSet::new();

    let mappings: Vec<Mapping> = mappings
        .iter()
        .filter_map(|m| {
            File::open(m)
                .map_err(|e| error!("Failed to load mapping {:?}: {}", m, e))
                .ok()
                .and_then(|f| match serde_yaml::from_reader(BufReader::new(f)) {
                    Ok(m) => Some(m),
                    Err(e) => {
                        error!("Failed to parse {:?}: {}", m, e);
                        None
                    }
                })
        })
        .collect();

    scripts
        .iter()
        .filter_map(|script| {
            get_script_config(script)
                .map_err(|e| {
                    error!("Failed to get script config for {:?}: {:?}", script, e);
                })
                .ok()
                .and_then(|mut config| {
                    if let Some(mapping) = mappings.iter().find(|mapping| {
                        mapping.name == config.name
                            && mapping.group == config.group
                            && mapping.kind == config.kind
                            && mapping.version == config.version
                    }) {
                        let unique_key = (
                            config.group.clone(),
                            config.kind.clone(),
                            config.version.clone(),
                        );
                        if processed_kinds.insert(unique_key.clone()) {
                            if !mapping.field_selectors.is_empty() {
                                config.field_selectors = mapping.field_selectors.clone();
                            }
                            if !mapping.label_selectors.is_empty() {
                                config.label_selectors = mapping.label_selectors.clone();
                            }
                            if let Some(ran) = mapping.requeue_after_noop {
                                config.requeue_after_noop = ran;
                            };
                            if let Some(rac) = mapping.requeue_after_change {
                                config.requeue_after_change = rac;
                            };
                            Some((script.clone(), config))
                        } else {
                            error!(
                                "Duplicate group, kind, and version found for script {:?}: {:?}",
                                &script, unique_key
                            );
                            None
                        }
                    } else {
                        warn!("No mapping present for {:?}", config);
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
