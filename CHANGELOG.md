# Change log

This is the log of all commits by each release. Earlier commit history is a
little untidy, but it should be cleaner for newer releases.

## Version 0.15.6 released on 2020-07-23
 - fix: make proj attach include ancestors and sort cache 

## Version 0.15.5 released on 2020-07-23
 - fix: revert gitlab release description injection 

## Version 0.15.4 released on 2020-07-23
 - fix: extend artifact expiry date to 1 year 

## Version 0.15.3 released on 2020-07-23
 - fix: address another typo in relnotes.sh 

## Version 0.15.2 released on 2020-07-23
 - fix: typo in release.sh script 
 - fix: fix error in justfile changelog generation script 

## Version 0.15.1 released on 2020-07-23
 - feat: auto generate release notes from ci 
 - feat: add autogenerating CHANGELOG.md 

## Version 0.15.0 released on 2020-07-21
 - feat: add wip toggle command for mr 

## Version 0.14.0 released on 2020-07-20
 - feat: add mr assign and issue assign cmds 

## Version 0.13.1 released on 2020-07-20
 - fix: mr create: don't populate commit if it's master's head 

## Version 0.13.0 released on 2020-07-19
 - feat: add mr checkout command... 
 - fix: fix copy-pasta typo in mr open error msg 
 - feat: add mr show 
 - fix: issue show to include web_url in all cases 
 - feat: add mr list 
 - refactor: move macros to dedicated module 
 - refactor: extract username->userid mapping fn to utils 
 - feat: add mr open 

## Version 0.12.0 released on 2020-07-18
 - feat: add close/reopen/lock/unlock for MRs 
 - refactor: move ShortCmd from issue to utils module... 

## Version 0.11.0 released on 2020-07-15
 - fix: include ancestors in members project attach 
 - fix: correct help text for mr create 
 - fix: increase label caching in project attach to 80 

## Version 0.10.0 released on 2020-07-14
 - feat: add label and assignee support for mr create 
 - fix: no labels bug in create issue 
 - feat: add mr create 
 - feat: add prompt before editing issue 
 - refactor: fix up graphql client to enable multiple queries 
 - refactor: parametrise issue arg in issue builder 
 - feat: add path_with_namespace to config and project attach 
 - add defaultbranch checks to mr create 
 - add default branch to project attach 
 - add defaultbranch to config 

## Version 0.9.5 released on 2020-07-01
 - patch semver no in cargo 

## Version 0.9.4 released on 2020-07-01
 - release: 0.9.4 

## Version 0.9.3 released on 2020-07-01
 - release: 0.9.3 
 - fix lint issue in quick_edit 
 - remove unicode chars as they don't work in all terminals 
 - fix issue list json errors 
 - fix: make project owner optional as it appear within groups 

## Version 0.9.1 released on 2020-06-27
 - 0.9.1 
 - update cargo 
 - 0.9.0 
 - update readme 
 - comment out `issue status` command 
 - fix: add project_id args to various issue cmds 
 - add issue close, open, lock, unlock 

## Version 0.9.0 released on 2020-06-27
 - 0.9.0 
 - add Cargo.lock 
 - update readme 

## Version 0.8.2 released on 2020-06-26
 - 0.8.2 
 - add assignee username support to non-interactive create project 
 - add musl binary badge 

## Version 0.8.1 released on 2020-06-25
 - 0.8.1 
 - add assignees to issue show 

## Version 0.8.0 released on 2020-06-23
 - 0.8.0 
 - update readme 
 - add issue list formatting and pagination 
 - refactor issue cmds to move shared stuff to mod.rs 
 - refactor project cmds to move shared stuff to mod.rs 
 - re-export new gitlab issue structs and assoc converters 
 - add validator for checking humanised duration strings 
 - refactor main command trait objects to have `Cmd` suffix 
 - add humantime lib 

## Version 0.7.3 released on 2020-06-20
 - 0.7.3 
 - add -xe to ci script 

## Version 0.7.2 released on 2020-06-20
 - 0.7.2 
 - update gitlab-ci to use newer image and install jq 

## Version 0.7.1 released on 2020-06-20
 - 0.7.1 
 - add readme 
 - add release publisher for tags 
 - try musl again 
 - add untracked=true to ci artifact musl bin 
 - try glob for tarball artifact 
 - add debug ls -la to ci for tarball 
 - add LICENSE and create musl tarball in ci 
 - fix ci yaml 
 - add musl build to justfile and gitlab-ci 
 - update README 
 - add crates.io badges to readme.md 
 - add build.rs and man page generator 
 - 0.7.0 

## Version 0.7.0 released on 2020-06-19
 - 0.7.0 
 - add assignee support for create issue command 
 - add label support to interactive issue creation 
 - update readme 
 - update dialoguer lib 
 - add project label support to config and attach 
 - add interactive issue create [wip] 
 - style: re-order use imports in init.rs 
 - fix some clap validators from 64- to 32-bit uints 
 - add additional validators for interactive issue creation 
 - tidy up readme 
 - update readme 

## Version 0.6.0 released on 2020-06-17
 - 0.6.0 
 - update readme 
 - improve error message when GraphQL query fails 
 - add wordwrap for labels in show issue 
 - add create issue 
 - add label printing to show issue 
 - refactor project command to share issue code better 
 - add validator for yyyy-mm-dd argument 
 - add issue show command 
 - refactor localtime handling in project cmd 
 - add nix lib SIGPIPE suppression to fix shell pipes 
 - add issue open command 
 - add clap commands and args for issue command 
 - add issue command to main.rs 
 - re-export gitlab issues from third party gitlab repo 

## Version 0.5.0 released on 2020-06-13
 - 0.5.0 
 - add project show command 

## Version 0.4.1 released on 2020-06-12
 - 0.4.1 
 - add cargo bump to justfile 

## Version 0.4.0 released on 2020-06-12
 - refactor ENV var processing to not use match 
 - set env var prefix to GITLABCLI 
 - bump version and update README.md 
 - feat: add project view command 
 - refactor ENV var processing to use match 
 - fix: update config writer to not write env vars to config 
 - style: sort use imports 
 - add Project and ProjectBuiler to gitlab module 
 - style: fix layout in config module 
 - improve error message on project create command 
 - enable JSON output for project attach command 
 - add not implemented yet message on mr command 
 - update deps and add webbrowser 

## Version 0.3.0 released on 2020-06-10
 - finish unit tests for project attach 
 - finish project attach command 
 - add graphql query, schema justfile and ci support 
 - add projectid to config module 
 - update cargo dependencies 

## Version 0.2.1 released on 2020-06-05
 - bump version and update README 
 - add JSON output option to create 
 - add OutputFormat to init and init int tests 
 - add format config variable to config module 
 - add .cargo to .gitignore, remove allow-dirty crate publish 

## Version 0.2.0 released on 2020-06-02
 - make cargo push verbose 
 - update gitlab dependency to 0.1300 
 - refactor creat project back to match statement from too many ifs 
 - test: make tarp dual-pass tests work 
 - add more options to create project 
 - refactor and improve create project test coverage 
 - test: add test for `to_str` gitlab functions 
 - add ${CARGO_HOME}/bin to path 
 - bump version 
 - update README with create project info 
 - test: remove create project tests for now 
 - style: clean up layout, comments, docstrings and clap help 
 - add deprecated disable_* flags and tidy up create project 
 - use gitlab lib from git rather than cargo 
 - fix lint issue 
 - update gitlab lib and rework project cmd accordingly 
 - add basic test for create project 
 - format code and linting 
 - fix typo in function name 
 - change rustfmt to 100 char lines 
 - add clap.rs validators for project command 
 - add short clap flags for create project 
 - ditch match-based builder method calls for if-based 
 - add match-based project create implementation 
 - add config file for rustfmt 
 - use serde de to create mock project object 
 - tooling: add just test command 
 - add justfile 
 - split IfGitlab into per-method mocks and refactor *_cmd 
 - remove mockall and add chrono dev dep for manual mocks 
 - checkpoint prior to introducing mockall mock for gitlab shim 
 - add project variable to print output of created project 
 - wrap 3rd party gitlab lib in shim 
 - add project command clap setup for attach and create subcommands 
 - fix init output to echo config updated 
 - add integration tests for init subcommand 

