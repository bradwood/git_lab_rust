/// All subcommands need to implement this method, which generates the cli-config that Clap requires
/// for the subcommand
pub trait SubCommand {
    fn gen_clap_command(&self) -> clap::App;
    fn run(&self);
}

/// A struct which holds a vector of heap-allocated Boxes of trait objects all of which must
/// implement the GenSubCommand trait, but other than that, can be of any type.
pub struct ClapCommands {
    pub commands: Vec<Box<dyn SubCommand>>,
}

/// A "method" on the ClapCommands type definition that generates a vector of Clap::Apps that can
/// be passed into Clap's `.subcommands()` method in order to generate the full CLI
impl ClapCommands {
    pub fn generate(&self) -> Vec<clap::App> {

        let mut v: Vec<clap::App> = Vec::new();

        for command in self.commands.iter() {
            v.push(command.gen_clap_command());
        }
        v
    }
}
