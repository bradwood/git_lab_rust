//! This module inplements a thin shim over the 3rd party GitLab API. As a result we don't
//! unit-test the shim code, and therefore try to keep as much logic out of this as possible, in
//! order to keep the shim as thin as possible.
//!
//! Where possible it will just re-export types from the 3rd party library when nothing special
//! needs to be abstracted. However, to aid in mocking/testing and where some abstraction is
//! needed, the methods here will fulfil that function.

use anyhow::{Context, Result, anyhow};

pub use gitlab::Project;

pub use gitlab::Gitlab as Client;
pub use gitlab::api::Query;
pub use gitlab::api::projects::CreateProject;

pub use gitlab::api::common::EnableState;
pub use gitlab::api::common::VisibilityLevel;
pub use gitlab::api::projects::AutoDevOpsDeployStrategy;
pub use gitlab::api::projects::FeatureAccessLevel;
pub use gitlab::api::projects::FeatureAccessLevelPublic;
pub use gitlab::api::projects::MergeMethod;
pub use gitlab::api::projects::BuildGitStrategy;

use crate::config::Config;

/// Misc converter functions used to convert string args to Gitlab types
pub mod converter {
    use super::*;

    pub fn auto_devops_deploy_strategy_from_str(s: &str) -> Result<AutoDevOpsDeployStrategy> {
        match s {
            "continuous" => Ok(AutoDevOpsDeployStrategy::Continuous),
            "manual" => Ok(AutoDevOpsDeployStrategy::Manual),
            "timed_incremental" => Ok(AutoDevOpsDeployStrategy::TimedIncremental),
            _ => Err(anyhow!("Incorrect state"))
        }
    }

    pub fn enable_state_from_str(s: &str) -> Result<EnableState> {
        match s {
            "enabled" => Ok(EnableState::Enabled),
            "disabled" => Ok(EnableState::Disabled),
            _ => Err(anyhow!("Incorrect state"))
        }
    }

    pub fn pipeline_git_strategy_from_str(s: &str) -> Result<BuildGitStrategy> {
        match s {
            "fetch" => Ok(BuildGitStrategy::Fetch),
            "clone" => Ok(BuildGitStrategy::Clone),
            _ => Err(anyhow!("Incorrect git strategy"))
        }
    }

    pub fn merge_method_from_str(s: &str) -> Result<MergeMethod> {
        match s {
            "merge" => Ok(MergeMethod::Merge),
            "rebase-merge" => Ok(MergeMethod::RebaseMerge),
            "fast-forward" => Ok(MergeMethod::FastForward),
            _ => Err(anyhow!("Incorrect merge method"))
        }
    }

    pub fn visibility_level_from_str(s: &str) -> Result<VisibilityLevel> {
        match s {
            "public" => Ok(VisibilityLevel::Public),
            "internal" => Ok(VisibilityLevel::Internal),
            "private" => Ok(VisibilityLevel::Private),
            _ => Err(anyhow!("Incorrect visibility level"))
        }
    }

    pub fn feature_access_level_public_from_str(s: &str) -> Result<FeatureAccessLevelPublic> {
        match s {
            "disabled" => Ok(FeatureAccessLevelPublic::Disabled),
            "private" => Ok(FeatureAccessLevelPublic::Private),
            "enabled" => Ok(FeatureAccessLevelPublic::Enabled),
            "public" => Ok(FeatureAccessLevelPublic::Public),
            _ => Err(anyhow!("Incorrect visibility level"))
        }
    }
    pub fn feature_access_level_from_str(s: &str) -> Result<FeatureAccessLevel> {
        match s {
            "disabled" => Ok(FeatureAccessLevel::Disabled),
            "private" => Ok(FeatureAccessLevel::Private),
            "enabled" => Ok(FeatureAccessLevel::Enabled),
            _ => Err(anyhow!("Incorrect Feature Access level"))
        }
    }
}
// /// Holds the GitLab wrapper that isolates the 3rd party GitLab lib
// pub struct GitLabClient {
//     client: TPGitLab,
// }

// pub trait IfGitLabNew {
//     /// Create a connected instance of GitLab
//     fn new(config: &Config) -> Result<Box<Self>>;
// }

// pub trait IfGitLabCreateProject {
//     /// Shim over 3rd party create_project() method
//     fn create_project<N: AsRef<str>, P: AsRef<str>>(
//         &self,
//         name: N,
//         path: Option<P>,
//         params: Option<CreateProjectParams>,
//     ) -> Result<Project>;
// }

/// Shim over 3rd party new() method
pub fn new(config: &Config) -> Result<Box<Client>> {
    let host = config
        .host
        .as_ref()
        .context("GitLab host not set. Run `git lab init`.")?;
    let token = config
        .token
        .as_ref()
        .context("GitLab token not set. Run `git lab init`.")?;

    let client = match config.tls {
        Some(tls) if !tls => Client::new_insecure(host, token)
            .with_context(|| {
                format!("Failed to make insecure (http) connection to {}", host)
            })? ,
        _ => Client::new(host, token)
            .with_context(|| format!("Failed to make secure (https) connection to {}", host))?,
    };
    Ok(Box::new(client))
}

// impl IfGitLabCreateProject for GitLabClient {
//     fn create_project<N: AsRef<str>, P: AsRef<str>>(
//         &self,
//         name: N,
//         path: Option<P>,
//         params: Option<CreateProjectParams>,
//     ) -> Result<Project> {
//         Ok(self.client.create_project(&name, path, params)?)
//     }
// }
