use crate::command::Command;
use ansi_term::Style;
use crossbeam_channel::unbounded;
use notify::{immediate_watcher, Error, RecursiveMode, Watcher};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Deserialize)]
struct Watch {
    #[serde(default)]
    name: String,
    path: PathBuf,
    command: Mutex<Command>,
}

#[derive(Deserialize)]
pub struct Config {
    watch: Vec<Watch>,
}

pub fn watch(config: Config) -> Result<(), Error> {
    use notify::event::{EventKind::*, *};

    let len = config.watch.len();
    let mut watchers = Vec::new();
    let (tx, rx) = unbounded();
    let bold = Style::new().bold();
    for Watch {
        name,
        path,
        command,
    } in config.watch.into_iter()
    {
        let tx = tx.clone();
        let mut watcher = immediate_watcher(move |res: Result<Event, _>| match res {
            Ok(event) => match event.kind {
                Access(AccessKind::Close(AccessMode::Write))
                | Create(_)
                | Modify(ModifyKind::Name(RenameMode::To))
                | Remove(_) => {
                    println!("{:?} changed, running {}", &event.paths, bold.paint(&name));
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
    rx.recv().map_err(|e| e.into()).and_then(Err)
}
