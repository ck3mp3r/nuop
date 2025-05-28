use std::path::PathBuf;

use kube::{Client, api::ApiResource};

use super::config::Config;

#[derive(Clone)]
pub struct State {
    pub api_resource: ApiResource,
    pub client: Client,
    pub config: Config,
    pub script: PathBuf,
}

impl State {
    pub fn new(api_resource: ApiResource, client: Client, config: Config, script: PathBuf) -> Self {
        State {
            api_resource,
            client,
            config,
            script,
        }
    }
}
