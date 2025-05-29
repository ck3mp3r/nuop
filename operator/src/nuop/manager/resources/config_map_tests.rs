use std::sync::Arc;

use http::{Request, Response};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::api::{Api, PatchParams};
use kube::{Client, client::Body};
use tokio::sync::Mutex;
use tower_test::mock::pair;

use crate::nuop::manager::{Mapping, Source};

#[cfg(test)]
use super::*;

#[tokio::test]
async fn test_generate_mapping_configmap() {
    let deployment_name = "test-deployment";
    let namespace = "test-namespace";
    let mappings = vec![Mapping {
        name: "test/mapping".to_string(),
        ..Default::default()
    }];

    let configmap = generate_mapping_configmap(deployment_name, namespace, None, &mappings);

    assert!(configmap.is_some());
    let configmap = configmap.unwrap();

    assert_eq!(
        configmap.metadata.name.unwrap(),
        "test-deployment-nuop-mapping-config"
    );
    assert_eq!(configmap.metadata.namespace.unwrap(), namespace);
    assert!(configmap.data.unwrap().contains_key("test-mapping.yaml"));
}

#[tokio::test]
async fn test_generate_source_configmap() {
    let deployment_name = "test-deployment";
    let namespace = "test-namespace";
    let sources = vec![Source {
        path: "test/source".to_string(),
        ..Default::default()
    }];

    let configmap = generate_source_configmap(deployment_name, namespace, None, &sources);

    assert!(configmap.is_some());
    let configmap = configmap.unwrap();

    assert_eq!(
        configmap.metadata.name.unwrap(),
        "test-deployment-nuop-sources-config"
    );
    assert_eq!(configmap.metadata.namespace.unwrap(), namespace);
    assert!(configmap.data.unwrap().contains_key("test-source.yaml"));
}

#[tokio::test]
async fn test_create_or_patch_config_map() {
    let (mock_svc, handle) = pair::<Request<Body>, Response<Body>>();
    let handle = Arc::new(Mutex::new(handle));
    let client = Client::new(mock_svc, "default");

    tokio::spawn(async move {
        let mut handle = handle.lock().await;

        let (_, send_response) = handle.next_request().await.unwrap();

        let error_response = serde_json::json!({
            "kind": "Status",
            "apiVersion": "v1",
            "metadata": {},
            "status": "Failure",
            "message": "configmaps \"test-configmap\" not found",
            "reason": "NotFound",
            "details": {
                "name": "test-configmap",
                "kind": "configmaps"
            },
            "code": 404
        });

        send_response.send_response(
            Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                .unwrap(),
        );

        let (_, send_response) = handle.next_request().await.unwrap();

        let created_cm = serde_json::json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": "test-configmap",
                "namespace": "test-namespace",
                "resourceVersion": "1"
            },
            "data": {
                "key": "value"
            }
        });

        send_response.send_response(
            Response::builder()
                .status(201)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&created_cm).unwrap()))
                .unwrap(),
        );
    });

    let configmap_api: Api<ConfigMap> = Api::namespaced(client.clone(), "test-namespace");
    let patch_params = PatchParams::default();

    let desired_cm = ConfigMap {
        metadata: kube::api::ObjectMeta {
            name: Some("test-configmap".to_string()),
            namespace: Some("test-namespace".to_string()),
            ..Default::default()
        },
        data: Some(std::collections::BTreeMap::from([(
            "key".to_string(),
            "value".to_string(),
        )])),
        ..Default::default()
    };

    let result = create_or_patch_config_map(&configmap_api, &desired_cm, &patch_params).await;

    assert!(result.is_ok());
}
