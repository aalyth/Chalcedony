use crate::error::{span::Span, ChalError};
use crate::lexer::{Keyword, Operator, Special, TokenKind};
use crate::parser::{ast::NodeExpr, TokenReader};

use crate::common::Type;

pub struct NodeVarDef {
    pub ty: Type,
    pub name: String,
    pub value: NodeExpr,
    pub span: Span,
}

impl NodeVarDef {
    pub fn new(mut reader: TokenReader) -> Result<NodeVarDef, ChalError> {
        /* let a = 5      */
        /* let b: int = 3 */
        reader.expect_exact(TokenKind::Keyword(Keyword::Let))?;

        let lhs_tok = reader.expect(TokenKind::Identifier("".to_string()))?;
        let name = lhs_tok.src;
        let span = lhs_tok.span;

        let mut ty = Type::Any;
        if reader
            .expect_exact(TokenKind::Special(Special::Colon))
            .is_ok()
        {
            ty = reader.expect_type()?;

            reader.expect_exact(TokenKind::Operator(Operator::Eq))?;
        } else {
            reader.expect_exact(TokenKind::Operator(Operator::Eq))?;
        }

        let rhs = reader.advance_until(|tk| tk == &TokenKind::Newline)?;
        let rhs_reader = TokenReader::new(rhs, reader.current());
        let value = NodeExpr::new(rhs_reader)?;
        reader.expect_exact(TokenKind::Newline)?;

        Ok(NodeVarDef {
            name,
            ty,
            value,
            span,
        })
    }
}
