use std::collections::BTreeMap;
use std::sync::Arc;

use http::{Request, Response};
use k8s_openapi::api::core::v1::{ConfigMap, SecretKeySelector};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{Api, PatchParams};
use kube::{Client, client::Body};
use tokio::sync::Mutex;
use tower_test::mock::pair;

use crate::nuop::manager::model::{Credentials, Mapping, Source};
use crate::nuop::manager::resources::{
    create_or_patch_config_map, field_manager, generate_source_configmap,
};

use super::generate_mapping_configmap;

#[test]
fn test_generate_configmaps() {
    let deployment_name = "test-deployment";
    let namespace = "test-namespace";

    // Test with empty collections - should return None
    let empty_mappings: Vec<Mapping> = vec![];
    assert!(
        generate_mapping_configmap(deployment_name, namespace, None, &empty_mappings).is_none()
    );

    let empty_sources: Vec<Source> = vec![];
    assert!(generate_source_configmap(deployment_name, namespace, None, &empty_sources).is_none());

    // Test with data including path replacement and owner references
    let owner_ref = OwnerReference {
        api_version: "v1".to_string(),
        kind: "Deployment".to_string(),
        name: "test-deployment".to_string(),
        uid: "12345".to_string(),
        ..Default::default()
    };

    let mappings = vec![Mapping {
        name: "test/mapping".to_string(),
        group: "apps".to_string(),
        version: "v1".to_string(),
        kind: "Deployment".to_string(),
        ..Default::default()
    }];

    let configmap = generate_mapping_configmap(
        deployment_name,
        namespace,
        Some(owner_ref.clone()),
        &mappings,
    )
    .unwrap();
    assert_eq!(
        configmap.metadata.name.unwrap(),
        "test-deployment-nuop-mapping-config"
    );
    assert_eq!(configmap.metadata.namespace.unwrap(), namespace);
    assert_eq!(configmap.metadata.owner_references.unwrap()[0], owner_ref);
    assert!(configmap.data.unwrap().contains_key("test-mapping.yaml"));

    let sources = vec![Source {
        location: "https://github.com/example/repo.git".to_string(),
        path: "test/source".to_string(),
        credentials: Some(Credentials {
            token: Some(SecretKeySelector {
                name: "github-token".to_string(),
                key: "token".to_string(),
                optional: Some(false),
            }),
            username: None,
            password: None,
        }),
    }];

    let configmap = generate_source_configmap(
        deployment_name,
        namespace,
        Some(owner_ref.clone()),
        &sources,
    )
    .unwrap();
    assert_eq!(
        configmap.metadata.name.unwrap(),
        "test-deployment-nuop-sources-config"
    );
    assert!(configmap.data.unwrap().contains_key("test-source.yaml"));
}

#[tokio::test]
async fn test_create_or_patch_config_map_scenarios() {
    let (mock_svc, handle) = pair::<Request<Body>, Response<Body>>();
    let handle = Arc::new(Mutex::new(handle));
    let client = Client::new(mock_svc, "default");

    tokio::spawn(async move {
        let mut handle = handle.lock().await;

        // Test 1: ConfigMap doesn't exist (404) -> create
        let (_, send_response) = handle.next_request().await.unwrap();
        send_response.send_response(
            Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
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
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        );

        let (_, send_response) = handle.next_request().await.unwrap();
        send_response.send_response(
            Response::builder()
                .status(201)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "apiVersion": "v1",
                        "kind": "ConfigMap",
                        "metadata": {
                            "name": "test-configmap",
                            "namespace": "test-namespace",
                            "resourceVersion": "1",
                            "uid": "12345-67890",
                            "creationTimestamp": "2023-01-01T00:00:00Z"
                        },
                        "data": {
                            "key": "value"
                        }
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        );

        // Test 2: ConfigMap exists with different data -> patch
        let (_, send_response) = handle.next_request().await.unwrap();
        send_response.send_response(
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "apiVersion": "v1",
                        "kind": "ConfigMap",
                        "metadata": {
                            "name": "test-configmap",
                            "namespace": "test-namespace",
                            "resourceVersion": "1",
                            "uid": "12345-67890"
                        },
                        "data": {"key": "old-value"}
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        );

        let (_, send_response) = handle.next_request().await.unwrap();
        send_response.send_response(
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "apiVersion": "v1",
                        "kind": "ConfigMap",
                        "metadata": {
                            "name": "test-configmap",
                            "namespace": "test-namespace",
                            "resourceVersion": "2",
                            "uid": "12345-67890"
                        },
                        "data": {"key": "value"}
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        );

        // Test 3: ConfigMap exists with same data -> no action
        let (_, send_response) = handle.next_request().await.unwrap();
        send_response.send_response(
            Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "apiVersion": "v1",
                        "kind": "ConfigMap",
                        "metadata": {
                            "name": "test-configmap",
                            "namespace": "test-namespace",
                            "resourceVersion": "1",
                            "uid": "12345-67890"
                        },
                        "data": {"key": "value"}
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        );
    });

    let configmap_api: Api<ConfigMap> = Api::namespaced(client, "test-namespace");
    let patch_params = PatchParams::default();

    let desired_cm = ConfigMap {
        metadata: kube::api::ObjectMeta {
            name: Some("test-configmap".to_string()),
            ..Default::default()
        },
        data: Some(BTreeMap::from([("key".to_string(), "value".to_string())])),
        ..Default::default()
    };

    // Test all three scenarios
    assert!(
        create_or_patch_config_map(&configmap_api, &desired_cm, &patch_params)
            .await
            .is_ok()
    );
    assert!(
        create_or_patch_config_map(&configmap_api, &desired_cm, &patch_params)
            .await
            .is_ok()
    );
    assert!(
        create_or_patch_config_map(&configmap_api, &desired_cm, &patch_params)
            .await
            .is_ok()
    );
}

#[test]
fn test_field_manager() {
    let result = field_manager::<ConfigMap>();
    assert_eq!(result, "ConfigMap.v1");
}
