use kube::api::GroupVersionKind;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub name: String,
    #[serde(default)]
    pub group: String,
    pub version: String,
    pub kind: String,

    #[serde(default, rename = "labelSelectors")]
    pub label_selectors: BTreeMap<String, String>,

    #[serde(default, rename = "fieldSelectors")]
    pub field_selectors: BTreeMap<String, String>,

    #[serde(default)]
    pub finalizer: Option<String>,

    #[serde(default)]
    pub namespace: Option<String>,

    #[serde(default = "default_requeue_after_change")]
    pub requeue_after_change: u64,
    #[serde(default = "default_requeue_after_noop")]
    pub requeue_after_noop: u64,
}

fn default_requeue_after_change() -> u64 {
    10
}

fn default_requeue_after_noop() -> u64 {
    5 * 60
}

impl From<&Config> for GroupVersionKind {
    fn from(config: &Config) -> Self {
        GroupVersionKind {
            group: config.group.clone(),
            version: config.version.clone(),
            kind: config.kind.clone(),
        }
    }
}

impl Config {
    pub fn label_selectors(&self) -> Option<String> {
        if !self.label_selectors.is_empty() {
            Some(
                self.label_selectors
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        } else {
            None
        }
    }

    pub fn field_selectors(&self) -> Option<String> {
        if !self.field_selectors.is_empty() {
            Some(
                self.field_selectors
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReconcilePhase<'a> {
    NeedsFinalizer,
    Active,
    Finalizing,
    Noop(&'a str),
}
