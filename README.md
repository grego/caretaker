# caretaker 
[![Build status](https://flat.badgen.net/travis/grego/caretaker)](https://travis-ci.org/grego/caretaker) 
[![Crates.io status](https://flat.badgen.net/crates/v/caretaker)](https://crates.io/crates/caretaker)

A simple tool that loads a list of paths to watch from a TOML file.
```toml
[[watch]]
name = "print hello"
path = "src"
command = "echo \"hello world\""

[[watch]]
name = "compile sass"
path = "sass"
command = "sassc -t compressed sass/style.scss static/style.css"
```
On a change in the `path`, it executes the `command`. Directories are watched recursively.

Using [notify](https://github.com/notify-rs/notify) crate, which provides efficient event handling 
support for the most operating systems (apart from BSD).

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
