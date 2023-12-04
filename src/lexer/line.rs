use crate::lexer::Token;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Line {
    /* the number of spaces in (not tabulations) */
    indent: u64,
    tokens: VecDeque<Token>,
}

impl Line {
    pub fn new(indent: u64, tokens: VecDeque<Token>) -> Self {
        Line { indent, tokens }
    }

    pub fn tokens(&self) -> &VecDeque<Token> {
        &self.tokens
    }

    pub fn indent(&self) -> u64 {
        self.indent
    }

    pub fn front_tok(&self) -> Option<&Token> {
        self.tokens.front()
    }
}

impl Into<VecDeque<Token>> for Line {
    fn into(self) -> VecDeque<Token> {
        self.tokens
    }
}