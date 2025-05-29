use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default)]
pub struct Mapping {
    /// name of the script that it returns from configuration
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) group: String,
    pub(crate) version: String,
    pub(crate) kind: String,
    #[serde(
        default,
        rename = "fieldSelectors",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub(crate) field_selectors: BTreeMap<String, String>,
    #[serde(
        default,
        rename = "labelSelectors",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub(crate) label_selectors: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requeue_after_change: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requeue_after_noop: Option<u64>,
}
