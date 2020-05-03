//! This module inplements a thin shim over the 3rd party GitLab API. As a result we don't
//! unit-test the shim code, and therefore try to keep as much logic out of this as possible, in
//! order to keep the shim as thin as possible.
//!
//! Where possible it will just re-export types from the 3rd party library when nothing special
//! needs to be abstracted. However, to aid in mocking/testing and where some abstraction is
//! needed, the methods here will fulfil that function.

use anyhow::{Context, Result, anyhow};

pub use gitlab::{CreateProjectParams, Project};

// Third party imports are prefixed with `TP`
use gitlab::BuildGitStrategy as TPBuildGitStrategy;
use gitlab::FeatureVisibilityLevel as TPFeatureVisibilityLevel;
use gitlab::Gitlab as TPGitLab;
use gitlab::GitlabBuilder as TPGitLabBuilder;
use gitlab::MergeMethod as TPMergeMethod;
use gitlab::VisibilityLevel as TPVisibilityLevel;

use crate::config::Config;

// TODO consider moving these functions to a separate module as they are not strictly part of the
// shim
pub fn pipeline_git_strategy_from_str(s: &str) -> Result<TPBuildGitStrategy> {
    match s {
        "fetch" => Ok(TPBuildGitStrategy::Fetch),
        "clone" => Ok(TPBuildGitStrategy::Clone),
        _ => Err(anyhow!("Incorrect git strategy"))
    }
}

pub fn merge_method_from_str(s: &str) -> Result<TPMergeMethod> {
    match s {
        "merge" => Ok(TPMergeMethod::Merge),
        "rebase-merge" => Ok(TPMergeMethod::RebaseMerge),
        "fast-forward" => Ok(TPMergeMethod::FastForward),
        _ => Err(anyhow!("Incorrect merge method"))
    }
}

pub fn visibilily_level_from_str(s: &str) -> Result<TPVisibilityLevel> {
    match s {
        "public" => Ok(TPVisibilityLevel::Public),
        "internal" => Ok(TPVisibilityLevel::Internal),
        "private" => Ok(TPVisibilityLevel::Private),
        _ => Err(anyhow!("Incorrect visibility level"))
    }
}

pub fn feature_visibility_level_from_str(s: &str) -> Result<TPFeatureVisibilityLevel> {
    match s {
        "disabled" => Ok(TPFeatureVisibilityLevel::Disabled),
        "private" => Ok(TPFeatureVisibilityLevel::Private),
        "enabled" => Ok(TPFeatureVisibilityLevel::Enabled),
        "public" => Ok(TPFeatureVisibilityLevel::Public),
        _ => Err(anyhow!("Incorrect visibility level"))
    }
}

/// Holds the GitLab wrapper that isolates the 3rd party GitLab lib
pub struct GitLab {
    gl: TPGitLab,
}

pub trait IfGitLabNew {
    /// Create a connected instance of GitLab
    fn new(config: &Config) -> Result<Box<Self>>;
}

pub trait IfGitLabCreateProject {
    /// Shim over 3rd party create_project() method
    fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> Result<Project>;
}

impl IfGitLabNew for GitLab {
    /// Shim over 3rd party new() method
    fn new(config: &Config) -> Result<Box<GitLab>> {
        let host = config
            .host
            .as_ref()
            .context("GitLab host not set. Run `git lab init`.")?;
        let token = config
            .token
            .as_ref()
            .context("GitLab token not set. Run `git lab init`.")?;

        let gl = match config.tls {
            Some(tls) if !tls => TPGitLabBuilder::new(host, token)
                .insecure()
                .build()
                .with_context(|| {
                    format!("Failed to make insecure (http) connection to {}", host)
                })?,
            _ => TPGitLabBuilder::new(host, token)
                .build()
                .with_context(|| format!("Failed to make secure (https) connection to {}", host))?,
        };
        Ok(Box::new(GitLab { gl }))
    }
}

impl IfGitLabCreateProject for GitLab {
    fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> Result<Project> {
        Ok(self.gl.create_project(&name, path, params)?)
    }
}
