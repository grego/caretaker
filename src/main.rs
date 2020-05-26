pub mod command;
pub mod watch;
pub mod error;

use ansi_term::Style;
use error::Error;
use structopt::StructOpt;
use watch::watch;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
/// A simple, configurable filesystem watcher
struct Opt {
    /// File to read what to watch from
    #[structopt(short, long, default_value = ".watch.toml")]
    watch_config: String,
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
command = \"echo \\\"hello world\\\"\"

# Repeat this to watch multiple paths";

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let bold = Style::new().bold();
    match opt.cmd {
        Some(Cmd::Init) => {
            let config = &opt.watch_config;
            if std::fs::metadata(config).is_ok() {
                println!("{} already exists, exiting", bold.paint(config))
            } else {
                std::fs::write(config, DUMMY)?;
                println!("{} created!", bold.paint(config))
            }
        }
        _ => {
            let config = std::fs::read(&opt.watch_config)?;
            match toml::from_slice(&config) {
                Ok(config) => {
                    watch(config)?;
                }
                Err(e) => {
                    println!("Unable to parse {}: {}", bold.paint(&opt.watch_config), e);
                }
            };
        }
    }

    Ok(())
}
