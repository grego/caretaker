use crate::command::Command;
use crate::Error;

use ansi_term::Style;
use crossbeam_channel::unbounded;
use glob::Pattern;
use notify::{immediate_watcher, RecursiveMode, Watcher};
use serde::Deserialize;

use std::convert::Infallible;
use std::path::{is_separator, Path};
use std::sync::Mutex;

#[derive(Deserialize)]
struct Watch {
    #[serde(default)]
    name: String,
    path: String,
    command: Mutex<Command>,
}

#[derive(Deserialize)]
pub struct Config {
    watch: Vec<Watch>,
}

pub fn watch(config: Config) -> Result<Infallible, Error> {
    use notify::event::{EventKind::*, *};

    let len = config.watch.len();
    let mut watchers = Vec::new();
    let (tx, rx) = unbounded();
    let bold = Style::new().bold();

    let is_glob = |c| c == '*' || c == '?' || c == '[';

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

        let mut watcher = immediate_watcher(move |res: Result<Event, _>| match res {
            Ok(Event { kind, paths, .. }) => match kind {
                Access(AccessKind::Close(AccessMode::Write))
                | Create(_)
                | Modify(ModifyKind::Name(RenameMode::To))
                | Remove(_) => {
                    let matches_glob = glob
                        .as_ref()
                        .map(|pattern| {
                            paths.iter().any(|path| {
                                path.to_str().map(|s| pattern.matches(s)).unwrap_or(false)
                            })
                        })
                        .unwrap_or(true);
                    if !matches_glob {
                        return;
                    }

                    println!("{:?} changed, running {}", &paths, bold.paint(&name));
                    if let Err(e) = command
                        .lock()
                        .map_err(|e| e.into())
                        .and_then(|mut c| c.status().map_err(|e| e.into()))
                    {
                        tx.send(e).unwrap();
                    }
                }
                _ => {}
            },
            Err(e) => {
                tx.send(e).unwrap();
            }
        })?;
        watcher.watch(&path, RecursiveMode::Recursive)?;
        watchers.push(watcher);
    }

    println!(
        "Watching {} path{} for changes...",
        bold.paint(len.to_string()),
        if len == 1 { "" } else { "s" }
    );
    rx.recv().map_err(|e| e.into()).and_then(|e| Err(e.into()))
}
