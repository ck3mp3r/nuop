use super::support::*;

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
