//! This module inplements a shim over the 3rd party GitLab API in used. The shim's current main purpose is
//! to enable dependency injection for testing.
use anyhow::{Result,Context};
pub use gitlab::{CreateProjectParams, GitlabBuilder};

use crate::config::Config;

/// Return a connected gitlab object
pub fn gen_gitlab(config: Config) -> Result<GitLab> {
    let host = &config.host
        .context("GitLab host not set. Run `git lab init`.")?;
    let token = &config.token
        .context("GitLab token not set. Run `git lab init`.")?;

    let _gitlab = match config.tls {
        Some(x) if !x => GitlabBuilder::new(host, token)
            .insecure()
            .build()
            .with_context(|| {
                format!(
                    "Failed to make insecure (http) connection to {}\n
Try running `git lab init` to ensure all connection parameters are correct.",
                    host
                )
            })?,
        _ => GitlabBuilder::new(host, token)
            .build()
            .with_context(|| {
                format!(
                    "Failed to make secure (https) connection to {}\n
Try running `git lab init` to ensure all connection parameters are correct.",
                    host
                )
            })?,
    };
    Ok(GitLab{})
}

/// Holds the GitLab wrapper that isolates the 3rd party GitLab lib
pub struct GitLab {}

// Defines the methods that need to be implemented by the GitLab wrapper/shim
pub trait GitLabShim {
    /// Shim over 3rd party create_project() method
    fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> Result<u64>;
}

impl GitLabShim for GitLab {
    fn create_project<N: AsRef<str>, P: AsRef<str>>(
        &self,
        name: N,
        path: Option<P>,
        params: Option<CreateProjectParams>,
    ) -> Result<u64> {
        // body to go here
        Ok(666)
    }
}
