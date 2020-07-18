//! This module inplements a thin shim over the 3rd party GitLab API where needed.
//!
//! As a result we don't unit-test the shim code, and therefore try to keep as much logic out of
//! this as possible, in order to keep the shim as thin as possible.
//!
//! Where possible it will just re-export types from the 3rd party library when nothing special
//! needs to be abstracted.
use anyhow::{Context, Result, anyhow};

pub use gitlab::Gitlab as Client;
pub use gitlab::api as api;
pub use gitlab::api::Query;
pub use gitlab::api::projects::Project;
pub use gitlab::api::projects::ProjectBuilder;
pub use gitlab::api::projects::CreateProject;
pub use gitlab::api::projects::CreateProjectBuilder;
pub use gitlab::api::projects::issues::Issue;
pub use gitlab::api::projects::issues::IssueBuilder;
pub use gitlab::api::projects::issues::Issues;
pub use gitlab::api::projects::issues::IssuesBuilder;
pub use gitlab::api::projects::issues::EditIssue;
pub use gitlab::api::projects::issues::EditIssueBuilder;
pub use gitlab::api::projects::issues::CreateIssue;
pub use gitlab::api::projects::issues::CreateIssueBuilder;
pub use gitlab::api::projects::issues::IssueState;
pub use gitlab::api::projects::issues::IssueStateEvent;
pub use gitlab::api::projects::issues::IssueScope;
pub use gitlab::api::projects::issues::IssueWeight;
pub use gitlab::api::projects::issues::IssueOrderBy;

pub use gitlab::api::projects::merge_requests::MergeRequest;
pub use gitlab::api::projects::merge_requests::MergeRequestBuilder;
pub use gitlab::api::projects::merge_requests::MergeRequests;
pub use gitlab::api::projects::merge_requests::MergeRequestsBuilder;
pub use gitlab::api::projects::merge_requests::EditMergeRequest;
pub use gitlab::api::projects::merge_requests::EditMergeRequestBuilder;
pub use gitlab::api::projects::merge_requests::CreateMergeRequest;
pub use gitlab::api::projects::merge_requests::CreateMergeRequestBuilder;
pub use gitlab::api::projects::merge_requests::MergeRequestState;
pub use gitlab::api::projects::merge_requests::MergeRequestStateEvent;
pub use gitlab::api::projects::merge_requests::MergeRequestOrderBy;
pub use gitlab::api::projects::merge_requests::MergeRequestScope;

pub use gitlab::api::projects::labels::Labels;
pub use gitlab::api::projects::labels::LabelsBuilder;

pub use gitlab::api::projects::members::ProjectMembers;
pub use gitlab::api::projects::members::ProjectMembersBuilder;


pub use gitlab::api::projects::repository::branches::CreateBranch;
pub use gitlab::api::projects::repository::branches::CreateBranchBuilder;
pub use gitlab::api::projects::repository::branches::Branch;
pub use gitlab::api::projects::repository::branches::BranchBuilder;

pub use gitlab::api::common::EnableState;
pub use gitlab::api::common::VisibilityLevel;
pub use gitlab::api::common::SortOrder;

pub use gitlab::api::projects::AutoDevOpsDeployStrategy;
pub use gitlab::api::projects::FeatureAccessLevel;
pub use gitlab::api::projects::FeatureAccessLevelPublic;
pub use gitlab::api::projects::MergeMethod;
pub use gitlab::api::projects::BuildGitStrategy;


use crate::config::Config;

/// Misc converter functions used to convert string args to Gitlab types
pub mod converter {
    use super::*;

    pub fn mr_order_by_from_str(s: &str) -> Result<MergeRequestOrderBy> {
        match s {
            "created_on" => Ok(MergeRequestOrderBy::CreatedAt),
            "updated_on" => Ok(MergeRequestOrderBy::UpdatedAt),
            _ => Err(anyhow!("Incorrect merge request list ordering"))
        }
    }

    pub fn mr_scope_from_str(s: &str) -> Result<MergeRequestScope> {
        match s {
            "created_by_me" => Ok(MergeRequestScope::CreatedByMe),
            "assigned_to_me" => Ok(MergeRequestScope::AssignedToMe),
            "all" => Ok(MergeRequestScope::All),
            _ => Err(anyhow!("Incorrect merge request scope"))
        }
    }

    pub fn mr_state_from_str(s: &str) -> Result<MergeRequestState> {
        match s {
            "opened" => Ok(MergeRequestState::Opened),
            "closed" => Ok(MergeRequestState::Closed),
            "locked" => Ok(MergeRequestState::Locked),
            "merged" => Ok(MergeRequestState::Merged),
            _ => Err(anyhow!("Incorrect issue state"))
        }
    }

    pub fn issue_order_by_from_str(s: &str) -> Result<IssueOrderBy> {
        match s {
            "created_on" => Ok(IssueOrderBy::CreatedAt),
            "updated_on" => Ok(IssueOrderBy::UpdatedAt),
            "priority" => Ok(IssueOrderBy::Priority),
            "due_date" => Ok(IssueOrderBy::DueDate),
            "relative_position" => Ok(IssueOrderBy::RelativePosition),
            "label_priority" => Ok(IssueOrderBy::LabelPriority),
            "milestone_date" => Ok(IssueOrderBy::MilestoneDue),
            "popularity" => Ok(IssueOrderBy::Popularity),
            "weight" => Ok(IssueOrderBy::WeightFields),
            _ => Err(anyhow!("Incorrect issue list ordering"))
        }
    }

    pub fn issue_scope_from_str(s: &str) -> Result<IssueScope> {
        match s {
            "created_by_me" => Ok(IssueScope::CreatedByMe),
            "assigned_to_me" => Ok(IssueScope::AssignedToMe),
            "all" => Ok(IssueScope::All),
            _ => Err(anyhow!("Incorrect issue scope"))
        }
    }

    pub fn issue_state_from_str(s: &str) -> Result<IssueState> {
        match s {
            "opened" => Ok(IssueState::Opened),
            "closed" => Ok(IssueState::Closed),
            _ => Err(anyhow!("Incorrect issue state"))
        }
    }

    pub fn auto_devops_deploy_strategy_from_str(s: &str) -> Result<AutoDevOpsDeployStrategy> {
        match s {
            "continuous" => Ok(AutoDevOpsDeployStrategy::Continuous),
            "manual" => Ok(AutoDevOpsDeployStrategy::Manual),
            "timed_incremental" => Ok(AutoDevOpsDeployStrategy::TimedIncremental),
            _ => Err(anyhow!("Incorrect deployment strategy"))
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
            _ => Err(anyhow!("Incorrect public feature access level"))
        }
    }
    pub fn feature_access_level_from_str(s: &str) -> Result<FeatureAccessLevel> {
        match s {
            "disabled" => Ok(FeatureAccessLevel::Disabled),
            "private" => Ok(FeatureAccessLevel::Private),
            "enabled" => Ok(FeatureAccessLevel::Enabled),
            _ => Err(anyhow!("Incorrect feature access level"))
        }
    }
}

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

#[cfg(test)]
mod gitlab_converter_unit_tests {
    use anyhow::Result;
    use rstest::*;
    use super::*;
    use super::converter::*;

    #[rstest(
        s, t, f,
        case("created_on", MergeRequestOrderBy::CreatedAt, &mr_order_by_from_str),
        case("updated_on", MergeRequestOrderBy::UpdatedAt, &mr_order_by_from_str),

        case("created_by_me", MergeRequestScope::CreatedByMe, &mr_scope_from_str),
        case("assigned_to_me", MergeRequestScope::AssignedToMe, &mr_scope_from_str),
        case("all", MergeRequestScope::All, &mr_scope_from_str),

        case("opened", MergeRequestState::Opened, &mr_state_from_str),
        case("closed", MergeRequestState::Closed, &mr_state_from_str),
        case("locked", MergeRequestState::Locked, &mr_state_from_str),
        case("merged", MergeRequestState::Merged, &mr_state_from_str),

        case("created_on", IssueOrderBy::CreatedAt, &issue_order_by_from_str),
        case("updated_on", IssueOrderBy::UpdatedAt, &issue_order_by_from_str),
        case("priority", IssueOrderBy::Priority, &issue_order_by_from_str),
        case("due_date", IssueOrderBy::DueDate, &issue_order_by_from_str),
        case("relative_position", IssueOrderBy::RelativePosition, &issue_order_by_from_str),
        case("label_priority", IssueOrderBy::LabelPriority, &issue_order_by_from_str),
        case("milestone_date", IssueOrderBy::MilestoneDue, &issue_order_by_from_str),
        case("popularity", IssueOrderBy::Popularity, &issue_order_by_from_str),
        case("weight", IssueOrderBy::WeightFields, &issue_order_by_from_str),

        case("created_by_me", IssueScope::CreatedByMe, &issue_scope_from_str),
        case("assigned_to_me", IssueScope::AssignedToMe, &issue_scope_from_str),
        case("all", IssueScope::All, &issue_scope_from_str),

        case("opened", IssueState::Opened, &issue_state_from_str),
        case("closed", IssueState::Closed, &issue_state_from_str),

        case("continuous", AutoDevOpsDeployStrategy::Continuous, &auto_devops_deploy_strategy_from_str),
        case("manual", AutoDevOpsDeployStrategy::Manual, &auto_devops_deploy_strategy_from_str),
        case("timed_incremental", AutoDevOpsDeployStrategy::TimedIncremental, &auto_devops_deploy_strategy_from_str),

        case("enabled", EnableState::Enabled, &enable_state_from_str),
        case("disabled", EnableState::Disabled, &enable_state_from_str),

        case("fetch", BuildGitStrategy::Fetch, &pipeline_git_strategy_from_str),
        case("clone", BuildGitStrategy::Clone, &pipeline_git_strategy_from_str),

        case("merge", MergeMethod::Merge, &merge_method_from_str),
        case("rebase-merge", MergeMethod::RebaseMerge, &merge_method_from_str),
        case("fast-forward", MergeMethod::FastForward, &merge_method_from_str),

        case("public", VisibilityLevel::Public, &visibility_level_from_str),
        case("internal", VisibilityLevel::Internal, &visibility_level_from_str),
        case("private", VisibilityLevel::Private, &visibility_level_from_str),

        case("disabled", FeatureAccessLevelPublic::Disabled, &feature_access_level_public_from_str),
        case("private", FeatureAccessLevelPublic::Private, &feature_access_level_public_from_str),
        case("enabled", FeatureAccessLevelPublic::Enabled, &feature_access_level_public_from_str),
        case("public", FeatureAccessLevelPublic::Public, &feature_access_level_public_from_str),

        case("disabled", FeatureAccessLevel::Disabled, &feature_access_level_from_str),
        case("private", FeatureAccessLevel::Private, &feature_access_level_from_str),
        case("enabled", FeatureAccessLevel::Enabled, &feature_access_level_from_str),
    )]
    fn test_gitlab_converter_from_str_ok<T>(s: &str, t: T, f: &dyn Fn(&str) -> Result<T>)
    where T: Eq + std::fmt::Debug
    {
        assert_eq!(f(s).unwrap(), t)
    }

    #[rstest(
        s,  f,
        case("blah", &issue_order_by_from_str),
        case("blah", &issue_scope_from_str),
        case("blah", &issue_state_from_str),
        case("blah", &auto_devops_deploy_strategy_from_str),
        case("blah", &enable_state_from_str),
        case("blah", &pipeline_git_strategy_from_str),
        case("blah", &merge_method_from_str),
        case("blah", &visibility_level_from_str),
        case("blah", &feature_access_level_public_from_str),
        case("blah", &feature_access_level_from_str),
    )]
    fn test_gitlab_converter_from_str_err<T>(s: &str,  f: &dyn Fn(&str) -> Result<T>)
    where T: Eq + std::fmt::Debug
    {
        assert!(f(s).is_err())
    }
}
