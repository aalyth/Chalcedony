use crate::error::span::{Span, Spanning};
use crate::error::{ChalError, InternalError};
use crate::lexer::{Keyword, Line, Token, TokenKind};

use std::collections::VecDeque;
use std::rc::Rc;

use super::token_reader::TokenReader;

pub struct LineReader {
    src: VecDeque<Line>,
    spanner: Rc<dyn Spanning>,
}

impl LineReader {
    pub fn new(src: VecDeque<Line>, spanner: Rc<dyn Spanning>) -> Self {
        LineReader { src, spanner }
    }

    pub fn spanner(&self) -> Rc<dyn Spanning> {
        self.spanner.clone()
    }

    pub fn indent(&self) -> Option<u64> {
        Some(self.src.front()?.indent)
    }

    pub fn peek_tok(&self) -> Option<&Token> {
        self.src.front()?.front_tok()
    }

    pub fn advance(&mut self) -> Option<Line> {
        self.src.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.src.is_empty()
    }

    pub fn advance_until(
        &mut self,
        cond: impl Fn(&Line) -> bool,
    ) -> Result<VecDeque<Line>, ChalError> {
        let mut result = VecDeque::<Line>::new();

        /* we advance at least the first line */
        let Some(front_ln) = self.advance() else {
            return Err(InternalError::new(
                "LexerReader::advance_chunk(): advancing an empty reader",
            )
            .into());
        };
        result.push_back(front_ln);

        while let Some(front) = self.src.front() {
            if cond(front) {
                break;
            }

            result.push_back(self.advance().unwrap());
        }
        Ok(result)
    }

    pub fn advance_chunk(&mut self) -> Result<Self, ChalError> {
        let Some(front) = self.src.front() else {
            return Err(InternalError::new(
                "LexerReader::advance_chunk(): advancing an empty reader",
            )
            .into());
        };
        /* NOTE: this line is necessary so front goes out of scope and the borrow checker is happy */
        let indent = front.indent;
        let cond = |ln: &Line| -> bool { ln.indent <= indent };

        let mut res = self.advance_until(cond)?;

        /* if the chunk is of type if statement check for elif/else bodies */
        if let Some(front_ln) = res.front() {
            if let Some(front_tok) = front_ln.front_tok() {
                if front_tok.kind != TokenKind::Keyword(Keyword::If) {
                    return Ok(LineReader::new(res, self.spanner.clone()));
                }
            }
        };
        while let Some(peek) = self.peek_tok() {
            match peek.kind {
                TokenKind::Keyword(Keyword::Elif) => res.append(&mut self.advance_until(cond)?),
                TokenKind::Keyword(Keyword::Else) => {
                    res.append(&mut self.advance_until(cond)?);
                    break;
                }
                _ => break,
            }
        }

        Ok(LineReader::new(res, self.spanner.clone()))
    }

    pub fn advance_reader(&mut self) -> Result<TokenReader, ChalError> {
        let Some(next) = self.src.pop_front() else {
            return Err(InternalError::new(
                "LineReader::advance_reader(): advancing an empty reader",
            )
            .into());
        };

        Ok(TokenReader::new(
            next.into(),
            Span::from(self.spanner.clone()),
        ))
    }
}
