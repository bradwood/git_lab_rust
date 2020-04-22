//! This module inplements a shim over the 3rd party GitLab API.
//!
//! Where possible it will just re-export types from the 3rd party library when nothing special
//! needs to be abstracted. However, to aid in mocking/testing and where some abstraction is
//! needed, the methods here will fulfil that function.
use anyhow::{Context, Result};

// re-export these 3rd party objects for use outside this module
pub use gitlab::{CreateProjectParams, Project};

// privately use these 3rd party objects within this module
use gitlab::Gitlab as TPGitLab;
use gitlab::GitlabBuilder as TPGitLabBuilder;

use crate::config::Config;

/// Holds the GitLab wrapper that isolates the 3rd party GitLab lib
pub struct GitLab {
    gl: TPGitLab,
}

/// Defines the methods that need to be implemented by the GitLab wrapper/shim
pub trait IfGitLab {
    /// Create a connected instance of GitLab
    fn new(config: &Config) -> Result<Box<Self>>; // Use a Box as Self is an unknown size and must go on the heap

    /// Shim over 3rd party create_project() method
    fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> Result<Project>;
}

impl IfGitLab for GitLab {
    /// Return a connected gitlab object
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
                    format!(
                        "Failed to make insecure (http) connection to {}\n
                    Try running `git lab init` to ensure all connection parameters are correct.",
                        host
                    )
                })?,
            _ => TPGitLabBuilder::new(host, token).build().with_context(|| {
                format!(
                    "Failed to make secure (https) connection to {}\n
                    Try running `git lab init` to ensure all connection parameters are correct.",
                    host
                )
            })?,
        };
        Ok(Box::new(GitLab { gl }))
    }

    fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> Result<Project> {
        self.gl.create_project(name, path, params).context("Failed to create a project.")
    }
}
