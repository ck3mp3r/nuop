use k8s_openapi::api::core::v1::EnvVar;

use crate::nuop::{
    manager::resources::{
        deployment::{DeploymentMeta, has_drifted},
        generate_deployment,
    },
    util::{NUOP_MODE, NuopMode},
};

use std::collections::BTreeMap;

#[test]
fn test_has_drifted() {
    use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::BTreeMap;

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
