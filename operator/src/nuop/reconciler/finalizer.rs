use crate::nuop::util::to_kube_error;

use super::config::ReconcilePhase;
use kube::{Api, Error, ResourceExt, api::DynamicObject, runtime::controller::Action};
use std::time::Duration;
use tracing::info;

pub fn detect_phase<'a>(obj: &DynamicObject, finalizer: Option<&'a str>) -> ReconcilePhase<'a> {
    match finalizer {
        None => ReconcilePhase::Noop("reconcile"),
        Some(f) => {
            let has = obj
                .metadata
                .finalizers
                .as_ref()
                .map_or(false, |fs| fs.contains(&f.to_string()));
            let deleting = obj.metadata.deletion_timestamp.is_some();

            match (deleting, has) {
                (true, true) => ReconcilePhase::Finalizing,
                (_, false) => ReconcilePhase::NeedsFinalizer,
                _ => ReconcilePhase::Active,
            }
        }
    }
}

pub async fn add_finalizer(
    api: &Api<DynamicObject>,
    obj: &DynamicObject,
    finalizer: &str,
) -> Result<Action, Error> {
    let mut obj = obj.clone();
    let finalizers = obj.metadata.finalizers.get_or_insert_with(Vec::new);

    if !finalizers.contains(&finalizer.to_string()) {
        finalizers.push(finalizer.to_string());
        api.replace(&obj.name_any(), &Default::default(), &obj)
            .await
            .map_err(|e| to_kube_error(&e.to_string(), "Failed to add finalizer", 500))?;

        info!(
            "Added finalizer to {}/{}",
            obj.namespace().unwrap_or_default(),
            obj.name_any()
        );
        return Ok(Action::requeue(Duration::from_secs(5)));
    }

    Ok(Action::await_change())
}

pub async fn remove_finalizer(
    api: &Api<DynamicObject>,
    obj: &DynamicObject,
    finalizer: &str,
) -> Result<Action, Error> {
    let mut obj = obj.clone();

    obj.metadata.finalizers = Some(
        obj.metadata
            .finalizers
            .take()
            .unwrap_or_default()
            .into_iter()
            .filter(|f| f != finalizer)
            .collect(),
    );

    api.replace(&obj.name_any(), &Default::default(), &obj)
        .await
        .map_err(|e| to_kube_error(&e.to_string(), "Failed to remove finalizer", 500))?;

    info!(
        "Removed finalizer from {}/{}",
        obj.namespace().unwrap_or_default(),
        obj.name_any()
    );

    Ok(Action::await_change())
}
