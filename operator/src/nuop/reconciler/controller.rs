use futures::StreamExt;
use kube::runtime::watcher::Config as WatcherConfig;
use kube::{
    Api, Client, Error,
    api::{ApiResource, DynamicObject, ResourceExt},
    runtime::{Controller, controller::Action},
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::nuop::util::to_kube_error;

use super::config::{Config, ReconcilePhase};
use super::finalizer::{add_finalizer, detect_phase, remove_finalizer};
use super::state::{State, CommandExecutor};

pub async fn reconcile<E>(obj: Arc<DynamicObject>, ctx: Arc<State<E>>) -> Result<Action, Error>
where
    E: CommandExecutor,
{
    let namespace = obj.namespace().unwrap_or_default();
    let finalizer = ctx.config.finalizer.as_deref();
    let api = Api::namespaced_with(ctx.client.clone(), &namespace, &ctx.api_resource);

    let phase = detect_phase(&obj, finalizer);

    match phase {
        ReconcilePhase::NeedsFinalizer => add_finalizer(&api, &obj, finalizer.unwrap()).await,
        ReconcilePhase::Active => run_delegate(&obj, &ctx, "reconcile").await,
        ReconcilePhase::Finalizing => {
            run_delegate(&obj, &ctx, "finalize").await?;
            remove_finalizer(&api, &obj, finalizer.unwrap()).await
        }
        ReconcilePhase::Noop(cmd) => run_delegate(&obj, &ctx, cmd).await,
    }
}

async fn run_delegate<E>(
    obj: &DynamicObject,
    ctx: &Arc<State<E>>,
    command: &str,
) -> Result<Action, Error>
where
    E: CommandExecutor,
{
    let input_data = serde_yaml::to_string(obj)
        .map_err(|e| to_kube_error(&e.to_string(), "Failed to serialize object", 500))?;

    debug!("Input data: {:?}", input_data);

    let result = ctx
        .executor
        .execute(&ctx.script, command, &input_data)
        .await
        .map_err(|e| to_kube_error(&e.to_string(), "Failed to execute script", 500))?;

    if !result.stderr.is_empty() {
        for line in result.stderr.lines() {
            error!("stderr: {}", line);
        }
    }

    if !result.stdout.is_empty() {
        for line in result.stdout.lines() {
            info!("stdout: {}", line);
        }
    }

    let code = result.exit_code as u16;

    match code {
        0 => {
            info!("No changes detected for object: {}", obj.name_any());
            Ok(Action::requeue(Duration::from_secs(
                ctx.config.requeue_after_noop,
            )))
        }
        2 => {
            info!("Changes detected for object: {}", obj.name_any());
            Ok(Action::requeue(Duration::from_secs(
                ctx.config.requeue_after_change,
            )))
        }
        _ => Err(to_kube_error(
            &format!("Exit code: {code}"),
            "Script exited with error",
            code,
        )),
    }
}

pub fn error_policy<E>(_obj: Arc<DynamicObject>, err: &Error, _ctx: Arc<State<E>>) -> Action
where
    E: CommandExecutor,
{
    error!("Reconcile error: {:?}", err);
    Action::requeue(std::time::Duration::from_secs(300))
}

pub async fn controller(client: Client, config: Config, script: PathBuf) {
    let gvk = (&config).into();
    let api_resource = ApiResource::from_gvk(&gvk);
    let obj_api: Api<DynamicObject> = Api::all_with(client.clone(), &api_resource);

    info!(
        "Starting controller for config: {:?} and script: {:?}",
        &config, &script
    );
    let context = Arc::new(State::new_default(api_resource.clone(), client, config, script));

    let watcher_config = WatcherConfig {
        label_selector: context.config.label_selectors(),
        field_selector: context.config.field_selectors(),
        ..WatcherConfig::default()
    };

    Controller::new_with(obj_api, watcher_config, api_resource)
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(obj) => info!("Reconciliation successful: {:?}", obj),
                Err(e) => warn!("Reconciliation failed: {:?}", e),
            }
        })
        .await;
}
