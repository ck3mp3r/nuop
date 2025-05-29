use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::EnvVar;

use crate::nuop::{
    manager::resources::{deployment::DeploymentMeta, generate_deployment},
    util::{NUOP_MODE, NuopMode},
};

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
