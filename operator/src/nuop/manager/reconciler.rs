use std::{sync::Arc, time::Duration};

use k8s_openapi::api::{apps::v1::Deployment, core::v1::ConfigMap};
use kube::{
    Api, Resource, ResourceExt,
    api::{Patch, PatchParams, PostParams},
    runtime::controller::Action,
};
use tracing::info;

use crate::nuop::{constants::DEFAULT_IMAGE, util::generate_owner_reference};

use super::{
    NuOperator, State,
    resources::{
        deployment::DeploymentMeta, deployment_has_drifted, generate_deployment, manage_config_maps,
    },
};

pub async fn reconcile(obj: Arc<NuOperator>, ctx: Arc<State>) -> Result<Action, kube::Error> {
    let client = &ctx.client;
    let namespace = obj.namespace().unwrap();
    let name = obj.name_any();
    let deployment_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let configmap_api: Api<ConfigMap> = Api::namespaced(client.clone(), &namespace);
    let owner_ref = obj.controller_owner_ref(&());

    let deployment_name = format!("nuop-{}", name);

    let env_vars = obj.spec.env.clone();

    let sources = obj.spec.sources.clone();
    let mappings = obj.spec.mappings.clone();
    let service_account_name = obj.spec.service_account_name.clone();

    let image = obj
        .spec
        .image
        .clone()
        .unwrap_or_else(|| DEFAULT_IMAGE.to_string());

    manage_config_maps(
        &deployment_name,
        &namespace,
        &owner_ref,
        &configmap_api,
        &sources,
        &mappings,
    )
    .await?;

    let desired_deployment = generate_deployment(
        &deployment_name,
        DeploymentMeta {
            name: &deployment_name,
            namespace: &namespace,
            owner_references: generate_owner_reference(obj.as_ref()).map(|o| vec![o]),
            service_account_name,
        },
        &image,
        &env_vars,
        &sources,
        &mappings,
    );

    match deployment_api.get_opt(&deployment_name).await? {
        Some(existing) => {
            if deployment_has_drifted(&existing, &desired_deployment) {
                info!("Deployment {} has drifted. Patching...", deployment_name);

                let patch = Patch::Merge(serde_json::json!({
                    "spec": desired_deployment.spec
                }));
                deployment_api
                    .patch(
                        &deployment_name,
                        &PatchParams::apply("nureconciler"),
                        &patch,
                    )
                    .await?;
            } else {
                info!("Deployment {} is already up-to-date.", deployment_name);
            }
        }
        _ => {
            info!("Deployment {} is missing. Creating...", deployment_name);
            deployment_api
                .create(&PostParams::default(), &desired_deployment)
                .await?;
        }
    }

    Ok(Action::requeue(Duration::from_secs(300)))
}
