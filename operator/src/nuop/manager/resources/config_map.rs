use kube::api::{Patch, PatchParams, PostParams};
use kube::{Api, Resource};
use std::collections::BTreeMap;

use k8s_openapi::{api::core::v1::ConfigMap, apimachinery::pkg::apis::meta::v1::OwnerReference};
use kube::api::ObjectMeta;

use crate::nuop::manager::model::{Mapping, NuOperator, Source};

pub(crate) const NUOP_SOURCES_CONFIG: &str = "nuop-sources-config";
pub(crate) const NUOP_MAPPING_CONFIG: &str = "nuop-mapping-config";

pub(crate) async fn manage_config_maps(
    deployment_name: &str,
    namespace: &str,
    owner_ref: &Option<OwnerReference>,
    configmap_api: &Api<ConfigMap>,
    sources: &[Source],
    mappings: &[Mapping],
) -> Result<(), kube::Error> {
    let patch_params = PatchParams::apply(&field_manager::<NuOperator>());
    if !mappings.is_empty() {
        create_or_patch_config_map(
            &generate_mapping_configmap(deployment_name, mappings, namespace, owner_ref.clone()),
            configmap_api,
            &patch_params,
        )
        .await?;
    }
    if !sources.is_empty() {
        create_or_patch_config_map(
            &generate_source_configmap(deployment_name, sources, namespace, owner_ref.clone()),
            configmap_api,
            &patch_params,
        )
        .await?;
    }
    Ok(())
}

pub(crate) fn generate_mapping_configmap(
    deployment_name: &str,
    mappings: &[Mapping],
    namespace: &str,
    owner_ref: Option<OwnerReference>,
) -> ConfigMap {
    let mut combined_data = BTreeMap::new();

    for mapping in mappings {
        let name = mapping.name.replace("/", "-");
        let yaml = match serde_yaml::to_string(&mapping) {
            Ok(yaml_string) => yaml_string,
            Err(e) => {
                tracing::error!("Failed to serialize mapping to YAML: {:?}", e);
                "".to_string()
            }
        };
        combined_data.insert(format!("{}.yaml", name), yaml);
    }

    let mut metadata = ObjectMeta {
        name: Some(format!("{}-{}", deployment_name, NUOP_MAPPING_CONFIG)),
        namespace: Some(namespace.to_string()),
        ..Default::default()
    };

    if let Some(ref owner) = owner_ref {
        metadata.owner_references = Some(vec![owner.clone()]);
    }

    ConfigMap {
        metadata,
        data: Some(combined_data),
        ..Default::default()
    }
}

pub(crate) fn generate_source_configmap(
    deployment_name: &str,
    sources: &[Source],
    namespace: &str,
    owner_ref: Option<OwnerReference>,
) -> ConfigMap {
    let mut combined_data = BTreeMap::new();

    for source in sources {
        let name = source.path.replace("/", "-");
        let yaml = match serde_yaml::to_string(source) {
            Ok(yaml_string) => yaml_string,
            Err(e) => {
                tracing::error!("Failed to serialize source to YAML: {:?}", e);
                "".to_string()
            }
        };
        combined_data.insert(format!("{}.yaml", name), yaml);
    }

    let mut metadata = ObjectMeta {
        name: Some(format!("{}-{}", deployment_name, NUOP_SOURCES_CONFIG)),
        namespace: Some(namespace.to_string()),
        ..Default::default()
    };

    if let Some(ref owner) = owner_ref {
        metadata.owner_references = Some(vec![owner.clone()]);
    }

    ConfigMap {
        metadata,
        data: Some(combined_data),
        ..Default::default()
    }
}

async fn create_or_patch_config_map(
    desired_cm: &ConfigMap,
    configmap_api: &Api<ConfigMap>,
    patch_params: &PatchParams,
) -> Result<(), kube::Error> {
    let name = desired_cm
        .metadata
        .name
        .as_deref()
        .expect("ConfigMap must have a name");

    if let Some(existing_cm) = configmap_api.get_opt(name).await? {
        let desired_data = desired_cm.data.as_ref();
        let existing_data = existing_cm.data.as_ref();

        let desired_bin_data = desired_cm.binary_data.as_ref();
        let existing_bin_data = existing_cm.binary_data.as_ref();

        if desired_data != existing_data || desired_bin_data != existing_bin_data {
            if let Some(resource_version) = existing_cm.metadata.resource_version.clone() {
                let mut updated_cm = desired_cm.clone();
                updated_cm.metadata.resource_version = Some(resource_version);

                tracing::debug!("Updating ConfigMap '{}'", name);
                configmap_api
                    .patch(name, patch_params, &Patch::Apply(&updated_cm))
                    .await?;
                tracing::info!("Updated ConfigMap '{}'", name);
            }
        } else {
            tracing::info!("ConfigMap '{}' is already up to date", name);
        }
    } else {
        tracing::debug!("Attempting to create ConfigMap '{}'", name);
        configmap_api
            .create(&PostParams::default(), desired_cm)
            .await?;
        tracing::info!("Created ConfigMap '{}'", name);
    }

    Ok(())
}

pub fn field_manager<T: Resource<DynamicType = ()>>() -> String {
    format!("{}.{}", T::kind(&()), T::api_version(&()))
}
