# git_lab_cli

[![pipeline status](https://gitlab.com/bradwood/git-lab-rust/badges/master/pipeline.svg)](https://gitlab.com/bradwood/git-lab-rust/-/commits/master)
[![Crates.io](https://img.shields.io/crates/v/git_lab_cli)](https://crates.io/crates/git_lab_cli)
[![Crates.io](https://img.shields.io/crates/d/git_lab_cli)](https://crates.io/crates/git_lab_cli)
[![musl binaries](https://img.shields.io/badge/musl%20binary-download-brightgreen)](https://gitlab.com/bradwood/git-lab-rust/-/releases)

This is a cli tool that adds the `lab` command to `git` to enable interaction with a GitLab server.

### Change Log

See [CHANGELOG.md](CHANGELOG.md) for a summary of fixes and features added to each release.

## Functionality

The tool is designed to work as a custom command to the vanilla `git` cli command. Once
installed you can invoke it with `git lab <subcommand> <options>...`.

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
    * `mr approve` -- approve merge request
    * `mr merge` -- merge merge request

### Planned functions

 * `labels` -- interact with Gitlab labels
 * `project list` -- get list of projects
 * `pipeline` -- interact with Gitlab CI jobs
 * `group` -- interact with Gitlab groups
 * `user` -- interact with Gitlab users
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
 * non-Linux support (PRs requested!)

## Installation

### Cargo

If you have the Rust toolchain installed, you can install via `cargo`:

```shell
cargo install git_lab_cli
```
### Statically linked Linux binaries

Grab a tarball for these [here](https://gitlab.com/bradwood/git-lab-rust/-/releases).
Untar the file and then copy the included files into place as shown in the below example:

```shell
cd git_lab_cli-x.y.z-x86_64-unknown-linux-musl
sudo cp git-lab /usr/local/bin/
sudo cp man/git-lab.1 /usr/local/share/man/man1/
```

## Getting started

To connect the `git-lab` cli binary to a GitLab instance you need to create a Personal Access
Token. On Gitlab.com this can be done
[here](https://gitlab.com/profile/personal_access_tokens). Copy the token to your clipboard and
then run the following from your home directory:

```shell
git lab init
```

This will prompt you though entering the required set-up parameters, one of which will require
you to paste the GitLab token copied in the previous step into the config. Your default
`.gitconfig` will then be updated with the information needed to connect `git-lab` to your
server. You can also set this config up with vanilla `git config` commands. See `git lab init
--help` for details on how to do this.

The easiest way to get started with an existing git repo is to run the following from _within_
the repo:

```shell
git lab project attach
```

This will assoicate the git repo you are working on with a server-side GitLab project by
looking at your `origin` git remote. Once this is done, you'll be able to query, create, and
modify project-specific objects like GitLab issues, merge-requests, and such like as long as
you remain within the repo's subtree.

## Compatibility

The tool tries to track GitLab.com's latest version pretty closely. Currently GitLab 13.0 and
above work but earlier versions do not.

## Contributions

Merge requests are welcome. Please raise a merge request on
[GitLab](https://gitlab.com/bradwood/git-lab-rust), not GitHub.


License: MIT
