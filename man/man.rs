use man::prelude::*;

fn man() -> String {
    Manual::new("git-lab")
        .about("A custom git command for interacting with a GitLab server")
        .author(Author::new("Bradley Wood").email("git@bradleywood.com"))
        .arg(
            Arg::new("COMMAND"),
        )
        .arg(
            Arg::new("[SUBCOMMAND]")
        )
        .arg(
            Arg::new("[OPTIONS...]")
        )
        .flag(
            Flag::new()
                .short("-v")
                .help("Enable verbose mode. Multiple v's increases verbosity."),
        )
        .example(
            Example::new()
                .text("Get top level help")
                .command("git lab help")
                .output("Prints all top level commands, options and flags.")
            )
        .example(
            Example::new()
                .text("Get help on `init` command")
                .command("git lab init --help")
                .output("Prints commands, options and flags for the `init` command.")
            )
        .example(
            Example::new()
                .text("Attach local repo to remote GitLab project")
                .command("git lab project attach")
                .output("Updates repo's `.git/config` with GitLab project metadata.")
            )
        .example(
            Example::new()
                .text("Open project in default browser")
                .command("git lab project browse")
            )
        .example(
            Example::new()
                .text("Create a project issue interactively")
                .command("git lab issue create")
                .output("Prompts the user through entering each GitLab issue field and then creates the issue on the server.")
            )
        .example(
            Example::new()
                .text("Create a project issue from the cli")
                .command("git lab issue create --desc 'This is the *issue* description' 'Title of issue'")
                .output("Creates the issue with the passed parameters")
            )
        .custom(
            Section::new("HELP")
            .paragraph("Pass the `help` command to get top-level help and a command listing.")
            .paragraph("Pass the `--help` flag after any command to get further information about that command.")
        )
        .custom(
            Section::new("BUGS")
                .paragraph("Please report bugs at https://gitlab.com/bradwood/git-lab-rust")
        )
        .custom(
            Section::new("LICENCE")
                .paragraph("MIT")
        )
        .render()
}

