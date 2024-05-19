use super::Bytecode;
use crate::error::{span::Span, ChalError, CompileError, CompileErrorKind};

/// The structure, representing a type inside the interpreter. Used to assert
/// the type strictness of the script before it's execution.
#[derive(PartialEq, Debug, Clone, Default)]
pub enum Type {
    Int,
    Uint,
    Float,
    Str,
    Bool,
    Any,
    #[default]
    Void,
    Exception,
    Custom(Box<String>),
}

impl Type {
    /// NOTE: it is very important that this function goes after value calls
    /// inside the result.
    pub fn verify(
        exp: Type,
        recv: Type,
        code: &mut Vec<Bytecode>,
        span: Span,
    ) -> Result<(), ChalError> {
        if exp == Type::Any || exp == recv {
            return Ok(());
        }
        if exp == Type::Int && recv == Type::Uint {
            code.push(Bytecode::CastI);
            return Ok(());
        }

        if exp == Type::Float && (recv == Type::Uint || recv == Type::Int) {
            code.push(Bytecode::CastF);
            return Ok(());
        }

        Err(CompileError::new(CompileErrorKind::InvalidType(exp, recv), span).into())
    }

    /// Used to check whether an overloaded function's definition is applicable
    /// to the passed argument's types.
    pub fn soft_eq(&self, other: &Self) -> bool {
        match (self, other) {
            /* universal types */
            (Type::Void, _) | (_, Type::Void) => false,
            (Type::Any, _) => true,
            /* actual types */
            (Type::Int, Type::Int)
            | (Type::Uint, Type::Uint)
            | (Type::Float, Type::Float)
            | (Type::Str, Type::Str)
            | (Type::Bool, Type::Bool) => true,
            (Type::Custom(lhs), Type::Custom(rhs)) => lhs == rhs,
            /* implicit type casts */
            (Type::Int, Type::Uint) => true,
            _ => false,
        }
    }

    pub fn as_class(&self) -> String {
        match self {
            Type::Int => "Int".to_string(),
            Type::Uint => "Uint".to_string(),
            Type::Float => "Float".to_string(),
            Type::Str => "String".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Exception => "Exception".to_string(),
            Type::Custom(class) => *class.clone(),
            Type::Any => "Any".to_string(),
            Type::Void => "Void".to_string(),
        }
    }
}
