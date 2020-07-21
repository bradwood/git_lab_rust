# git_lab_cli

[![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
[![Crates.io](https://img.shields.io/crates/v/git_lab_cli)](https://crates.io/crates/git_lab_cli)
[![Crates.io](https://img.shields.io/crates/d/git_lab_cli)](https://crates.io/crates/git_lab_cli)
[![musl binaries](https://img.shields.io/badge/musl%20binary-download-brightgreen)](https://gitlab.com/bradwood/git-lab-rust/-/releases)

_ALPHA_ - what is here works, but functionality is still under active development.

This is a cli tool that adds the `lab` command to `git` to enable interaction with a GitLab server.

## Functionality

The tool is designed to work as a custom command to the vanilla `git` cli command.

### Current functions

 * `init` -- initialise credentials against a remote GitLab server
 * `project` -- interact with GitLab projects
    * `project create` -- create project
    * `project attach` -- associate a local repo with a project
    * `project (open|view|browse)` -- open project's URL in browser
    * `project (show|info|get)` -- show details about a project
 * `issue` -- interact with issues
    * `issue create` -- create issue (either entirely via cli-passed parameters, or
       interactively, by prompting the user for the inputs needed)
    * `issue assign` -- assign issue
    * `issue (open|view|browse)` -- open issue's URL in browser
    * `issue (show|info|get)` -- show details about a issue
    * `issue list` -- get list of issues
    * `issue close` -- close issue
    * `issue reopen` -- reopen issue
    * `issue lock` -- lock discussions on issue
    * `issue unlock` -- unlock discussions on issue
 * `mr` -- interact with merge requests
    * `mr create` -- create merge request (either entirely via cli-passed parameters, or
       interactively, by prompting the user for the inputs needed)
    * `mr assign` -- assign merge request
    * `mr close` -- close merge request
    * `mr reopen` -- reopen merge request
    * `mr lock` -- lock discussions on merge request
    * `mr unlock` -- unlock discussions on merge request
    * `mr list` -- get list of merge requests
    * `mr (open|view|browse)` -- open merge request's URL in browser
    * `mr (show|info|get)` -- show details about a merge request
    * `mr (checkout|co)` -- checkout merge request
    * `mr wip` -- toggle `WIP:` (or `Draft:`) status of merge request

### Planned functions

 * `mr` -- interact with merge requests
    * `mr approve` -- approve merge request
    * `mr merge` -- merge merge request
 * `project list` -- get list of projects
 * `pipeline` -- interact with Gitlab CI jobs
 * `group` -- interact with Gitlab groups
 * `user` -- interact with Gitlab users
 * `labels` -- interact with Gitlab labels
 * probably others...

## Features

### Current features

 * Config stored using standard `git config` machinery
 * Locally cached Gitlab metadata to improve usability when creating gitlab objects
   interactively
 * JSON output in addition to plain text to allow for parsing with tools like `jq`
 * Terminal-based markdown rendering
 * `$EDITOR` integration on `create` commands
 * `musl` binaries available [here](https://gitlab.com/bradwood/git-lab-rust/-/releases)

### Planned features

 * support for various linux packaging tools like AUR, Deb, RPM, etc
 * non-Linux support (maybe)

## Installation

### Cargo

To install via `cargo`:

```rust
cargo install git_lab_cli
```
### Statically linked Linux binaries

Grab a tarball for these [here](https://gitlab.com/bradwood/git-lab-rust/-/releases).

## Compatibility

Supports GitLab server version 13

## Contributions

Merge requests are welcome. Please raise a merge request on [GitLab](https://gitlab.com/bradwood/git-lab-rust), not GitHub.

License: MIT
