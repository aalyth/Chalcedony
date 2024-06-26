mod pos;
mod spanning;

pub use pos::Position;
pub use spanning::InlineSpanner;

use std::fmt;
use std::rc::Rc;

/// The trait, used to define objects which can build code snippets from a given
/// start and end position in the source code.
///
/// The filename is an optional part of the `Spanning` since it could be used in
/// a context such as a shell, in which case the code does not originate from a
/// script, but from the user's input.
pub trait Spanning {
    fn context(&self, start: &Position, end: &Position) -> String;
    fn filename(&self) -> Option<String>;
}

/// The structure, denoting a snippet of source code. Used in numerous
/// structures from the Lexer's tokens to the Abstract Syntax Tree nodes. The
/// purpose of this abstractions is to provide an easy way to display adequate
/// error messages with code snippets upon any encountered error.
#[derive(Clone)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub spanner: Rc<dyn Spanning>,
}

impl Span {
    pub fn new(start: Position, end: Position, spanner: Rc<dyn Spanning>) -> Self {
        Span {
            start,
            end,
            spanner,
        }
    }

    pub fn context(&self) -> String {
        self.spanner.context(&self.start, &self.end)
    }
}

impl From<Rc<dyn Spanning>> for Span {
    fn from(value: Rc<dyn Spanning>) -> Self {
        Span {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
            spanner: value.clone(),
        }
    }
}

impl std::cmp::PartialEq for Span {
    fn eq(&self, _other: &Span) -> bool {
        true
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<span instance>")
    }
}
