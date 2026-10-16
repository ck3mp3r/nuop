mod controller;
mod model;
mod reconciler;
mod resources;
mod state;

pub use controller::controller as manager_controller;
pub use model::Mapping;
pub use model::NuOperator;
pub use model::Source;
use state::State;

#[cfg(test)]
#[path = "../../../test/nuop/manager/controller.rs"]
mod controller_tests;
