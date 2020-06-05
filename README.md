# git_lab_cli

[![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
[![coverage report](https://gitlab.com/bradwood/git-lab-rust/badges/master/coverage.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)

__WORK IN PROGRESS__ use at your own risk!

This is a cli tool that adds the `lab` command to `git` to enable interaction with a GitLab server.

## Features

The tool is designed to work as a custom command to the vanilla `git` cli command. Current
functions include:
* `init` -- initialise credentials aganst a remote GitLab server
* `project` -- interact with GitLab projects
    * `project create` -- create project
    * `project attach` -- associate a local repo with a project -- still WIP

It has the following features:
* config stored using standard `git config` machinery
* JSON output in addition to plain text to allow for parsing with tools like `jq`.

## Installation

For now, this is only available via `cargo` while under development.

```rust
cargo install git_lab_cli
```

## Contributions

Merge requests are welcome. Please raise a merge request on [GitLab](https://gitlab.com/bradwood/git-lab-rust), not GitHub.

License: MIT
