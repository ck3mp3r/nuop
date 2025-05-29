use crate::nuop::{
    manager::resources::{
        create_or_patch_deployment,
        deployment::{DeploymentMeta, has_drifted},
        generate_deployment,
    },
    util::{NUOP_MODE, NuopMode},
};

use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::EnvVar;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::Api;
use kube::{Client, client::Body};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_test::mock::pair;

#[tokio::test]
async fn test_create_or_patch_deployment() {
    let (mock_svc, handle) = pair::<http::Request<Body>, http::Response<Body>>();
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
            "message": "deployments \"test-deployment\" not found",
            "reason": "NotFound",
            "details": {
                "name": "test-deployment",
                "kind": "deployments"
            },
            "code": 404
        });
        send_response.send_response(
            http::Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                .unwrap(),
        );

        let (_, send_response) = handle.next_request().await.unwrap();
        let created_deployment = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "test-deployment",
                "namespace": "test-namespace",
                "resourceVersion": "1"
            },
            "spec": {
                "replicas": 1
            }
        });
        send_response.send_response(
            http::Response::builder()
                .status(201)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&created_deployment).unwrap()))
                .unwrap(),
        );
    });

    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), "test-namespace");

    let desired_deployment = Deployment {
        metadata: ObjectMeta {
            name: Some("test-deployment".to_string()),
            namespace: Some("test-namespace".to_string()),
            annotations: Some(BTreeMap::from([(
                "nuop.hash".to_string(),
                "12345".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = create_or_patch_deployment(&deployment_api, &desired_deployment).await;

    assert!(result.is_ok());
}

#[test]
fn test_has_drifted() {
    let existing = Deployment {
        metadata: ObjectMeta {
            name: Some("test-deployment".to_string()),
            annotations: Some(BTreeMap::from([(
                "nuop.hash".to_string(),
                "12345".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    };

    let desired = Deployment {
        metadata: ObjectMeta {
            name: Some("test-deployment".to_string()),
            annotations: Some(BTreeMap::from([(
                "nuop.hash".to_string(),
                "67890".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(2),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert!(has_drifted(&existing, &desired));

    let identical = Deployment {
        metadata: ObjectMeta {
            name: Some("test-deployment".to_string()),
            annotations: Some(BTreeMap::from([(
                "nuop.hash".to_string(),
                "12345".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert!(!has_drifted(&existing, &identical));
}

#[test]
fn test_generate_deployment() {
    let deployment_name = "test-deployment";
    let meta = DeploymentMeta {
        name: "test-app",
        namespace: "test-namespace",
        owner_references: None,
        service_account_name: Some("test-service-account".to_string()),
        annotations: Some(BTreeMap::from([(
            "nuop.hash".to_string(),
            "12345".to_string(),
        )])),
    };

    let image = "test-image";
    let env_vars = vec![EnvVar {
        name: "TEST_ENV".to_string(),
        value: Some("test-value".to_string()),
        ..Default::default()
    }];

    let sources = vec![];
    let mappings = vec![];

    let deployment = generate_deployment(
        deployment_name,
        meta.clone(),
        image,
        &env_vars,
        &sources,
        &mappings,
    );

    assert_eq!(deployment.metadata.name.unwrap(), meta.name);
    assert_eq!(deployment.metadata.namespace.unwrap(), meta.namespace);
    assert_eq!(
        deployment.metadata.annotations.unwrap(),
        meta.annotations.unwrap()
    );

    let spec = deployment.spec.unwrap();
    assert_eq!(spec.replicas.unwrap(), 1);
    assert_eq!(spec.selector.match_labels.unwrap()["app"], meta.name);

    let template_metadata = spec.template.metadata.unwrap();
    assert_eq!(template_metadata.labels.unwrap()["app"], meta.name);

    let container = &spec.template.spec.unwrap().containers[0];
    assert_eq!(container.name, "nureconciler");
    assert_eq!(container.image.as_ref().unwrap(), image);
    assert_eq!(container.env.as_ref().unwrap()[0].name, NUOP_MODE);
    assert_eq!(
        container.env.as_ref().unwrap()[0].value.as_ref().unwrap(),
        &NuopMode::Managed.to_string()
    );
}
