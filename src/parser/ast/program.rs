use crate::error::span::{Span, Spanning};
use crate::error::{ChalError, InternalError, ParserError};
use crate::lexer::{Delimiter, Operator};
use crate::lexer::{Keyword, Line, TokenKind};
use crate::parser::ast::{
    NodeAssign, NodeFuncCall, NodeFuncDef, NodeIfStmnt, NodeVarDef, NodeWhileLoop,
};

use crate::parser::{LineReader, TokenReader};

use std::collections::VecDeque;
use std::rc::Rc;

pub enum NodeProg {
    VarDef(NodeVarDef),
    FuncDef(NodeFuncDef),
    FuncCall(NodeFuncCall),
    Assign(NodeAssign),
    IfStmnt(NodeIfStmnt),
    WhileLoop(NodeWhileLoop),
}

macro_rules! single_line_stmnt {
    ( $enum_type: ident, $node_type: ident, $chunk: ident, $spanner: ident) => {{
        // SAFETY: the front line is already checked
        let front_line = $chunk.pop_front().unwrap().into();
        Ok(NodeProg::$enum_type($node_type::new(TokenReader::new(
            front_line,
            Span::from($spanner),
        ))?))
    }};
}

macro_rules! multi_line_stmnt {
    ( $enum_type: ident, $node_type: ident, $chunk: ident, $spanner: ident) => {{
        Ok(NodeProg::$enum_type($node_type::new(LineReader::new(
            $chunk, $spanner,
        ))?))
    }};
}

impl NodeProg {
    pub fn new(mut chunk: VecDeque<Line>, spanner: Rc<dyn Spanning>) -> Result<Self, ChalError> {
        if chunk.is_empty() {
            return Err(InternalError::new("NodeProg::new(): received an empty code chunk").into());
        }

        let front_line = chunk.front().unwrap();
        if front_line.tokens.is_empty() {
            return Err(InternalError::new("NodeProg::new(): empty first line of chunk").into());
        }

        let front_tok = front_line.front_tok().unwrap();

        match front_tok.kind {
            TokenKind::Keyword(Keyword::Let) => {
                single_line_stmnt!(VarDef, NodeVarDef, chunk, spanner)
            }
            TokenKind::Keyword(Keyword::Fn) => {
                multi_line_stmnt!(FuncDef, NodeFuncDef, chunk, spanner)
            }
            TokenKind::Keyword(Keyword::If) => {
                multi_line_stmnt!(IfStmnt, NodeIfStmnt, chunk, spanner)
            }
            TokenKind::Keyword(Keyword::While) => {
                multi_line_stmnt!(WhileLoop, NodeWhileLoop, chunk, spanner)
            }

            TokenKind::Identifier(_) => {
                let Some(peek_2nd) = front_line.tokens.get(1) else {
                    // by deafult we expect a function call
                    return Err(ParserError::expected_token(
                        TokenKind::Delimiter(Delimiter::OpenPar),
                        front_tok.span.clone(),
                    )
                    .into());
                };

                match &peek_2nd.kind {
                    TokenKind::Delimiter(Delimiter::OpenPar) => {
                        single_line_stmnt!(FuncCall, NodeFuncCall, chunk, spanner)
                    }
                    TokenKind::Operator(Operator::Eq)
                    | TokenKind::Operator(Operator::AddEq)
                    | TokenKind::Operator(Operator::SubEq)
                    | TokenKind::Operator(Operator::MulEq)
                    | TokenKind::Operator(Operator::DivEq)
                    | TokenKind::Operator(Operator::ModEq) => {
                        single_line_stmnt!(Assign, NodeAssign, chunk, spanner)
                    }
                    recv_kind => Err(ParserError::invalid_token(
                        TokenKind::Delimiter(Delimiter::OpenPar),
                        recv_kind.clone(),
                        peek_2nd.span.clone(),
                    )
                    .into()),
                }
            }

            _ => Err(InternalError::new(&format!(
                "NodeProg::new(): invalid chunk front - {:?}",
                front_tok.kind
            ))
            .into()),
        }
    }
}
