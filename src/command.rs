use serde::de::{Deserialize, Deserializer, Visitor};
use std::borrow::Cow;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::process;
use std::str::CharIndices;

pub struct Command(process::Command);

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(CommandVisitor)
    }
}

struct CommandVisitor;

impl<'de> Visitor<'de> for CommandVisitor {
    type Value = Command;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a command string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        let mut args = ArgIter::new(v).map(|cow| match cow {
            Cow::Borrowed(c) => Cow::Borrowed(c.as_ref()),
            Cow::Owned(c) => Cow::Owned(c.into()),
        });
        let mut command = process::Command::new(args.next().unwrap_or_default());
        command.args(args);
        Ok(Command(command))
    }
}

impl Deref for Command {
    type Target = process::Command;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Iterator over arguments of the command.
/// Can't simply use `split_whitespace`, since some argumenst may be quoted
/// and contain multiple words.
struct ArgIter<'a> {
    source: &'a str,
    chars: CharIndices<'a>,
}

impl<'a> ArgIter<'a> {
    fn new(source: &'a str) -> Self {
        ArgIter {
            source,
            chars: source.char_indices(),
        }
    }
}

impl<'a> Iterator for ArgIter<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        #[derive(Clone, Copy)]
        enum State {
            None,
            Quote(char),
        };

        let mut previous;
        let mut initial;
        let mut owned: Option<String> = None;
        loop {
            let (i, ch) = self.chars.next()?;
            if !ch.is_whitespace() {
                previous = ch;
                initial = i;
                break;
            }
        }
        let mut last = self.source.len();

        let mut state = match previous {
            '"' | '\'' => {
                initial += 1;
                State::Quote(previous)
            }
            '\\' => {
                initial += 1;
                State::None
            }
            _ => State::None,
        };
        let initial_state = state;
        let mut state_changes = 0;
        let mut found_whitespace = false;

        while let Some((l, c)) = self.chars.next() {
            last = l;

            if (c == '"' || c == '\'') && previous != '\\' {
                match state {
                    State::None => {
                        state_changes += 1;
                        state = State::Quote(c)
                    }
                    State::Quote(q) if c == q => {
                        state_changes += 1;
                        state = State::None
                    }
                    _ => {}
                }
            } else if let State::None = state {
                if c.is_whitespace() {
                    found_whitespace = true;
                    break;
                }
            }

            if c != '\\' || previous == '\\' {
                if let Some(ref mut arg) = owned {
                    arg.push(c);
                }
            }

            if c == '\\' && owned.is_none() {
                owned = Some(self.source[initial..last].to_string());
            }
            previous = c;
        }

        if !found_whitespace && owned.is_none() {
            last = self.source.len();
        };

        match initial_state {
            State::Quote(_) if state_changes == 1 => {
                if let Some(ref mut arg) = owned {
                    arg.pop();
                } else {
                    last -= 1;
                }
            }
            _ => {}
        };

        owned
            .map(|s| s.into())
            .or_else(|| self.source.get(initial..last).map(|s| s.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argiter() {
        let source = "echo h \"hello\\\" world\" \"\" \"h\\\"\"";
        let args: Vec<Cow<str>> = ArgIter::new(source).collect();
        assert_eq!(args, &["echo", "h", "hello\" world", "", "h\""]);
    }
}
