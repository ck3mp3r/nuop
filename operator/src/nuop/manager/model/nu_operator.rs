use k8s_openapi::api::core::v1::EnvVar;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{mapping::Mapping, source::Source};

#[derive(CustomResource, Default, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "kemper.buzz",
    version = "v1alpha1",
    kind = "NuOperator",
    plural = "nuoperators",
    namespaced
)]
pub struct NuOperatorSpec {
    /// supply potentially required environment variables
    #[serde(default)]
    pub(crate) env: Vec<EnvVar>,
    /// alternative image to use that builds on default image
    #[serde(default)]
    pub(crate) image: Option<String>,
    /// mappings to be used to narrow down which scripts to register
    #[serde(default)]
    pub(crate) mappings: Vec<Mapping>,
    /// sources to fetch that contain the reconcile scripts
    #[serde(default)]
    pub(crate) sources: Vec<Source>,
    /// service account to use
    #[serde(default, rename = "serviceAccountName")]
    pub(crate) service_account_name: Option<String>,
}
