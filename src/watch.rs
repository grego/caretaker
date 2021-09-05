use crate::Error;

use ansi_term::Style;
use crossbeam_channel::unbounded;
use glob::Pattern;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use serde::Deserialize;

use std::convert::Infallible;
use std::path::{is_separator, Path};
use std::process::Command;

#[cfg(target_family = "unix")]
pub(crate) static SHELL: &str = "sh";
#[cfg(target_family = "unix")]
pub(crate) static ARGUMENT: &str = "-c";
#[cfg(target_family = "windows")]
pub(crate) static SHELL: &str = "cmd";
#[cfg(target_family = "windows")]
pub(crate) static ARGUMENT: &str = "/c";

/// One path to watch
#[derive(Deserialize)]
pub struct Watch {
    /// A name of the action to do on the path change.
    #[serde(default)]
    pub name: String,
    /// The path to watch for change.
    pub path: String,
    /// The command to execute on path change.
    pub command: String,
}

/// The config file of Caretaker
#[derive(Deserialize)]
pub struct Config {
    /// A list of paths and commands to watch.
    pub watch: Vec<Watch>,
}

/// Watch the paths specified in the config, executing the commands using the provided shell.
pub fn watch(config: Config, shell: &str, arg: &str) -> Result<Infallible, Error> {
    use notify::event::{EventKind::*, *};

    let len = config.watch.len();
    let mut watchers = Vec::new();
    let (tx, rx) = unbounded();
    let bold = Style::new().bold();

    let is_glob = |c| c == '*' || c == '?' || c == '[';
    let matches =
        |pattern: &Pattern, path: &Path| path.to_str().map(|s| pattern.matches(s)).unwrap_or(false);

    for Watch {
        name,
        mut path,
        command,
    } in config.watch.into_iter()
    {
        let tx = tx.clone();
        let glob = if path.contains(is_glob) {
            let mut last = 0;
            for (index, matched) in path.match_indices(is_separator) {
                if path[last..index].contains(is_glob) {
                    break;
                };
                last = index + matched.len();
            }
            let pattern = Pattern::new(
                &Path::new(&path[0..last])
                    .canonicalize()?
                    .join(&path[last..])
                    .to_string_lossy(),
            )?;
            path.truncate(last);
            Some(pattern)
        } else {
            None
        };

        let mut cmd = Command::new(shell);
        cmd.args(&[arg, &command]);
        let command = Mutex::new(cmd);

        let mut watcher = recommended_watcher(move |res: Result<Event, _>| match res {
            Ok(Event { kind, paths, .. }) => match kind {
                Access(AccessKind::Close(AccessMode::Write))
                | Modify(ModifyKind::Name(RenameMode::To))
                | Remove(_) => {
                    for path in &paths {
                        if !glob
                            .as_ref()
                            .map(|pattern| matches(pattern, path))
                            .unwrap_or(true)
                        {
                            return;
                        }

                        println!("{:?} changed, running {}", path, bold.paint(&name));
                        if let Err(e) = command
                            .lock()
                            .env("EVENT_PATH", path)
                            .status()
                            .map_err(|e| e.into())
                        {
                            tx.send(e).unwrap();
                        }
                    }
                }
                _ => {}
            },
            Err(e) => {
                tx.send(e).unwrap();
            }
        })?;
        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|source| Error::PathWatch { source, path })?;
        watchers.push(watcher);
    }

    println!(
        "Watching {} path{} for changes...",
        bold.paint(len.to_string()),
        if len == 1 { "" } else { "s" }
    );
    rx.recv().map_err(|e| e.into()).and_then(|e| Err(e.into()))
}
