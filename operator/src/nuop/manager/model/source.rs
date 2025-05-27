use k8s_openapi::api::core::v1::SecretKeySelector;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
pub struct Source {
    /// source location i.e. github repo
    pub(crate) location: String,
    /// path that is to be used for the volume mounts (configs and secrets)
    pub(crate) path: String,
    /// credentials to be used to fetch source from location
    pub(crate) credentials: Option<Credentials>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
pub struct Credentials {
    pub(crate) token: Option<SecretKeySelector>,
    pub(crate) username: Option<SecretKeySelector>,
    pub(crate) password: Option<SecretKeySelector>,
}
