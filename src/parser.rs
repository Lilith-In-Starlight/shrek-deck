use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    path::PathBuf,
};

use crate::{CardEntry, GetCardInfo};

pub enum Error {
    UnexpectedChar {
        obtained: char,
        expected: Vec<String>,
    },
    AmountIsZero {
        card_name: String,
    },
    NameIsEmpty,
    NotANumber {
        string: String,
        error: ParseIntError,
    },
    CantOpenFile {
        path: PathBuf,
        error: io::Error,
    },
    NameMultipleTimes {
        name: String,
    },
    CouldntReadLine {
        path: PathBuf,
        line: usize,
        error: io::Error,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedChar { obtained, expected } => {
                let obtained = if *obtained == '\n' || *obtained == '\r' {
                    "<newline>".to_string()
                } else if *obtained == '\t' {
                    "<tab>".to_string()
                } else {
                    obtained.to_string()
                };
                write!(
                    f,
                    "\n Obtained character `{obtained}`, expected one of the following: "
                )?;

                for expected in expected {
                    write!(f, "\n - {expected}")?;
                }

                Ok(())
            }
            Self::AmountIsZero { card_name } => write!(
                f,
                "Tried to create {card_name} with an amount of 0, which is frankly ridiculous"
            ),
            Self::NameIsEmpty => write!(f, "Tried to create a card with an empty name"),
            Self::NotANumber { string, error } => {
                write!(f, "Failed to parse `{string}` as a number:\n  {error}")
            }
            Self::CantOpenFile { path, error } => write!(
                f,
                "Failed to load file `{}`, with the following error: {error}",
                path.display()
            ),
            Self::NameMultipleTimes { name } => write!(
                f,
                "The name `{name}` appears multiple times, which is not allowed."
            ),
            Self::CouldntReadLine { path, line, error } => {
                write!(
                    f,
                    "Failed to read line {line} in file {}:\n  {error}",
                    path.display()
                )
            }
        }
    }
}

pub struct ParseError {
    position: LinePosition,
    error: Error,
}

impl ParseError {
    fn at_line(self, line: usize) -> Self {
        Self {
            position: LinePosition {
                line: Some(line),
                ..self.position
            },
            ..self
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.position {
            LinePosition {
                line: None,
                column: None,
            } => write!(f, "Error at unknown position: {}", self.error),
            LinePosition {
                line: Some(line),
                column: Some(column),
            } => {
                write!(
                    f,
                    "Error at line {}, column {}: {}",
                    line, column, self.error
                )
            }
            LinePosition {
                line: None,
                column: Some(column),
            } => {
                write!(
                    f,
                    "Error at unknown line, column {}: {}",
                    column, self.error
                )
            }
            LinePosition {
                line: Some(line),
                column: None,
            } => {
                write!(f, "Error at line {}: {}", line, self.error)
            }
        }
    }
}

pub struct LinePosition {
    line: Option<usize>,
    column: Option<usize>,
}

impl LinePosition {
    const fn void() -> Self {
        Self {
            line: None,
            column: None,
        }
    }
}

/// Parses a line of text
/// # Errors
/// - Whenever the supplied `GetCardInfo` implementation of `parse` fails.
/// - Whenever a non-arabic digit character that is neither a space, a tab or an `x` is found during the parsing of the number.
/// - If the characters found as the amount of copies of the card cannot be parsed into an i64.
/// - If the characters found as the amount of copies of the card are parsed into the number 0.
/// - If the characters found as the name of the card is empty after being trimmed of spaces.
pub fn parse_line<T: GetCardInfo + Clone>(string: &str) -> Result<CardEntry<T>, ParseError> {
    let mut parserstate = ParserState::Numbering;
    let mut number_str = String::new();
    let mut name = String::new();
    for (idx, chr) in string.char_indices() {
        match parserstate {
            ParserState::Numbering => match chr {
                chr @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') => {
                    number_str.push(chr);
                }
                ' ' | '\t' => parserstate = ParserState::Exing,
                'x' => parserstate = ParserState::Naming,
                chr => {
                    let mut expected = vec!["a digit".to_string()];
                    if !number_str.is_empty() {
                        expected.push("a number separator (space, tab or `x`)".to_string());
                        expected.push("a card name".to_string());
                    }
                    return Err(ParseError {
                        error: Error::UnexpectedChar {
                            obtained: chr,
                            expected,
                        },
                        position: LinePosition {
                            line: None,
                            column: Some(idx + 1),
                        },
                    });
                }
            },
            ParserState::Exing => match chr {
                ' ' | '\t' => continue,
                'x' => parserstate = ParserState::Naming,
                chr => {
                    name.push(chr);
                    parserstate = ParserState::Naming;
                }
            },
            ParserState::Naming => name.push(chr),
        }
    }
    let name = name.trim().to_owned();

    let number = number_str.parse().map_err(|error| ParseError {
        position: LinePosition {
            line: None,
            column: None,
        },
        error: Error::NotANumber {
            string: number_str,
            error,
        },
    })?;

    if number == 0 {
        return Err(ParseError {
            error: Error::AmountIsZero { card_name: name },
            position: LinePosition {
                line: None,
                column: None,
            },
        });
    } else if name.is_empty() {
        return Err(ParseError {
            error: Error::NameIsEmpty,
            position: LinePosition {
                line: None,
                column: None,
            },
        });
    }

    Ok(CardEntry {
        card: T::parse(&name)?,
        amount: number,
    })
}

enum ParserState {
    Numbering,
    Naming,
    Exing,
}

/// Parses a file
/// # Errors
/// - If `parse_line` fails on any of the lines
/// - If the same card name appears multiple times in the file
/// - If the reader fails to read a line
pub fn parse_file<T: GetCardInfo + Clone>(
    path: &PathBuf,
) -> Result<Vec<CardEntry<T>>, Vec<ParseError>> {
    let file = File::open(path).map_err(|error| {
        vec![ParseError {
            position: LinePosition::void(),
            error: Error::CantOpenFile {
                path: path.clone(),
                error,
            },
        }]
    })?;
    let mut reader = BufReader::new(file);
    let mut cards = vec![];
    let mut used_names = vec![];
    let mut line_idx = 0;
    let mut errors = vec![];
    loop {
        line_idx += 1;
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if !line.trim().is_empty() => match parse_line::<T>(&line) {
                Ok(entry) => {
                    let name = entry.card.get_name().to_owned();
                    if used_names.contains(&name) {
                        errors.push(ParseError {
                            position: LinePosition {
                                line: Some(line_idx),
                                column: None,
                            },
                            error: Error::NameMultipleTimes { name },
                        });
                    } else {
                        used_names.push(name);
                        cards.push(entry);
                    }
                }
                Err(error) => errors.push(error.at_line(line_idx)),
            },
            Ok(_) => continue,
            Err(error) => errors.push(ParseError {
                position: LinePosition {
                    line: Some(line_idx),
                    column: None,
                },
                error: Error::CouldntReadLine {
                    path: path.clone(),
                    line: line_idx,
                    error,
                },
            }),
        }
    }
    if errors.is_empty() {
        Ok(cards)
    } else {
        Err(errors)
    }
}
