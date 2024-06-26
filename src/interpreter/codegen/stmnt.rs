use super::var::var_exists;
use super::ToBytecode;

use crate::error::{ChalError, CompileError, CompileErrorKind};
use crate::interpreter::{Chalcedony, LoopScope, SafetyScope, VarAnnotation};
use crate::parser::ast::{
    NodeAssign, NodeBreakStmnt, NodeContStmnt, NodeElifStmnt, NodeElseStmnt, NodeExprInner,
    NodeForLoop, NodeFuncCallStmnt, NodeIfBranch, NodeIfStmnt, NodeRetStmnt, NodeStmnt, NodeThrow,
    NodeTryCatch, NodeWhileLoop,
};

use crate::common::operators::{AssignOprType, BinOprType};
use crate::common::{Bytecode, Type};

use std::collections::VecDeque;

/// Used for easier manipulation over the current while scope.
fn increment_loop_scope(interpreter: &mut Chalcedony, val: usize) {
    if let Some(loop_scope) = interpreter.current_loop.as_mut() {
        loop_scope.current_length += val;
    }
}

/// Used to default back the previously set scope length. The reason for this is
/// that `NodeStmnt::to_bytecode()` by default increments the current loop's
/// length by the resulting bytecode's length, which would lead to double
/// increments in the nodes `NodeIfStmnt`, `NodeElifStmnt`, `NodeElseStmnt`, and
/// `NodeElifStmnt`.
fn set_loop_scope(interpreter: &mut Chalcedony, val: usize) {
    if let Some(loop_scope) = interpreter.current_loop.as_mut() {
        loop_scope.current_length = val;
    }
}

/// Updates the loop scope to a new scope, returning the old one.
fn update_loop_scope(interpreter: &mut Chalcedony) -> Option<LoopScope> {
    let prev_loop_scope = interpreter.current_loop.clone();
    interpreter.current_loop = Some(LoopScope::default());
    prev_loop_scope
}

fn get_loop_scope_len(interpreter: &mut Chalcedony) -> usize {
    match interpreter.current_loop.as_ref() {
        Some(scope) => scope.current_length,
        None => 0,
    }
}

fn fix_unfinished_breaks(interpreter: &Chalcedony, code: &mut Vec<Bytecode>) {
    let scope = interpreter
        .current_loop
        .as_ref()
        .expect("the while scope disappeared");
    /* finish off any break statements */
    for pos in &scope.unfinished_breaks {
        /* the distance to terminate the while */
        let distance: isize = (code.len() - pos) as isize;
        *code.get_mut(*pos).unwrap() = Bytecode::Jmp(distance);
    }
}

impl ToBytecode for NodeStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let result: Vec<Bytecode> = match self {
            // the reason that `NodeVarDef::to_bytecode()` is not used is due to
            // `NodeStmnt::VarDef()` represents a local variable, where the
            // `NodeProg::VarDef()` represents a global variable
            NodeStmnt::VarDef(mut node) => {
                /* the empty variable is ignored */
                if node.name == "_" {
                    /* check for any potential invalid code */
                    let _ = node.value.as_type(interpreter)?;
                    return Ok(vec![]);
                }

                if interpreter.locals.contains_key(&node.name) {
                    return Err(CompileError::new(
                        CompileErrorKind::RedefiningVariable,
                        node.span.clone(),
                    )
                    .into());
                }

                /* check whether the variable exists as a function's argument */
                if let Some(func) = interpreter.current_func.clone() {
                    if func.arg_lookup.get(&node.name).is_some() {
                        return Err(CompileError::new(
                            CompileErrorKind::RedefiningFunctionArg,
                            node.span,
                        )
                        .into());
                    }
                }

                let mut result = node.value.clone().to_bytecode(interpreter)?;

                let value_type = node.value.as_type(interpreter)?;
                if node.ty != Type::Any {
                    Type::verify(
                        node.ty.clone(),
                        value_type,
                        &mut result,
                        node.value.span.clone(),
                    )?;
                } else if value_type.root_type() == Type::Any {
                    return Err(CompileError::new(
                        CompileErrorKind::UninferableType(value_type),
                        node.value.span,
                    )
                    .into());
                } else {
                    node.ty = value_type;
                }

                /* this implicitly adds the variable to the locals symtable */
                let var_id = interpreter.get_local_id(&node);
                result.push(Bytecode::SetLocal(var_id));

                result
            }

            NodeStmnt::FuncCall(NodeFuncCallStmnt(node)) => {
                let resolution_ty = node.as_type(interpreter)?;
                if resolution_ty != Type::Void && interpreter.inside_stmnt {
                    return Err(CompileError::new(
                        CompileErrorKind::NonVoidFunctionStmnt(resolution_ty),
                        node.span.clone(),
                    )
                    .into());
                }

                node.to_bytecode(interpreter)?
            }
            NodeStmnt::RetStmnt(node) => node.to_bytecode(interpreter)?,
            NodeStmnt::Assign(node) => node.to_bytecode(interpreter)?,

            NodeStmnt::IfStmnt(node) => node.to_bytecode(interpreter)?,
            NodeStmnt::WhileLoop(node) => node.to_bytecode(interpreter)?,
            NodeStmnt::ContStmnt(node) => node.to_bytecode(interpreter)?,
            NodeStmnt::BreakStmnt(node) => node.to_bytecode(interpreter)?,
            NodeStmnt::ForLoop(node) => node.to_bytecode(interpreter)?,

            NodeStmnt::TryCatch(node) => node.to_bytecode(interpreter)?,
            NodeStmnt::Throw(node) => node.to_bytecode(interpreter)?,
        };

        increment_loop_scope(interpreter, result.len());
        Ok(result)
    }
}

impl ToBytecode for NodeIfBranch {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        match self {
            NodeIfBranch::Elif(node) => node.to_bytecode(interpreter),
            NodeIfBranch::Else(node) => node.to_bytecode(interpreter),
        }
    }
}

impl ToBytecode for NodeElifStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let prev_loop_scope_len = get_loop_scope_len(interpreter);

        let mut result = self.condition.clone().to_bytecode(interpreter)?;

        let cond_ty = self.condition.as_type(interpreter)?;
        Type::verify(
            Type::Bool,
            cond_ty,
            &mut result,
            self.condition.span.clone(),
        )?;

        increment_loop_scope(interpreter, result.len());

        let body = self.body.to_bytecode(interpreter)?;

        /* the extra length is for potential jumps over other branches */
        result.push(Bytecode::If(body.len() + 1));
        result.extend(body);

        set_loop_scope(interpreter, prev_loop_scope_len);

        Ok(result)
    }
}

impl ToBytecode for NodeElseStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        self.body.to_bytecode(interpreter)
    }
}

impl ToBytecode for Vec<NodeStmnt> {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let mut result = Vec::<Bytecode>::new();
        let mut errors = Vec::<ChalError>::new();

        for stmnt in self {
            match stmnt.to_bytecode(interpreter) {
                Ok(bytecode) => result.extend(bytecode),
                Err(err) => errors.push(err),
            }
        }

        if !errors.is_empty() {
            return Err(errors.into());
        }

        Ok(result)
    }
}

impl ToBytecode for NodeIfStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let prev_loop_scope_len = get_loop_scope_len(interpreter);

        let mut result = self.condition.clone().to_bytecode(interpreter)?;
        let cond_ty = self.condition.as_type(interpreter)?;
        Type::verify(
            Type::Bool,
            cond_ty,
            &mut result,
            self.condition.span.clone(),
        )?;

        increment_loop_scope(interpreter, result.len() + 1);

        let body = self.body.to_bytecode(interpreter)?;

        increment_loop_scope(interpreter, body.len() + 1);

        let mut branches: Vec<Vec<Bytecode>> = Vec::new();
        let mut errors: Vec<ChalError> = Vec::new();
        for branch in self.branches {
            match branch.to_bytecode(interpreter) {
                Ok(bytecode) => {
                    increment_loop_scope(interpreter, bytecode.len() + 1);
                    branches.push(bytecode);
                }
                Err(err) => errors.push(err),
            }
        }

        set_loop_scope(interpreter, prev_loop_scope_len + 1);

        if !errors.is_empty() {
            return Err(errors.into());
        }

        let mut branches_len: usize = branches.iter().map(|el| el.len()).sum();
        branches_len += branches.len();

        let mut leftover_branch_len: isize = branches_len as isize;

        fn push_jump(code: &mut Vec<Bytecode>, jmp_dist: isize) {
            if jmp_dist > 0 {
                code.push(Bytecode::Jmp(jmp_dist));
            } else {
                code.push(Bytecode::Nop);
            }
        }

        result.push(Bytecode::If(body.len() + 1));
        result.extend(body);
        push_jump(&mut result, leftover_branch_len);

        for branch in branches.into_iter() {
            leftover_branch_len -= (branch.len() + 1) as isize;
            result.extend(branch);
            push_jump(&mut result, leftover_branch_len);
        }

        Ok(result)
    }
}

impl ToBytecode for NodeWhileLoop {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let prev_loop_scope = update_loop_scope(interpreter);
        let mut result = self.condition.clone().to_bytecode(interpreter)?;

        let cond_ty = self.condition.as_type(interpreter)?;
        Type::verify(
            Type::Bool,
            cond_ty,
            &mut result,
            self.condition.span.clone(),
        )?;

        increment_loop_scope(interpreter, result.len() + 1);

        let body = self.body.to_bytecode(interpreter)?;
        let body_len = body.len() + 1; // taking into account the jump backwards

        result.push(Bytecode::If(body_len));
        result.extend(body);

        fix_unfinished_breaks(interpreter, &mut result);
        interpreter.current_loop = prev_loop_scope;

        /* how much to go back when we have iterated the body */
        let dist = -(result.len() as isize) - 1;
        result.push(Bytecode::Jmp(dist));

        Ok(result)
    }
}

impl ToBytecode for NodeAssign {
    fn to_bytecode(mut self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        /* the annotation of the mutated variable */
        let annotation: VarAnnotation;
        let mut set_instr: Bytecode;

        let root = self.lhs.first().as_var_call().unwrap().clone();

        if !var_exists(&root.name, interpreter) {
            return Err(
                CompileError::new(CompileErrorKind::UnknownVariable(root.name), root.span).into(),
            );
        }

        if let Some(var) = interpreter.locals.get(&root.name) {
            annotation = var.clone();
            set_instr = Bytecode::SetLocal(annotation.id);

        /* check whether the interpreter is compiling inside a function scope */
        } else if let Some(func) = interpreter.current_func.clone() {
            /* check whether the variable is an argument */
            if let Some(arg) = func.arg_lookup.get(&root.name) {
                annotation = VarAnnotation::new(arg.id, arg.ty.clone(), false);
                set_instr = Bytecode::SetLocal(annotation.id);

            /* the variable is global, but is mutated inside a statement */
            } else {
                return Err(
                    CompileError::new(CompileErrorKind::MutatingExternalState, root.span).into(),
                );
            }

        /* the interpreter is in the global scope*/
        } else if let Some(var) = interpreter.globals.get(&root.name) {
            annotation = var.clone();
            set_instr = Bytecode::SetGlobal(annotation.id);
        } else {
            /* this is necessary for the proper compilation */
            unreachable!();
        }

        if annotation.is_const {
            return Err(CompileError::new(CompileErrorKind::MutatingConstant, root.span).into());
        }

        let mut result = Vec::<Bytecode>::new();
        if self.lhs.resolution.len() > 1 {
            result.extend(self.lhs.clone().to_bytecode(interpreter)?);
            let Bytecode::GetAttr(attr_id) = result.pop().unwrap() else {
                panic!("attribute resolution does not end with `GetAttr`");
            };
            set_instr = Bytecode::SetAttr(attr_id);
        }

        if self.opr != AssignOprType::Eq {
            self.rhs
                .expr
                .push_front(NodeExprInner::Resolution(self.lhs.clone()));

            macro_rules! push_bin_opr {
                ($expr:expr, $type:ident) => {{
                    $expr.push_back(NodeExprInner::BinOpr(BinOprType::$type))
                }};
            }

            match self.opr {
                AssignOprType::AddEq => push_bin_opr!(self.rhs.expr, Add),
                AssignOprType::SubEq => push_bin_opr!(self.rhs.expr, Sub),
                AssignOprType::MulEq => push_bin_opr!(self.rhs.expr, Mul),
                AssignOprType::DivEq => push_bin_opr!(self.rhs.expr, Div),
                AssignOprType::ModEq => push_bin_opr!(self.rhs.expr, Mod),
                _ => unreachable!(),
            }
        }

        result.extend(self.rhs.clone().to_bytecode(interpreter)?);

        let rhs_ty = self.rhs.as_type(interpreter)?;
        let lhs_ty = self.lhs.as_type(interpreter)?;
        Type::verify(lhs_ty, rhs_ty, &mut result, self.rhs.span)?;

        result.push(set_instr);

        Ok(result)
    }
}

impl ToBytecode for NodeRetStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let Some(func) = interpreter.current_func.clone() else {
            return Err(
                CompileError::new(CompileErrorKind::ReturnOutsideFunc, self.span.clone()).into(),
            );
        };

        let recv_type = self.value.as_type(interpreter)?;
        let exp_type = func.ret_type.clone();

        if exp_type == Type::Void && recv_type == Type::Void {
            return Ok(vec![Bytecode::ReturnVoid]);
        }

        let mut result = self.value.clone().to_bytecode(interpreter)?;

        Type::verify(exp_type, recv_type, &mut result, self.value.span)?;
        result.push(Bytecode::Return);

        Ok(result)
    }
}

impl ToBytecode for NodeBreakStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let Some(scope) = interpreter.current_loop.as_mut() else {
            return Err(CompileError::new(
                CompileErrorKind::CtrlFlowOutsideLoop,
                self.span.clone(),
            )
            .into());
        };
        scope.unfinished_breaks.push(scope.current_length);
        Ok(vec![Bytecode::Nop])
    }
}

impl ToBytecode for NodeContStmnt {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let Some(scope) = interpreter.current_loop.as_mut() else {
            return Err(CompileError::new(
                CompileErrorKind::CtrlFlowOutsideLoop,
                self.span.clone(),
            )
            .into());
        };
        Ok(vec![Bytecode::Jmp(-(scope.current_length as isize) - 1)])
    }
}

impl ToBytecode for NodeForLoop {
    #[allow(non_upper_case_globals)]
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let prev_loop_scope = update_loop_scope(interpreter);

        if interpreter.locals.contains_key(&self.iter.name) {
            return Err(CompileError::new(
                CompileErrorKind::RedefiningVariable,
                self.iter.span.clone(),
            )
            .into());
        }

        /* check whether the variable exists as a function's argument */
        if let Some(func) = interpreter.current_func.clone() {
            if func.arg_lookup.get(&self.iter.name).is_some() {
                return Err(CompileError::new(
                    CompileErrorKind::RedefiningVariable,
                    self.iter.span,
                )
                .into());
            }
        }

        let iterable_ty = self.iterable.as_type(interpreter)?;
        let iterable_class = iterable_ty.as_class();

        let Some(iter_fn_ann) = interpreter.get_function_universal(
            "__iter__",
            &VecDeque::from([iterable_ty.clone()]),
            Some(&iterable_class),
        ) else {
            return Err(CompileError::new(
                CompileErrorKind::MethodNotImplemented(iterable_class + "::__iter__()"),
                self.iterable.span,
            )
            .into());
        };

        let iter_gen_type = iter_fn_ann.ret_type;
        let iter_gen_class = iter_gen_type.as_class();

        let Some(next_fn_ann) = interpreter.get_function_universal(
            "__next__!",
            &VecDeque::from([iter_gen_type.clone()]),
            Some(&iter_gen_class),
        ) else {
            return Err(CompileError::new(
                CompileErrorKind::MethodNotImplemented(iter_gen_class + "::__next__!()"),
                self.iterable.span,
            )
            .into());
        };

        let iterator_type = next_fn_ann.ret_type;
        let iterator_id = interpreter.get_local_id_internal(&self.iter.name, iterator_type, false);
        let iterator_gen_id = interpreter.globals_id_counter;
        interpreter.globals_id_counter += 1;

        // For loop structure:
        // let iterator = <iterable>.__iter__()
        // try:
        //     <iterator-var> = iterator.__next__!()
        // catch(_: exception):
        //      jump(body_len + 1)
        // <body>
        // <jump-to-try-block>

        let mut init_iterator = Vec::<Bytecode>::new();
        init_iterator.extend(self.iterable.to_bytecode(interpreter)?);
        init_iterator.extend(iter_fn_ann.bytecode);
        init_iterator.push(Bytecode::SetGlobal(iterator_gen_id));

        let mut for_loop = Vec::<Bytecode>::new();
        for_loop.extend(vec![
            Bytecode::TryScope(next_fn_ann.bytecode.len() + 3),
            Bytecode::GetGlobal(iterator_gen_id),
        ]);

        for_loop.extend(next_fn_ann.bytecode);
        for_loop.extend(vec![
            Bytecode::SetLocal(iterator_id),
            Bytecode::CatchJmp(1),
            Bytecode::Nop,
        ]);
        let jump_over_idx = for_loop.len() - 1;

        increment_loop_scope(interpreter, for_loop.len());

        let body = self.body.to_bytecode(interpreter)?;
        let body_len = body.len() as isize;
        for_loop.extend(body);

        fix_unfinished_breaks(interpreter, &mut for_loop);
        interpreter.current_loop = prev_loop_scope;

        for_loop.push(Bytecode::Jmp(-(for_loop.len() as isize) - 1));
        *for_loop.get_mut(jump_over_idx).unwrap() = Bytecode::Jmp(body_len + 1);

        interpreter.remove_local(&self.iter.name);
        interpreter.globals_id_counter -= 1;
        init_iterator.extend(for_loop);

        Ok(init_iterator)
    }
}

impl ToBytecode for NodeTryCatch {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        if interpreter.safety_scope == SafetyScope::Guarded {
            return Err(CompileError::new(CompileErrorKind::NestedTryCatch, self.try_span).into());
        }

        interpreter.safety_scope = SafetyScope::Guarded;
        /* this instruction will be overwritten by `Bytecode::TryScope()` */
        let mut result = vec![Bytecode::Nop];
        result.extend(self.try_body.to_bytecode(interpreter)?);

        interpreter.safety_scope = SafetyScope::Safe;
        if var_exists(&self.exception_var.name, interpreter) {
            return Err(CompileError::new(
                CompileErrorKind::RedefiningVariable,
                self.exception_var.span,
            )
            .into());
        }
        /* create the variable, holding the exception */
        let exc_id =
            interpreter.get_local_id_internal(&self.exception_var.name, Type::Exception, false);
        let mut catch_body = vec![Bytecode::SetLocal(exc_id)];

        catch_body.extend(self.catch_body.to_bytecode(interpreter)?);
        interpreter.remove_local(&self.exception_var.name);

        result.push(Bytecode::CatchJmp(catch_body.len()));
        *result.get_mut(0).unwrap() = Bytecode::TryScope(result.len() - 1);
        result.extend(catch_body);

        interpreter.safety_scope = SafetyScope::Normal;

        Ok(result)
    }
}

impl ToBytecode for NodeThrow {
    fn to_bytecode(self, interpreter: &mut Chalcedony) -> Result<Vec<Bytecode>, ChalError> {
        let NodeThrow(exception) = self;

        if interpreter.safety_scope == SafetyScope::Safe {
            return Err(
                CompileError::new(CompileErrorKind::UnsafeOpInSafeBlock, exception.span).into(),
            );
        }

        if let Some(func) = interpreter.current_func.clone() {
            if !func.is_unsafe && interpreter.safety_scope != SafetyScope::Guarded {
                return Err(
                    CompileError::new(CompileErrorKind::ThrowInSafeFunc, exception.span).into(),
                );
            }
        }

        let exc_ty = exception.as_type(interpreter)?;
        if exc_ty != Type::Str {
            return Err(CompileError::new(
                CompileErrorKind::InvalidType(Type::Str, exc_ty),
                exception.span,
            )
            .into());
        }

        let mut result = exception.to_bytecode(interpreter)?;
        result.push(Bytecode::ThrowException);
        Ok(result)
    }
}
