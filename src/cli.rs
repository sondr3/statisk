use std::{io, path::PathBuf};

use clap::{Command, Parser, Subcommand, ValueHint};
use clap_complete::{Generator, Shell, generate};

#[derive(Debug, Parser)]
#[command(version, about, author)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Options {
    #[arg(long, short)]
    pub verbose: bool,
    /// Directory to run in
    #[arg(value_hint = ValueHint::DirPath, value_name = "dir", global = true)]
    pub dir: Option<PathBuf>,
    /// Configuration management
    #[command(subcommand)]
    pub cmd: Option<Cmds>,
}

#[derive(Subcommand, Debug)]
pub enum Cmds {
    /// start the dev loop
    Dev,
    /// build for production
    Build,
    /// start a local server
    Serve,
    /// Print completions
    Completion {
        /// Shell to generate completion for
        #[clap(value_enum)]
        shell: Shell,
    },
}

pub fn print_completion<G: Generator>(generator: G, app: &mut Command) {
    generate(
        generator,
        app,
        app.get_name().to_string(),
        &mut io::stdout(),
    );
}
