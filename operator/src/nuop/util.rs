use core::fmt::{self, Display, Formatter};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::{Error, api::ResourceExt, core::ErrorResponse};

pub const NUOP_MODE: &str = "NUOP_MODE";
pub enum NuopMode {
    Init,
    Manager,
    Managed,
    Standard,
}

impl NuopMode {
    pub fn from_env() -> Self {
        match std::env::var(NUOP_MODE)
            .map(|v| v.to_lowercase())
            .as_deref()
        {
            Ok("manager") => Self::Manager, // run in operator mode, manage instances
            Ok("managed") => Self::Managed, // run in managed instance mode
            _ => Self::Standard,            // run in unmanaged mode
        }
    }
}

impl Display for NuopMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mode_str = match self {
            NuopMode::Init => "init",
            NuopMode::Manager => "manager",
            NuopMode::Managed => "managed",
            NuopMode::Standard => "standard",
        };
        write!(f, "{mode_str}")
    }
}

pub(crate) fn generate_owner_reference<T: kube::Resource<DynamicType = ()>>(
    resource: &T,
) -> Option<OwnerReference> {
    Some(OwnerReference {
        api_version: T::api_version(&()).to_string(),
        kind: T::kind(&()).to_string(),
        name: resource.name_any(),
        uid: resource.meta().uid.clone()?,
        controller: Some(true),
        block_owner_deletion: Some(true),
    })
}

pub fn to_kube_error(reason: &str, message: &str, code: u16) -> Error {
    Error::Api(ErrorResponse {
        status: "Failure".to_string(),
        message: message.to_string(),
        reason: reason.to_string(),
        code,
    })
}
