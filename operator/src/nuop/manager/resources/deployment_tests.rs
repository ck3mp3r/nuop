use crate::nuop::{
    manager::model::{Credentials, Mapping, Source},
    manager::resources::{
        create_or_patch_deployment,
        deployment::{DeploymentMeta, generate_volumes_and_mounts, has_drifted},
        generate_deployment,
    },
    util::{NUOP_MODE, NuopMode},
};

use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, EnvVar, PodSpec, PodTemplateSpec, SecretKeySelector, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::Api;
use kube::{Client, client::Body};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_test::mock::pair;

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

#[test]
fn test_has_drifted_comprehensive() {
    // Test annotation drift
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

    let desired_different_annotation = Deployment {
        metadata: ObjectMeta {
            name: Some("test-deployment".to_string()),
            annotations: Some(BTreeMap::from([(
                "nuop.hash".to_string(),
                "67890".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(has_drifted(&existing, &desired_different_annotation));

    // Test replica drift
    let desired_different_replicas = Deployment {
        metadata: ObjectMeta {
            name: Some("test-deployment".to_string()),
            annotations: Some(BTreeMap::from([(
                "nuop.hash".to_string(),
                "12345".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(3),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(has_drifted(&existing, &desired_different_replicas));

    // Test container image drift
    let existing_with_containers = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "test".to_string(),
                        image: Some("old-image".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let desired_different_image = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "test".to_string(),
                        image: Some("new-image".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(has_drifted(
        &existing_with_containers,
        &desired_different_image
    ));

    // Test container env drift
    let existing_with_env = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "test".to_string(),
                        env: Some(vec![EnvVar {
                            name: "TEST".to_string(),
                            value: Some("old".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let desired_different_env = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "test".to_string(),
                        env: Some(vec![EnvVar {
                            name: "TEST".to_string(),
                            value: Some("new".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(has_drifted(&existing_with_env, &desired_different_env));

    // Test volume mounts drift
    let existing_with_mounts = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "test".to_string(),
                        volume_mounts: Some(vec![VolumeMount {
                            name: "old-mount".to_string(),
                            mount_path: "/old".to_string(),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let desired_different_mounts = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "test".to_string(),
                        volume_mounts: Some(vec![VolumeMount {
                            name: "new-mount".to_string(),
                            mount_path: "/new".to_string(),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(has_drifted(
        &existing_with_mounts,
        &desired_different_mounts
    ));

    // Test volume drift
    let existing_with_volumes = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    volumes: Some(vec![Volume {
                        name: "old-volume".to_string(),
                        empty_dir: Some(Default::default()),
                        ..Default::default()
                    }]),
                    containers: vec![Container {
                        name: "test".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let desired_different_volumes = Deployment {
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                spec: Some(PodSpec {
                    volumes: Some(vec![Volume {
                        name: "new-volume".to_string(),
                        empty_dir: Some(Default::default()),
                        ..Default::default()
                    }]),
                    containers: vec![Container {
                        name: "test".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(has_drifted(
        &existing_with_volumes,
        &desired_different_volumes
    ));

    // Test no drift scenario
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
fn test_generate_deployment_with_sources_and_mappings() {
    let deployment_name = "test-deployment";
    let meta = DeploymentMeta {
        name: "test-app",
        namespace: "test-namespace",
        owner_references: Some(vec![OwnerReference {
            api_version: "apps/v1".to_string(),
            kind: "ReplicaSet".to_string(),
            name: "parent-rs".to_string(),
            uid: "parent-uid".to_string(),
            controller: Some(true),
            block_owner_deletion: Some(true),
        }]),
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

    let sources = vec![Source {
        location: "https://github.com/test/repo".to_string(),
        path: "test/path".to_string(),
        credentials: Some(Credentials {
            token: Some(SecretKeySelector {
                name: "test-secret".to_string(),
                key: "token".to_string(),
                optional: Some(false),
            }),
            username: None,
            password: None,
        }),
    }];

    let mappings = vec![Mapping {
        name: "test-mapping".to_string(),
        group: "apps".to_string(),
        version: "v1".to_string(),
        kind: "Deployment".to_string(),
        field_selectors: BTreeMap::from([("metadata.name".to_string(), "test".to_string())]),
        label_selectors: BTreeMap::from([("app".to_string(), "test".to_string())]),
        requeue_after_change: Some(30),
        requeue_after_noop: Some(60),
    }];

    let deployment = generate_deployment(
        deployment_name,
        meta.clone(),
        image,
        &env_vars,
        &sources,
        &mappings,
    );

    // Verify metadata
    assert_eq!(deployment.metadata.name.unwrap(), meta.name);
    assert_eq!(deployment.metadata.namespace.unwrap(), meta.namespace);
    assert_eq!(deployment.metadata.owner_references, meta.owner_references);
    assert_eq!(
        deployment.metadata.annotations.unwrap(),
        meta.annotations.unwrap()
    );

    // Verify spec
    let spec = deployment.spec.unwrap();
    assert_eq!(spec.replicas.unwrap(), 1);
    assert_eq!(spec.selector.match_labels.unwrap()["app"], meta.name);

    // Verify template
    let template_metadata = spec.template.metadata.unwrap();
    assert_eq!(template_metadata.labels.unwrap()["app"], meta.name);

    // Verify template has hash annotation
    let template_annotations = template_metadata.annotations.unwrap();
    assert_eq!(template_annotations["nuop.hash"], "12345");

    // Verify pod spec
    let pod_spec = spec.template.spec.unwrap();
    assert_eq!(pod_spec.service_account_name, meta.service_account_name);

    // Verify init containers are created when sources exist
    let init_containers = pod_spec.init_containers.unwrap();
    assert_eq!(init_containers.len(), 1);
    assert_eq!(init_containers[0].name, "init-container");
    assert_eq!(init_containers[0].image.as_ref().unwrap(), image);

    // Verify init container environment
    let init_env = init_containers[0].env.as_ref().unwrap();
    assert_eq!(init_env[0].name, NUOP_MODE);
    assert_eq!(
        init_env[0].value.as_ref().unwrap(),
        &NuopMode::Init.to_string()
    );
    assert_eq!(init_env[1].name, "TEST_ENV");
    assert_eq!(init_env[1].value.as_ref().unwrap(), "test-value");

    // Verify main container
    let container = &pod_spec.containers[0];
    assert_eq!(container.name, "nureconciler");
    assert_eq!(container.image.as_ref().unwrap(), image);

    // Verify main container environment
    let container_env = container.env.as_ref().unwrap();
    assert_eq!(container_env[0].name, NUOP_MODE);
    assert_eq!(
        container_env[0].value.as_ref().unwrap(),
        &NuopMode::Managed.to_string()
    );
    assert_eq!(container_env[1].name, "TEST_ENV");
    assert_eq!(container_env[1].value.as_ref().unwrap(), "test-value");

    // Verify volumes and mounts are present
    let volumes = pod_spec.volumes.unwrap();
    assert!(!volumes.is_empty());

    let volume_mounts = container.volume_mounts.as_ref().unwrap();
    assert!(!volume_mounts.is_empty());
}

#[test]
fn test_generate_deployment_without_sources_and_mappings() {
    let deployment_name = "test-deployment";
    let meta = DeploymentMeta {
        name: "test-app",
        namespace: "test-namespace",
        owner_references: None,
        service_account_name: None,
        annotations: None,
    };

    let image = "test-image";
    let env_vars = vec![];
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

    // Verify no init containers when sources are empty
    let pod_spec = deployment.spec.unwrap().template.spec.unwrap();
    assert!(pod_spec.init_containers.is_none());

    // Verify main container still has NUOP_MODE env var
    let container = &pod_spec.containers[0];
    let container_env = container.env.as_ref().unwrap();
    assert_eq!(container_env[0].name, NUOP_MODE);
    assert_eq!(
        container_env[0].value.as_ref().unwrap(),
        &NuopMode::Managed.to_string()
    );

    // Verify minimal volumes and mounts when no sources/mappings
    let volumes = pod_spec.volumes.unwrap();
    let volume_mounts = container.volume_mounts.as_ref().unwrap();

    // Should have empty volumes/mounts since no sources or mappings
    assert_eq!(volumes.len(), 0);
    assert_eq!(volume_mounts.len(), 0);
}

#[test]
fn test_generate_volumes_and_mounts() {
    let deployment_name = "test-deployment";

    // Test with sources and mappings
    let sources = vec![
        Source {
            location: "https://github.com/test/repo1".to_string(),
            path: "path/to/scripts".to_string(),
            credentials: Some(Credentials {
                token: Some(SecretKeySelector {
                    name: "secret1".to_string(),
                    key: "token".to_string(),
                    optional: Some(false),
                }),
                username: None,
                password: None,
            }),
        },
        Source {
            location: "https://github.com/test/repo2".to_string(),
            path: "another/path".to_string(),
            credentials: None,
        },
    ];

    let mappings = vec![Mapping {
        name: "test-mapping".to_string(),
        group: "apps".to_string(),
        version: "v1".to_string(),
        kind: "Deployment".to_string(),
        field_selectors: BTreeMap::new(),
        label_selectors: BTreeMap::new(),
        requeue_after_change: None,
        requeue_after_noop: None,
    }];

    let (volumes, mounts) = generate_volumes_and_mounts(deployment_name, &sources, &mappings);

    // Should have: scripts, config-sources, config-mappings, and secret volumes
    assert_eq!(volumes.len(), 4);
    assert_eq!(mounts.len(), 4);

    // Verify scripts volume
    let scripts_volume = volumes.iter().find(|v| v.name == "scripts").unwrap();
    assert!(scripts_volume.empty_dir.is_some());

    let scripts_mount = mounts.iter().find(|m| m.name == "scripts").unwrap();
    assert_eq!(scripts_mount.mount_path, "/scripts");

    // Verify config-sources volume
    let sources_volume = volumes.iter().find(|v| v.name == "config-sources").unwrap();
    let config_map_source = sources_volume.config_map.as_ref().unwrap();
    assert_eq!(
        config_map_source.name,
        format!("{}-nuop-sources-config", deployment_name)
    );
    assert_eq!(config_map_source.default_mode.unwrap(), 420);

    let sources_mount = mounts.iter().find(|m| m.name == "config-sources").unwrap();
    assert_eq!(sources_mount.mount_path, "/config/sources");

    // Verify config-mappings volume
    let mappings_volume = volumes
        .iter()
        .find(|v| v.name == "config-mappings")
        .unwrap();
    let config_map_source = mappings_volume.config_map.as_ref().unwrap();
    assert_eq!(
        config_map_source.name,
        format!("{}-nuop-mapping-config", deployment_name)
    );

    let mappings_mount = mounts.iter().find(|m| m.name == "config-mappings").unwrap();
    assert_eq!(mappings_mount.mount_path, "/config/mappings");

    // Verify secret volume for credentials
    let secret_volume = volumes
        .iter()
        .find(|v| v.name == "path-to-scripts-nuop-secret")
        .unwrap();
    let secret_source = secret_volume.secret.as_ref().unwrap();
    assert_eq!(secret_source.secret_name.as_ref().unwrap(), "secret1");
    assert_eq!(secret_source.default_mode.unwrap(), 420);

    let secret_mount = mounts
        .iter()
        .find(|m| m.name == "path-to-scripts-nuop-secret")
        .unwrap();
    assert_eq!(secret_mount.mount_path, "/secrets/path/to/scripts");
    assert!(secret_mount.read_only.unwrap());

    // Test with empty sources and mappings
    let (empty_volumes, empty_mounts) = generate_volumes_and_mounts(deployment_name, &[], &[]);
    assert_eq!(empty_volumes.len(), 0);
    assert_eq!(empty_mounts.len(), 0);

    // Test with only sources, no mappings
    let (source_volumes, source_mounts) =
        generate_volumes_and_mounts(deployment_name, &sources, &[]);
    assert_eq!(source_volumes.len(), 3); // scripts, config-sources, secret
    assert_eq!(source_mounts.len(), 3);

    // Test with only mappings, no sources
    let (mapping_volumes, mapping_mounts) =
        generate_volumes_and_mounts(deployment_name, &[], &mappings);
    assert_eq!(mapping_volumes.len(), 1); // config-mappings only
    assert_eq!(mapping_mounts.len(), 1);
}
