use futures::future::try_join_all;
use kube::Client;
use operator::nuop::manager::manager_controller;
use operator::nuop::{logging, util::NuopMode};
use tracing::{info, instrument};

#[instrument]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    info!("Initializing Kubernetes client");
    let client = Client::try_default().await?;

    logging::init();

    let controllers = match NuopMode::from_env() {
        NuopMode::Init => vec![],
        NuopMode::Manager => {
            info!("Starting Manager mode...");
            vec![tokio::spawn(manager_controller(client.clone()))]
        }
        _ => vec![], // NuopMode::Managed => {
                     //     info!("Starting Managed mode...");
                     //     get_managed_controllers(
                     //         &client,
                     //         find_mappings(&get_mapping_path()).as_slice(),
                     //         find_scripts(&get_script_path()).as_slice(),
                     //     )
                     // }
                     // NuopMode::Standard => {
                     //     info!("Starting Standard mode...");
                     //     get_standard_controllers(&client, find_scripts(&get_script_path()).as_slice())
                     // }
    };

    try_join_all(controllers).await?;
    Ok(())
}
