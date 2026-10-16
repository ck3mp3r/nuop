use super::support::*;

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
        source_type: SourceType::Scripts,
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
            source_type: SourceType::Scripts,
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
            source_type: SourceType::Scripts,
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

    // Should have: sources, config-sources, config-mappings, and secret volumes
    assert_eq!(volumes.len(), 4);
    assert_eq!(mounts.len(), 4);

    // Verify sources volume
    let sources_volume = volumes.iter().find(|v| v.name == "sources").unwrap();
    assert!(sources_volume.empty_dir.is_some());

    let sources_mount = mounts.iter().find(|m| m.name == "sources").unwrap();
    assert_eq!(sources_mount.mount_path, "/sources");

    // Verify no legacy scripts volume or mount remains
    assert!(volumes.iter().all(|v| v.name != "scripts"));
    assert!(mounts.iter().all(|m| m.mount_path != "/scripts"));

    // Verify config-sources volume
    let config_sources_volume = volumes.iter().find(|v| v.name == "config-sources").unwrap();
    let config_map_source = config_sources_volume.config_map.as_ref().unwrap();
    assert_eq!(
        config_map_source.name,
        format!("{deployment_name}-nuop-sources-config")
    );
    assert_eq!(config_map_source.default_mode.unwrap(), 420);

    let config_sources_mount = mounts.iter().find(|m| m.name == "config-sources").unwrap();
    assert_eq!(config_sources_mount.mount_path, "/config/sources");

    // Verify config-mappings volume
    let mappings_volume = volumes
        .iter()
        .find(|v| v.name == "config-mappings")
        .unwrap();
    let config_map_source = mappings_volume.config_map.as_ref().unwrap();
    assert_eq!(
        config_map_source.name,
        format!("{deployment_name}-nuop-mapping-config")
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
    assert_eq!(source_volumes.len(), 3); // sources, config-sources, secret
    assert_eq!(source_mounts.len(), 3);

    // Test with only mappings, no sources
    let (mapping_volumes, mapping_mounts) =
        generate_volumes_and_mounts(deployment_name, &[], &mappings);
    assert_eq!(mapping_volumes.len(), 1); // config-mappings only
    assert_eq!(mapping_mounts.len(), 1);
}
