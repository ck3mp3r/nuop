use crate::nuop::{
    constants::{NUOP_MAPPING_CONFIG, NUOP_SOURCES_CONFIG},
    manager::model::{Mapping, Source},
    util::{NUOP_MODE, NuopMode},
};
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMapVolumeSource, Container, EnvVar, PodSpec, PodTemplateSpec, SecretVolumeSource,
            Volume, VolumeMount,
        },
    },
    apimachinery::pkg::apis::meta::v1::{LabelSelector, OwnerReference},
};
use kube::api::ObjectMeta;
use std::{collections::BTreeMap, iter::once};
use tracing::debug;

pub(crate) struct DeploymentMeta<'a> {
    pub(crate) name: &'a str,
    pub(crate) namespace: &'a str,
    pub(crate) owner_references: Option<Vec<OwnerReference>>,
    pub(crate) service_account_name: Option<String>,
    pub(crate) annotations: Option<BTreeMap<String, String>>,
}

pub(crate) fn generate_deployment(
    deployment_name: &str,
    meta: DeploymentMeta,
    image: &str,
    env_vars: &[EnvVar],
    sources: &[Source],
    mappings: &[Mapping],
) -> Deployment {
    let (volumes, volume_mounts) = generate_volumes_and_mounts(deployment_name, sources, mappings);

    let init_containers = if !sources.is_empty() {
        Some(vec![Container {
            name: "init-container".to_string(),
            image: Some(image.to_string()),
            volume_mounts: Some(volume_mounts.clone()),
            image_pull_policy: Some("Never".to_string()),
            env: Some(
                once(EnvVar {
                    name: NUOP_MODE.to_string(),
                    value: Some(NuopMode::Init.to_string()),
                    ..Default::default()
                })
                .chain(env_vars.to_vec())
                .collect::<Vec<EnvVar>>(),
            ),
            ..Default::default()
        }])
    } else {
        None
    };

    Deployment {
        metadata: ObjectMeta {
            name: Some(meta.name.to_string()),
            namespace: Some(meta.namespace.to_string()),
            owner_references: meta.owner_references.clone(),
            annotations: meta.annotations.clone(),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([("app".to_string(), meta.name.to_string())])),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([("app".to_string(), meta.name.to_string())])),
                    annotations: meta.annotations.as_ref().and_then(|annotations| {
                        annotations.get("nuop.hash").map(|hash| {
                            let mut template_annotations = BTreeMap::new();
                            template_annotations.insert("nuop.hash".to_string(), hash.clone());
                            template_annotations
                        })
                    }),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    init_containers,
                    service_account_name: meta.service_account_name.clone(),
                    volumes: Some(volumes),
                    containers: vec![Container {
                        name: "nureconciler".to_string(),
                        image: Some(image.to_string()),
                        image_pull_policy: Some("Never".to_string()),
                        env: Some(
                            once(EnvVar {
                                name: NUOP_MODE.to_string(),
                                value: Some(NuopMode::Managed.to_string()),
                                ..Default::default()
                            })
                            .chain(env_vars.to_vec())
                            .collect::<Vec<EnvVar>>(),
                        ),
                        volume_mounts: Some(volume_mounts),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub(crate) fn has_drifted(existing: &Deployment, desired: &Deployment) -> bool {
    let existing_spec = existing.spec.as_ref();
    let desired_spec = desired.spec.as_ref();

    if let (Some(existing_spec), Some(desired_spec)) = (existing_spec, desired_spec) {
        if existing_spec.replicas != desired_spec.replicas {
            debug!(
                "Replicas have diverged: existing {:?} vs. desired {:?}",
                existing_spec.replicas, desired_spec.replicas
            );
            return true;
        }

        if existing.metadata.annotations != desired.metadata.annotations {
            debug!(
                "Annotations have diverged: {:?} vs. {:?}",
                existing.metadata.annotations, desired.metadata.annotations
            );
            return true;
        }

        let existing_volumes = existing_spec
            .template
            .spec
            .as_ref()
            .map(|pod_spec| pod_spec.volumes.as_slice())
            .unwrap_or(&[]);
        let desired_volumes = desired_spec
            .template
            .spec
            .as_ref()
            .map(|pod_spec| pod_spec.volumes.as_slice())
            .unwrap_or(&[]);
        for (existing, desired) in existing_volumes.iter().zip(desired_volumes.iter()) {
            if existing != desired {
                debug!("Volumes have diverged: {:?} vs. {:?}", existing, desired);
                return true;
            }
        }

        let existing_containers = &existing_spec.template.spec.as_ref().map(|s| &s.containers);
        let desired_containers = &desired_spec.template.spec.as_ref().map(|s| &s.containers);

        if let (Some(existing_containers), Some(desired_containers)) =
            (existing_containers, desired_containers)
        {
            for (existing, desired) in existing_containers.iter().zip(desired_containers.iter()) {
                if existing.image != desired.image {
                    debug!(
                        "Container images have diverged: {:?} vs. {:?}",
                        existing.image, desired.image
                    );
                    return true;
                }
                if existing.env != desired.env {
                    debug!(
                        "Container environment variables have diverged: {:?} vs. {:?}",
                        existing.env, desired.env
                    );
                    return true;
                }
                if existing.volume_mounts != desired.volume_mounts {
                    debug!(
                        "Container volume mounts have diverged: {:?} vs. {:?}",
                        existing.volume_mounts, desired.volume_mounts
                    );
                    return true;
                }
            }
        }
    }

    false
}

pub fn generate_volumes_and_mounts(
    deployment_name: &str,
    sources: &[Source],
    mappings: &[Mapping],
) -> (Vec<Volume>, Vec<VolumeMount>) {
    let mut volumes = Vec::new();
    let mut mounts = Vec::new();

    if !sources.is_empty() {
        volumes.push(Volume {
            name: "scripts".to_string(),
            empty_dir: Some(Default::default()),
            ..Default::default()
        });

        mounts.push(VolumeMount {
            name: "scripts".to_string(),
            mount_path: "/scripts".to_string(),
            ..Default::default()
        });

        volumes.push(Volume {
            name: "config-sources".to_string(),
            config_map: Some(ConfigMapVolumeSource {
                name: format!("{}-{}", deployment_name, NUOP_SOURCES_CONFIG),
                default_mode: Some(420),
                ..Default::default()
            }),
            ..Default::default()
        });

        mounts.push(VolumeMount {
            name: "config-sources".to_string(),
            mount_path: "/config/sources".to_string(),
            ..Default::default()
        });
    }

    if !mappings.is_empty() {
        volumes.push(Volume {
            name: "config-mappings".to_string(),
            config_map: Some(ConfigMapVolumeSource {
                name: format!("{}-{}", deployment_name, NUOP_MAPPING_CONFIG),
                default_mode: Some(420),
                ..Default::default()
            }),
            ..Default::default()
        });
        mounts.push(VolumeMount {
            name: "config-mappings".to_string(),
            mount_path: "/config/mappings".to_string(),
            ..Default::default()
        });
    }

    for source in sources {
        let name = source.path.replace("/", "-");

        if let Some(creds) = &source.credentials {
            let secret_name = creds
                .token
                .as_ref()
                .or(creds.username.as_ref())
                .or(creds.password.as_ref())
                .map(|s| s.name.clone());

            if let Some(secret_name) = secret_name {
                let name = format!("{}-nuop-secret", name);
                volumes.push(Volume {
                    name: name.to_string(),
                    secret: Some(SecretVolumeSource {
                        secret_name: Some(secret_name.clone()),
                        default_mode: Some(420),
                        ..Default::default()
                    }),
                    ..Default::default()
                });

                mounts.push(VolumeMount {
                    name,
                    mount_path: format!("/secrets/{}", source.path),
                    read_only: Some(true),
                    ..Default::default()
                });
            }
        }
    }

    (volumes, mounts)
}
