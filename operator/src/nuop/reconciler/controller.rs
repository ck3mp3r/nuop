use futures::StreamExt;
use kube::runtime::watcher::Config as WatcherConfig;
use kube::{
    Api, Client, Error,
    api::{ApiResource, DynamicObject, ResourceExt},
    runtime::{Controller, controller::Action},
};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::nuop::util::to_kube_error;

use super::config::{Config, ReconcilePhase};
use super::finalizer::{add_finalizer, detect_phase, remove_finalizer};
use super::state::State;

pub async fn reconcile(obj: Arc<DynamicObject>, ctx: Arc<State>) -> Result<Action, Error> {
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

async fn run_delegate(
    obj: &DynamicObject,
    ctx: &Arc<State>,
    command: &str,
) -> Result<Action, Error> {
    let mut child = Command::new(&ctx.script)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| to_kube_error(&e.to_string(), "Failed to spawn script", 500))?;

    if let Some(mut stdin) = child.stdin.take() {
        let input_data = serde_yaml::to_string(obj)
            .map_err(|e| to_kube_error(&e.to_string(), "Failed to serialize object", 500))?;

        debug!("Input data: {:?}", input_data);
        stdin
            .write_all(input_data.as_bytes())
            .map_err(|e| to_kube_error(&e.to_string(), "Failed to write to stdin", 500))?;

        stdin
            .flush()
            .map_err(|e| to_kube_error(&e.to_string(), "Failed to flush stdin", 500))?;

        drop(stdin);
    } else {
        return Err(to_kube_error("", "Failed to open stdin", 500));
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| to_kube_error("", "Failed to capture stdout", 500))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| to_kube_error("", "Failed to capture stderr", 500))?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stderr_reader.lines() {
        let line = line.map_err(|e| to_kube_error(&e.to_string(), "Error reading stderr", 500))?;
        error!("stderr: {}", line);
    }

    for line in stdout_reader.lines() {
        let line = line.map_err(|e| to_kube_error(&e.to_string(), "Error reading stdout", 500))?;
        info!("stdout: {}", line);
    }

    let status = child
        .wait()
        .map_err(|e| to_kube_error(&e.to_string(), "Failed to wait for script process", 500))?;

    let code = status.code().unwrap_or(1) as u16;

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
            &format!("Exit code: {}", code),
            "Script exited with error",
            code,
        )),
    }
}

pub fn error_policy(_obj: Arc<DynamicObject>, err: &Error, _ctx: Arc<State>) -> Action {
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
    let context = Arc::new(State::new(api_resource.clone(), client, config, script));

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
