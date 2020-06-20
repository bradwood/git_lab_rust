# git_lab_cli

[![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
[![Crates.io](https://img.shields.io/crates/v/git_lab_cli)](https://crates.io/crates/git_lab_cli)
[![Crates.io](https://img.shields.io/crates/d/git_lab_cli)](https://crates.io/crates/git_lab_cli)

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
    * `issue (open|view|browse)` -- open issue's URL in browser
    * `issue (show|info|get)` -- show details about a issue

### Planned functions

 * `project list` -- get list of projects
 * `issue list` -- get list of issues
 * `merge-request` -- interact with merge requests
 * `pipeline` -- interact with Gitlab CI jobs
 * `group` -- interact with Gitlab groups
 * `user` -- interact with Gitlab users
 * probably others like `labels`, etc..

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
