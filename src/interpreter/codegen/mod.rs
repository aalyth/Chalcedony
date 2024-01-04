mod expr;
mod func;
mod stmnt;
mod var;

use crate::error::ChalError;
use crate::parser::ast::NodeProg;
use crate::utils::BytecodeOpr;

use std::collections::BTreeMap;

use stmnt::stmnt_to_bytecode;

use super::FuncAnnotation;

pub trait ToBytecode {
    fn to_bytecode(
        self,
        bytecode_len: usize,
        var_symtable: &mut BTreeMap<String, usize>,
        func_symtable: &mut BTreeMap<String, FuncAnnotation>,
    ) -> Result<Vec<BytecodeOpr>, ChalError>;
}

impl ToBytecode for NodeProg {
    fn to_bytecode(
        self,
        bytecode_len: usize,
        var_symtable: &mut BTreeMap<String, usize>,
        func_symtable: &mut BTreeMap<String, FuncAnnotation>,
    ) -> Result<Vec<BytecodeOpr>, ChalError> {
        match self {
            NodeProg::VarDef(node) => node.to_bytecode(bytecode_len, var_symtable, func_symtable),
            NodeProg::FuncDef(node) => node.to_bytecode(bytecode_len, var_symtable, func_symtable),
        }
    }
}
