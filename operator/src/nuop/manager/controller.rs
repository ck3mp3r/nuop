use super::reconciler::reconcile;
use super::state::State;
use crate::nuop::manager::NuOperator;
use futures::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::runtime::controller::Action;
use kube::{Client, ResourceExt, api::Api, runtime::controller::Controller};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn controller(client: Client) {
    let nureconciler_api: Api<NuOperator> = Api::all(client.clone());
    let deployment_api: Api<Deployment> = Api::all(client.clone());

    let context = Arc::new(State::new(client.clone()));

    Controller::new(nureconciler_api, Default::default())
        .owns(deployment_api, Default::default())
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(_) => info!("Reconciliation successful"),
                Err(e) => warn!("Reconciliation failed: {:?}", e),
            }
        })
        .await;
}

pub fn error_policy(
    nureconciler: Arc<NuOperator>,
    error: &kube::Error,
    _ctx: Arc<State>,
) -> Action {
    error!(
        "Reconciliation error for {}: {:?}",
        nureconciler.name_any(),
        error
    );
    Action::requeue(Duration::from_secs(60))
}
