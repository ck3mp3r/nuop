mod config_map;
pub(crate) mod deployment;

pub(crate) use config_map::create_or_patch_config_map;
pub(crate) use config_map::field_manager;
pub(crate) use config_map::generate_mapping_configmap;
pub(crate) use config_map::generate_source_configmap;
pub(crate) use deployment::create_or_patch_deployment;
pub(crate) use deployment::generate_deployment;

#[cfg(test)]
#[path = "../../../../test/nuop/manager/resources/config_map.rs"]
mod config_map_tests;

#[cfg(test)]
#[path = "../../../../test/nuop/manager/resources/deployment/support.rs"]
mod support;

#[cfg(test)]
#[path = "../../../../test/nuop/manager/resources/deployment/create_or_patch.rs"]
mod deployment_create_or_patch_tests;

#[cfg(test)]
#[path = "../../../../test/nuop/manager/resources/deployment/drift.rs"]
mod deployment_drift_tests;

#[cfg(test)]
#[path = "../../../../test/nuop/manager/resources/deployment/generate.rs"]
mod deployment_generate_tests;
