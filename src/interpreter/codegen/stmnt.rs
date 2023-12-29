use super::ToBytecode;

use crate::error::{ChalError, InternalError, RuntimeError};
use crate::interpreter::FuncAnnotation;
use crate::lexer::Type;
use crate::parser::ast::operators::AssignOprType;
use crate::parser::ast::{NodeRetStmnt, NodeStmnt, NodeVarCall};
use crate::utils::Bytecode;

use std::collections::{BTreeMap, HashSet};

pub fn stmnt_to_bytecode(
    node: NodeStmnt,
    bytecode_len: usize,
    func_symtable: &mut BTreeMap<String, FuncAnnotation>,
    func_lookup: &mut BTreeMap<String, u64>,
    parent_scope: &mut HashSet<String>,
    local_scope: &mut HashSet<String>,
) -> Result<Vec<u8>, ChalError> {
    match node {
        NodeStmnt::VarDef(node) => {
            let var_name = node.name();
            let mut var_bytecode = node.to_bytecode(bytecode_len, func_symtable, func_lookup)?;

            /* this capacity is for the best case scenario, if we need to delete the already
             * existing variable, there will be another allocation*/
            let mut result = Vec::<u8>::with_capacity(var_bytecode.len() + 1);

            /* 'shadow' the old variable value */
            if local_scope.contains(&var_name) {
                result.push(Bytecode::OpDeleteVar as u8);
                result.extend_from_slice(var_name.as_bytes());
                result.push(0);
            } else {
                local_scope.insert(var_name);
            }

            result.append(&mut var_bytecode);
            return Ok(result);
        }

        NodeStmnt::FuncCall(node) => {
            let Some(annotation) = func_symtable.get(&node.name()) else {
                let (fn_name, _, start, end, span) = node.disassemble();
                return Err(RuntimeError::unknown_function(fn_name, start, end, span).into());
            };

            let fn_ty = &annotation.ret_type;
            if *fn_ty != Type::Void {
                let (_, _, start, end, span) = node.disassemble();
                return Err(
                    RuntimeError::non_void_func_stmnt(fn_ty.clone(), start, end, span).into(),
                );
            }

            node.to_bytecode(bytecode_len, func_symtable, func_lookup)
        }

        NodeStmnt::RetStmnt(NodeRetStmnt(expr)) => {
            let mut result = Vec::<u8>::new();
            result.append(&mut expr.to_bytecode(bytecode_len, func_symtable, func_lookup)?);

            /* remove all variables in the current scope */
            for var in parent_scope.iter() {
                result.push(Bytecode::OpDeleteVar as u8);
                result.extend_from_slice(var.as_bytes());
                result.push(0);
            }

            for var in local_scope.iter() {
                result.push(Bytecode::OpDeleteVar as u8);
                result.extend_from_slice(var.as_bytes());
                result.push(0);
            }

            result.push(Bytecode::OpReturn as u8);
            Ok(result)
        }

        NodeStmnt::IfStmnt(node) => {
            let (cond, body_raw, branches) = node.disassemble();

            let mut result = Vec::<u8>::new();
            result.append(&mut cond.to_bytecode(bytecode_len, func_symtable, func_lookup)?);
            /* there is no need to use a type assertion for bool value since the OpIf
             * instruction already checks it */
            result.push(Bytecode::OpIf as u8);

            let mut body = Vec::<u8>::new();
            let mut current_parent_scope = parent_scope.clone();
            current_parent_scope.extend(local_scope.clone().into_iter());

            let mut current_local_scope = HashSet::<String>::new();
            for stmnt in body_raw {
                body.append(&mut stmnt_to_bytecode(
                    stmnt,
                    bytecode_len,
                    func_symtable,
                    func_lookup,
                    &mut current_parent_scope,
                    &mut current_local_scope,
                )?);
            }

            for var in current_local_scope {
                body.push(Bytecode::OpDeleteVar as u8);
                body.extend_from_slice(var.as_bytes());
                body.push(0);
            }

            let body_len = body.len() as u64;
            result.extend_from_slice(&body_len.to_ne_bytes());
            result.append(&mut body);

            Ok(result)
        }

        NodeStmnt::WhileLoop(node) => {
            let (cond, body_raw) = node.disassemble();

            let mut result = Vec::<u8>::new();
            result.append(&mut cond.to_bytecode(bytecode_len, func_symtable, func_lookup)?);
            /* there is no need to use a type assertion for bool value since the OpIf
             * instruction already checks it */
            result.push(Bytecode::OpIf as u8);

            let mut body = Vec::<u8>::new();
            let mut current_parent_scope = parent_scope.clone();
            current_parent_scope.extend(local_scope.clone().into_iter());

            let mut current_local_scope = HashSet::<String>::new();
            for stmnt in body_raw {
                body.append(&mut stmnt_to_bytecode(
                    stmnt,
                    bytecode_len,
                    func_symtable,
                    func_lookup,
                    &mut current_parent_scope,
                    &mut current_local_scope,
                )?);
            }

            for var in current_local_scope {
                body.push(Bytecode::OpDeleteVar as u8);
                body.extend_from_slice(var.as_bytes());
                body.push(0);
            }

            /* skipping over the body if the condition is false */
            let body_len = body.len() as u64 + 9;
            result.extend_from_slice(&body_len.to_ne_bytes());
            result.append(&mut body);

            /* how much to go back when we have iterated the body */
            let dist: i64 = -(result.len() as i64) - 1;
            result.push(Bytecode::OpJmp as u8);
            result.extend_from_slice(&dist.to_ne_bytes());

            Ok(result)
        }

        NodeStmnt::Assign(node) => {
            let (NodeVarCall(varname), opr, rhs) = node.disassemble();
            let mut result = Vec::<u8>::new();
            if opr != AssignOprType::Eq {
                result.push(Bytecode::OpGetVar as u8);
                result.extend_from_slice(&varname.as_bytes());
                result.push(0);
            }

            result.append(&mut rhs.to_bytecode(bytecode_len, func_symtable, func_lookup)?);

            match opr {
                AssignOprType::AddEq => result.push(Bytecode::OpAdd as u8),
                AssignOprType::SubEq => result.push(Bytecode::OpSub as u8),
                AssignOprType::MulEq => result.push(Bytecode::OpMul as u8),
                AssignOprType::DivEq => result.push(Bytecode::OpDiv as u8),
                AssignOprType::ModEq => result.push(Bytecode::OpMod as u8),
                _ => {}
            }

            result.push(Bytecode::OpDeleteVar as u8);
            result.extend_from_slice(&varname.as_bytes());
            result.push(0);
            result.push(Bytecode::OpCreateVar as u8);
            result.extend_from_slice(&varname.as_bytes());
            result.push(0);

            Ok(result)
        }
    }
}
