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

### Planned functions

* `issue` -- interact with issues
* `merge-request` -- interact with merge requests
* `pipeline` -- interact with Gitlab CI jobs
* probably others

## Features

### Current features

* Config stored using standard `git config` machinery
* JSON output in addition to plain text to allow for parsing with tools like `jq`.

### Planned features

* `$EDITOR` integration
* Terminal-based markdown rendering

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
