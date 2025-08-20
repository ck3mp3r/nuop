use std::{sync::Arc, time::Duration};

use k8s_openapi::api::{apps::v1::Deployment, core::v1::ConfigMap};
use kube::{Api, Resource, ResourceExt, api::PatchParams, runtime::controller::Action};
use sha2::{Digest, Sha256};

use crate::nuop::{constants::DEFAULT_IMAGE, util::generate_owner_reference};

use super::{
    NuOperator, State,
    resources::{
        create_or_patch_config_map, create_or_patch_deployment, deployment::DeploymentMeta,
        field_manager, generate_deployment, generate_mapping_configmap, generate_source_configmap,
    },
};

pub async fn reconcile(obj: Arc<NuOperator>, ctx: Arc<State>) -> Result<Action, kube::Error> {
    let client = &ctx.client;
    let namespace = obj.namespace().unwrap();
    let name = obj.name_any();
    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let configmap_api: Api<ConfigMap> = Api::namespaced(client.clone(), &namespace);
    let owner_ref = obj.controller_owner_ref(&());

    let deployment_name = format!("{name}-nuop");

    let env_vars = obj.spec.env.clone();

    let sources = obj.spec.sources.clone();
    let mappings = obj.spec.mappings.clone();
    let service_account_name = obj.spec.service_account_name.clone();

    let image = obj
        .spec
        .image
        .clone()
        .unwrap_or_else(|| DEFAULT_IMAGE.to_string());

    let patch_params = PatchParams::apply(&field_manager::<NuOperator>());
    let mut hasher = Sha256::new();
    if let Some(mapping_cm) =
        generate_mapping_configmap(&deployment_name, &namespace, owner_ref.clone(), &mappings)
    {
        create_or_patch_config_map(&configmap_api, &mapping_cm, &patch_params).await?;
        if let Some(data) = &mapping_cm.data {
            for (key, value) in data {
                hasher.update(key);
                hasher.update(value);
            }
        }
    }

    if let Some(sources_cm) =
        generate_source_configmap(&deployment_name, &namespace, owner_ref.clone(), &sources)
    {
        create_or_patch_config_map(&configmap_api, &sources_cm, &patch_params).await?;
        if let Some(data) = &sources_cm.data {
            for (key, value) in data {
                hasher.update(key);
                hasher.update(value);
            }
        }
    }

    let desired_deployment = generate_deployment(
        &deployment_name,
        DeploymentMeta {
            name: &deployment_name,
            namespace: &namespace,
            owner_references: generate_owner_reference(obj.as_ref()).map(|o| vec![o]),
            service_account_name,
            annotations: {
                let mut annotations = std::collections::BTreeMap::new();
                annotations.insert("nuop.hash".to_string(), format!("{:x}", hasher.finalize()));
                Some(annotations)
            },
        },
        &image,
        &env_vars,
        &sources,
        &mappings,
    );

    create_or_patch_deployment(&deployment_api, &desired_deployment).await?;

    Ok(Action::requeue(Duration::from_secs(300)))
}
