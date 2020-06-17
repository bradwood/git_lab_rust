# git_lab_cli

[![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
[![coverage report](https://gitlab.com/bradwood/git-lab-rust/badges/master/coverage.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)

_ALPHA_ - what is here works, but functionality is still under active development.

This is a cli tool that adds the `lab` command to `git` to enable interaction with a GitLab server.

## Functionality

The tool is designed to work as a custom command to the vanilla `git` cli command.

### Current functions

 * `init` -- initialise credentials aganst a remote GitLab server
 * `project` -- interact with GitLab projects
    * `project create` -- create project
    * `project attach` -- associate a local repo with a project
    * `project (open|view|browse)` -- open project's URL in browser
    * `project (show|info|get)` -- show details about a project

 * `issue` -- interact with issues
    * `issue create` -- create issue
    * `issue (open|view|browse)` -- open issue's URL in browser
    * `issue (show|info|get)` -- show details about a issue

### Planned functions

 * `project` -- interact with GitLab projects
    * `project list` -- get list of projects
 * `issue` -- interact with issues
    * `issue list` -- get list of issues
 * `merge-request` -- interact with merge requests
    * `merge-request create` -- create merge request
    * `merge-request list` -- get list of merge requests
    * `merge-request (open|view|browse)` -- open merge-request's URL in browser
    * `merge-request (show|info|get)` -- show details about a merge-request
    * `merge-request approve` -- approve merge request
    * `merge-request merge` -- merge merge request

 * `pipeline` -- interact with Gitlab CI jobs
 * `group` -- interact with Gitlab groups
 * `user` -- interact with Gitlab users
 * probably others like `labels`, etc..

## Features

### Current features

 * Config stored using standard `git config` machinery
 * JSON output in addition to plain text to allow for parsing with tools like `jq`.
 * Terminal-based markdown rendering

### Planned features

 * `$EDITOR` integration on `create` commands
 * `musl` and `glibc` binary packages
 * support for various linux packaging tools like AUR, Deb, RPM, etc
 * non-Linux support (maybe)

## Installation

For now, this is only available via `cargo` while under development.

```rust
cargo install git_lab_cli
```
## Compatibility

Supports GitLab server version 13

## Contributions

Merge requests are welcome. Please raise a merge request on [GitLab](https://gitlab.com/bradwood/git-lab-rust), not GitHub.

License: MIT
