//! This module inplements a thin shim over the 3rd party GitLab API. As a result we don't
//! unit-test the shim code, and therefore try to keep as much logic out of this as possible, in
//! order to keep the shim as thin as possible.
//!
//! Where possible it will just re-export types from the 3rd party library when nothing special
//! needs to be abstracted. However, to aid in mocking/testing and where some abstraction is
//! needed, the methods here will fulfil that function.
use anyhow::{Context, Result};

pub use gitlab::{CreateProjectParams, Project};

use gitlab::Gitlab as TPGitLab;
use gitlab::GitlabBuilder as TPGitLabBuilder;

use crate::config::Config;

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
