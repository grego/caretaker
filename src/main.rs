//! A simple tool that loads a list of paths to watch from a TOML file.
//! ```toml
//! [[watch]]
//! name = "print hello"
//! path = "src"
//! command = "echo $EVENT_PATH"
//!
//! [[watch]]
//! name = "compile sass"
//! path = "sass/*.sass"
//! command = "sassc -t compressed sass/style.scss static/style.css"
//! ```
//! On a change in the `path`, it executes the `command`. Directories are watched recursively.
//! Paths can also be specified with [globs](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).
//! Any shell command can be used, along with pipes and so on.
//! By default, the shell specified in the `$SHELL` environment variable is used to parse and execute the command.
//! Otherwise, on Unix system, it invokes the default
//! Bourne shell (`sh` command), on windows [cmd.exe](https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).
//! Additionally, each command gets the `$EVENT_PATH` environment variable, containing the path that changed.
//!
//! Using [notify](https://docs.rs/notify) crate, which provides efficient event handling
//! support for the most operating systems (apart from BSD).
#![warn(missing_docs)]
mod error;
mod watch;

pub use error::Error;
pub use watch::{watch, Config, Watch};

use watch::SHELL;

use ansi_term::Style;
use std::env;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// A simple, configurable filesystem watcher
struct Opt {
    /// File to read what to watch from
    #[structopt(short, long, default_value = ".watch.toml")]
    config: String,
    /// Shell to parse and execute the commands with
    #[structopt(short, long)]
    shell: Option<String>,
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(StructOpt)]
enum Cmd {
    /// Create a dummy .watch.toml file
    Init,
    /// Watch for changes (default)
    Watch,
}

const DUMMY: &str = "[[watch]]
# What does the command do?
name = \"print hello\"
# Where to look for changes?
path = \"src\"
# What to execute on change?
command = \"echo $EVENT_PATH\"

# Repeat this to watch multiple paths";

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let bold = Style::new().bold();
    match opt.cmd {
        Some(Cmd::Init) => {
            let config = &opt.config;
            if std::fs::metadata(config).is_ok() {
                println!("{} already exists, exiting", bold.paint(config))
            } else {
                std::fs::write(config, DUMMY)?;
                println!("{} created!", bold.paint(config))
            }
        }
        _ => {
            let config = std::fs::read(&opt.config)?;
            match toml::from_slice(&config) {
                Ok(config) => {
                    let shell = opt.shell.or_else(|| env::var("SHELL").ok());
                    watch(config, shell.as_deref().unwrap_or(SHELL))?;
                }
                Err(e) => {
                    println!("Unable to parse {}: {}", bold.paint(&opt.config), e);
                }
            };
        }
    }

    Ok(())
}
