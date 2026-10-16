use super::support::*;

#[tokio::test]
async fn test_create_or_patch_deployment_create_scenario() {
    let (mock_svc, handle) = pair::<http::Request<Body>, http::Response<Body>>();
    let handle = Arc::new(Mutex::new(handle));
    let client = Client::new(mock_svc, "default");

    tokio::spawn(async move {
        let mut handle = handle.lock().await;

        // Mock GET request returning 404 (deployment not found)
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

        // Mock CREATE request
        let (_, send_response) = handle.next_request().await.unwrap();
        let created_deployment = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "test-deployment",
                "namespace": "test-namespace",
                "resourceVersion": "1",
                "uid": "12345-67890"
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": "test-deployment"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "test-deployment"
                        }
                    },
                    "spec": {
                        "containers": [{
                            "name": "nureconciler",
                            "image": "test-image"
                        }]
                    }
                }
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

#[tokio::test]
async fn test_create_or_patch_deployment_patch_scenario() {
    let (mock_svc, handle) = pair::<http::Request<Body>, http::Response<Body>>();
    let handle = Arc::new(Mutex::new(handle));
    let client = Client::new(mock_svc, "default");

    tokio::spawn(async move {
        let mut handle = handle.lock().await;

        // Mock GET request returning existing deployment with different hash
        let (_, send_response) = handle.next_request().await.unwrap();
        let existing_deployment = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "test-deployment",
                "namespace": "test-namespace",
                "resourceVersion": "1",
                "annotations": {
                    "nuop.hash": "old-hash"
                }
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": "test-deployment"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "test-deployment"
                        }
                    },
                    "spec": {
                        "containers": [{
                            "name": "nureconciler",
                            "image": "old-image"
                        }]
                    }
                }
            }
        });
        send_response.send_response(
            http::Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&existing_deployment).unwrap(),
                ))
                .unwrap(),
        );

        // Mock PATCH request
        let (_, send_response) = handle.next_request().await.unwrap();
        let patched_deployment = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "test-deployment",
                "namespace": "test-namespace",
                "resourceVersion": "2",
                "annotations": {
                    "nuop.hash": "new-hash"
                }
            },
            "spec": {
                "replicas": 1
            }
        });
        send_response.send_response(
            http::Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&patched_deployment).unwrap()))
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
                "new-hash".to_string(),
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

#[tokio::test]
async fn test_create_or_patch_deployment_no_change_scenario() {
    let (mock_svc, handle) = pair::<http::Request<Body>, http::Response<Body>>();
    let handle = Arc::new(Mutex::new(handle));
    let client = Client::new(mock_svc, "default");

    tokio::spawn(async move {
        let mut handle = handle.lock().await;

        // Mock GET request returning identical deployment
        let (_, send_response) = handle.next_request().await.unwrap();
        let existing_deployment = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "test-deployment",
                "namespace": "test-namespace",
                "resourceVersion": "1",
                "annotations": {
                    "nuop.hash": "same-hash"
                }
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": "test-deployment"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "test-deployment"
                        }
                    },
                    "spec": {
                        "containers": [{
                            "name": "nureconciler",
                            "image": "test-image"
                        }]
                    }
                }
            }
        });
        send_response.send_response(
            http::Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&existing_deployment).unwrap(),
                ))
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
                "same-hash".to_string(),
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
