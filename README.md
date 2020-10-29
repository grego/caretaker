# caretaker 
[![Build status](https://badgen.net/travis/grego/caretaker)](https://travis-ci.org/grego/caretaker) 
[![Crates.io status](https://badgen.net/crates/v/caretaker)](https://crates.io/crates/caretaker)
[![Docs](https://docs.rs/caretaker/badge.svg)](https://docs.rs/caretaker)

A simple tool that loads a list of paths to watch from a TOML file.
```toml
[[watch]]
name = "print hello"
path = "src"
command = "echo $EVENT_PATH"

[[watch]]
name = "compile sass"
path = "sass/*.sass"
command = "sassc -t compressed sass/style.scss static/style.css"
```
On a change in the `path`, it executes the `command`. Directories are watched recursively.
Paths can also be specified with [globs](https://docs.rs/glob/0.3.0/glob/struct.Pattern.html).
Any shell command can be used, along with pipes and so on. 
By default, the shell specified in the `$SHELL` environment variable is used to parse and execute the command.
Otherwise, on Unix system, it invokes the default 
Bourne shell (`sh` command), on windows [cmd.exe](https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).
Additionally, each command gets the `$EVENT_PATH` environment variable, containing the path that changed.

Using [notify](https://github.com/notify-rs/notify) crate, which provides efficient event handling 
support for the most operating systems (apart from BSD).

## Installing
Currently, Caretaker is available on [AUR](https://aur.archlinux.org/packages/caretaker-bin/). You can
install it with some AUR helper, like `yay -S caretaker-bin`.

If you have Rust toolchain installed, you can install it with Cargo:
```
cargo install caretaker
```

## Running
Initialising with a dummy `.watch.toml` file:
```
caretaker init
````

Watching:
```
caretaker
```

You can also pass another file to load the config from via the `-w` option.

## License
[MIT](LICENSE)
