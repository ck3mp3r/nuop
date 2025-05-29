use crate::nuop::manager::{
    NuOperator, controller::error_policy, reconciler::reconcile, state::State,
};

use k8s_openapi::api::core::v1::EnvVar;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::error::ErrorResponse;
use kube::runtime::controller::Action;
use kube::{Client, Error as KubeError, client::Body};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower_test::mock::pair;

#[tokio::test]
async fn test_error_policy_returns_requeue_action() {
    let nuoperator = Arc::new(NuOperator {
        metadata: ObjectMeta {
            name: Some("test-nuoperator".to_string()),
            namespace: Some("test-namespace".to_string()),
            ..Default::default()
        },
        spec: Default::default(),
    });

    let ctx = Arc::new(State::new(Client::try_default().await.unwrap()));
    let error = KubeError::Api(ErrorResponse {
        status: "Failure".to_string(),
        message: "Test error".to_string(),
        reason: "TestFailure".to_string(),
        code: 500,
    });

    let action = error_policy(nuoperator, &error, ctx);

    assert_eq!(action, Action::requeue(Duration::from_secs(60)));
}

#[tokio::test]
async fn test_error_policy_with_different_error_types() {
    let nuoperator = Arc::new(NuOperator {
        metadata: ObjectMeta {
            name: Some("test-nuoperator".to_string()),
            namespace: Some("test-namespace".to_string()),
            ..Default::default()
        },
        spec: Default::default(),
    });

    let ctx = Arc::new(State::new(Client::try_default().await.unwrap()));

    let api_error = KubeError::Api(ErrorResponse {
        status: "Failure".to_string(),
        message: "API error".to_string(),
        reason: "BadRequest".to_string(),
        code: 400,
    });

    let action = error_policy(nuoperator.clone(), &api_error, ctx.clone());
    assert_eq!(action, Action::requeue(Duration::from_secs(60)));

    let not_found_error = KubeError::Api(ErrorResponse {
        status: "Failure".to_string(),
        message: "Resource not found".to_string(),
        reason: "NotFound".to_string(),
        code: 404,
    });

    let action = error_policy(nuoperator, &not_found_error, ctx);
    assert_eq!(action, Action::requeue(Duration::from_secs(60)));
}

#[tokio::test]
async fn test_reconcile_via_controller_creates_deployment() {
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
            "message": "deployments \"test-nuoperator-nuop\" not found",
            "reason": "NotFound",
            "details": {
                "name": "test-nuoperator-nuop",
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
                "name": "test-nuoperator-nuop",
                "namespace": "test-namespace",
                "resourceVersion": "1",
                "uid": "deployment-uid-123",
                "creationTimestamp": "2023-01-01T00:00:00Z"
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": "test-nuoperator"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "test-nuoperator"
                        }
                    },
                    "spec": {
                        "containers": [{
                            "name": "nureconciler",
                            "image": "ghcr.io/ck3mp3r/nuop:latest"
                        }]
                    }
                }
            },
            "status": {
                "replicas": 0,
                "readyReplicas": 0,
                "updatedReplicas": 0
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

    let nuoperator = Arc::new(NuOperator {
        metadata: ObjectMeta {
            name: Some("test-nuoperator".to_string()),
            namespace: Some("test-namespace".to_string()),
            uid: Some("test-uid".to_string()),
            ..Default::default()
        },
        spec: Default::default(), // Minimal spec - no sources or mappings
    });

    let ctx = Arc::new(State::new(client));

    let result = reconcile(nuoperator, ctx).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Action::requeue(Duration::from_secs(300)));
}

#[tokio::test]
async fn test_reconcile_via_controller_with_custom_spec() {
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
            "message": "deployments \"custom-nuoperator-nuop\" not found",
            "reason": "NotFound",
            "details": {
                "name": "custom-nuoperator-nuop",
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
                "name": "custom-nuoperator-nuop",
                "namespace": "test-namespace",
                "resourceVersion": "1",
                "uid": "deployment-uid-456",
                "creationTimestamp": "2023-01-01T00:00:00Z"
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": "custom-nuoperator"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "custom-nuoperator"
                        }
                    },
                    "spec": {
                        "serviceAccountName": "custom-sa",
                        "containers": [{
                            "name": "nureconciler",
                            "image": "custom-image:v1.0",
                            "env": [{
                                "name": "CUSTOM_ENV",
                                "value": "custom-value"
                            }]
                        }]
                    }
                }
            },
            "status": {
                "replicas": 0,
                "readyReplicas": 0,
                "updatedReplicas": 0
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

    let nuoperator = Arc::new(NuOperator {
        metadata: ObjectMeta {
            name: Some("custom-nuoperator".to_string()),
            namespace: Some("test-namespace".to_string()),
            uid: Some("test-uid".to_string()),
            ..Default::default()
        },
        spec: crate::nuop::manager::model::NuOperatorSpec {
            env: vec![EnvVar {
                name: "CUSTOM_ENV".to_string(),
                value: Some("custom-value".to_string()),
                ..Default::default()
            }],
            image: Some("custom-image:v1.0".to_string()),
            mappings: vec![],
            sources: vec![],
            service_account_name: Some("custom-sa".to_string()),
        },
    });

    let ctx = Arc::new(State::new(client));

    let result = reconcile(nuoperator, ctx).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Action::requeue(Duration::from_secs(300)));
}

#[tokio::test]
async fn test_reconcile_via_controller_handles_errors() {
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
            "message": "deployments \"error-nuoperator-nuop\" not found",
            "reason": "NotFound",
            "details": {
                "name": "error-nuoperator-nuop",
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
        let error_response = serde_json::json!({
            "kind": "Status",
            "apiVersion": "v1",
            "metadata": {},
            "status": "Failure",
            "message": "Internal server error during deployment creation",
            "reason": "InternalError",
            "details": {
                "name": "error-nuoperator-nuop",
                "kind": "deployments"
            },
            "code": 500
        });
        send_response.send_response(
            http::Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&error_response).unwrap()))
                .unwrap(),
        );
    });

    let nuoperator = Arc::new(NuOperator {
        metadata: ObjectMeta {
            name: Some("error-nuoperator".to_string()),
            namespace: Some("test-namespace".to_string()),
            uid: Some("test-uid".to_string()),
            ..Default::default()
        },
        spec: Default::default(),
    });

    let ctx = Arc::new(State::new(client));

    let result = reconcile(nuoperator, ctx).await;

    assert!(result.is_err());
    // Verify it's a KubeError::Api
    match result.unwrap_err() {
        KubeError::Api(error_response) => {
            assert_eq!(error_response.code, 500);
            assert_eq!(error_response.reason, "InternalError");
        }
        _ => panic!("Expected KubeError::Api"),
    }
}
