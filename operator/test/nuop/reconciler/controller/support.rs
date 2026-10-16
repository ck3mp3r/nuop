pub(crate) use std::collections::BTreeMap;
pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;
pub(crate) use std::time::Duration;

pub(crate) use http::{Request, Response, StatusCode};
pub(crate) use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
pub(crate) use kube::{
    Client, Error,
    api::{ApiResource, DynamicObject, GroupVersionKind},
    client::Body,
    runtime::controller::Action,
};
pub(crate) use serde_json::json;
pub(crate) use tower_test::mock;

pub(crate) use super::{
    config::{Config, ReconcilePhase},
    controller::{error_policy, reconcile},
    finalizer::detect_phase,
    state::State,
};

pub(crate) fn create_test_config() -> Config {
    Config {
        name: "test-controller".to_string(),
        group: "apps".to_string(),
        version: "v1".to_string(),
        kind: "Deployment".to_string(),
        label_selectors: BTreeMap::new(),
        field_selectors: BTreeMap::new(),
        finalizer: Some("test.example.com/finalizer".to_string()),
        namespace: Some("default".to_string()),
        requeue_after_change: 10,
        requeue_after_noop: 300,
    }
}

pub(crate) fn create_test_object(
    name: &str,
    namespace: &str,
    has_finalizer: bool,
    deleting: bool,
) -> DynamicObject {
    let mut obj = DynamicObject::new(
        name,
        &ApiResource::from_gvk(&GroupVersionKind {
            group: "apps".to_string(),
            version: "v1".to_string(),
            kind: "Deployment".to_string(),
        }),
    );

    obj.metadata = ObjectMeta {
        name: Some(name.to_string()),
        namespace: Some(namespace.to_string()),
        uid: Some("test-uid-123".to_string()),
        resource_version: Some("123".to_string()),
        finalizers: if has_finalizer {
            Some(vec!["test.example.com/finalizer".to_string()])
        } else {
            None
        },
        deletion_timestamp: if deleting {
            Some(Time(k8s_openapi::jiff::Timestamp::now()))
        } else {
            None
        },
        ..Default::default()
    };

    obj.data = json!({
        "spec": {
            "replicas": 3,
            "selector": {
                "matchLabels": {
                    "app": "test"
                }
            }
        }
    });

    obj
}

pub(crate) fn get_test_script_path(script_name: &str) -> PathBuf {
    PathBuf::from(format!(
        "test/fixtures/controller/scripts/{script_name}/mod.nu",
    ))
}

// Helper function to check if we should skip script execution tests
// Skip if nu command is not available in PATH
pub(crate) fn should_skip_script_tests() -> bool {
    std::process::Command::new("nu")
        .arg("--version")
        .output()
        .is_err()
}
