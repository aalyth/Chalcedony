use chalcedony::common::operators::{BinOprType, UnaryOprType};
use chalcedony::common::Type;

use chalcedony::lexer::{Delimiter, Keyword, Operator, Special, TokenKind};
use chalcedony::parser::ast::{
    class::Member, func::Arg, NodeAttrRes, NodeAttribute, NodeBreakStmnt, NodeClass, NodeContStmnt,
    NodeElifStmnt, NodeElseStmnt, NodeExpr, NodeExprInner, NodeFuncCall, NodeFuncCallStmnt,
    NodeFuncDef, NodeIfBranch, NodeIfStmnt, NodeInlineClass, NodeList, NodeRetStmnt, NodeStmnt,
    NodeThrow, NodeTryCatch, NodeValue, NodeVarCall, NodeVarDef, NodeWhileLoop,
};

use chalcedony::mocks::{hash_map, line, line_reader, token_reader, vecdeq, SpanMock};

#[test]
fn parse_var_def() {
    // equivalent to the code:
    // ```
    // let a: uint = (-fib(10)) + 3
    // ```
    let tokens = token_reader!(
        TokenKind::Keyword(Keyword::Let),
        TokenKind::Identifier("a".to_string()),
        TokenKind::Special(Special::Colon),
        TokenKind::Type(Type::Uint),
        TokenKind::Operator(Operator::Eq),
        TokenKind::Delimiter(Delimiter::OpenPar),
        TokenKind::Operator(Operator::Neg),
        TokenKind::Identifier("fib".to_string()),
        TokenKind::Delimiter(Delimiter::OpenPar),
        TokenKind::Uint(10),
        TokenKind::Delimiter(Delimiter::ClosePar),
        TokenKind::Delimiter(Delimiter::ClosePar),
        TokenKind::Operator(Operator::Add),
        TokenKind::Uint(3)
    );

    let recv = NodeVarDef::new(tokens).expect("did not parse NodeVarDef");

    let exp = NodeVarDef {
        ty: Type::Uint,
        name: "a".to_string(),
        value: NodeExpr {
            expr: vecdeq![
                NodeExprInner::Resolution(NodeAttrRes {
                    resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                        name: "fib".to_string(),
                        namespace: None,
                        args: vec![NodeExpr {
                            expr: vecdeq!(NodeExprInner::Value(NodeValue::Uint(10))),
                            span: SpanMock::new()
                        }],
                        span: SpanMock::new(),
                    })],
                    span: SpanMock::new(),
                }),
                NodeExprInner::UnaryOpr(UnaryOprType::Neg),
                NodeExprInner::Value(NodeValue::Uint(3)),
                NodeExprInner::BinOpr(BinOprType::Add)
            ],
            span: SpanMock::new(),
        },
        is_const: false,
        span: SpanMock::new(),
    };

    assert_eq!(exp, recv);
}

#[test]
fn parse_func_def() {
    // equivalent to the code:
    // ```
    // fn fib(n: int) -> uint:
    //     if n > 2:
    //         return fib(n-2) + fib(n-1)
    //     return 1
    // ```
    let code = line_reader!(
        line!(
            0,
            TokenKind::Keyword(Keyword::Fn),
            TokenKind::Identifier("fib".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("n".to_string()),
            TokenKind::Special(Special::Colon),
            TokenKind::Type(Type::Int),
            TokenKind::Delimiter(Delimiter::ClosePar),
            TokenKind::Special(Special::RightArrow),
            TokenKind::Type(Type::Uint),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Keyword(Keyword::If),
            TokenKind::Identifier("n".to_string()),
            TokenKind::Operator(Operator::Gt),
            TokenKind::Uint(2),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            8,
            TokenKind::Keyword(Keyword::Return),
            TokenKind::Identifier("fib".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("n".to_string()),
            TokenKind::Operator(Operator::Sub),
            TokenKind::Uint(2),
            TokenKind::Delimiter(Delimiter::ClosePar),
            TokenKind::Operator(Operator::Add),
            TokenKind::Identifier("fib".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("n".to_string()),
            TokenKind::Operator(Operator::Sub),
            TokenKind::Uint(1),
            TokenKind::Delimiter(Delimiter::ClosePar)
        ),
        line!(4, TokenKind::Keyword(Keyword::Return), TokenKind::Uint(1))
    );

    let recv = NodeFuncDef::new(code).expect("did not parse NodeFuncDef");

    let exp = NodeFuncDef {
        name: "fib".to_string(),
        args: vecdeq![Arg {
            name: "n".to_string(),
            ty: Type::Int,
        }],
        namespace: None,
        ret_type: Type::Uint,
        body: vec![
            /* if n > 2: */
            NodeStmnt::IfStmnt(NodeIfStmnt {
                condition: NodeExpr {
                    expr: vecdeq![
                        NodeExprInner::Resolution(NodeAttrRes {
                            resolution: vec![NodeAttribute::VarCall(NodeVarCall {
                                name: "n".to_string(),
                                span: SpanMock::new()
                            })],
                            span: SpanMock::new()
                        }),
                        NodeExprInner::Value(NodeValue::Uint(2)),
                        NodeExprInner::BinOpr(BinOprType::Gt)
                    ],
                    span: SpanMock::new(),
                },
                /* return fib(n-2) + fib(n-1) */
                body: vec![NodeStmnt::RetStmnt(NodeRetStmnt {
                    value: NodeExpr {
                        expr: vecdeq![
                            NodeExprInner::Resolution(NodeAttrRes {
                                resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                                    name: "fib".to_string(),
                                    namespace: None,
                                    args: vec![NodeExpr {
                                        expr: vecdeq![
                                            NodeExprInner::Resolution(NodeAttrRes {
                                                resolution: vec![NodeAttribute::VarCall(
                                                    NodeVarCall {
                                                        name: "n".to_string(),
                                                        span: SpanMock::new(),
                                                    }
                                                )],
                                                span: SpanMock::new(),
                                            }),
                                            NodeExprInner::Value(NodeValue::Uint(2)),
                                            NodeExprInner::BinOpr(BinOprType::Sub)
                                        ],
                                        span: SpanMock::new()
                                    }],
                                    span: SpanMock::new(),
                                })],
                                span: SpanMock::new(),
                            }),
                            NodeExprInner::Resolution(NodeAttrRes {
                                resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                                    name: "fib".to_string(),
                                    namespace: None,
                                    args: vec![NodeExpr {
                                        expr: vecdeq![
                                            NodeExprInner::Resolution(NodeAttrRes {
                                                resolution: vec![NodeAttribute::VarCall(
                                                    NodeVarCall {
                                                        name: "n".to_string(),
                                                        span: SpanMock::new(),
                                                    }
                                                )],
                                                span: SpanMock::new(),
                                            }),
                                            NodeExprInner::Value(NodeValue::Uint(1)),
                                            NodeExprInner::BinOpr(BinOprType::Sub)
                                        ],
                                        span: SpanMock::new()
                                    }],
                                    span: SpanMock::new(),
                                })],
                                span: SpanMock::new(),
                            }),
                            NodeExprInner::BinOpr(BinOprType::Add)
                        ],
                        span: SpanMock::new(),
                    },
                    span: SpanMock::new(),
                })],
                branches: vec![],
            }),
            /* return 1 */
            NodeStmnt::RetStmnt(NodeRetStmnt {
                value: NodeExpr {
                    expr: vecdeq![NodeExprInner::Value(NodeValue::Uint(1))],
                    span: SpanMock::new(),
                },
                span: SpanMock::new(),
            }),
        ],
        span: SpanMock::new(),
    };

    assert_eq!(exp, recv);
}

#[test]
fn parse_if_statement() {
    // equivalent to the code:
    // ```
    // if 2 > 3:
    //     print('one')
    // elif 3 > 4:
    //     print('two')
    // else:
    //     print('default')
    // ```
    let code = line_reader!(
        line!(
            0,
            TokenKind::Keyword(Keyword::If),
            TokenKind::Uint(2),
            TokenKind::Operator(Operator::Gt),
            TokenKind::Uint(3),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("print".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Str("one".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar)
        ),
        line!(
            0,
            TokenKind::Keyword(Keyword::Elif),
            TokenKind::Uint(3),
            TokenKind::Operator(Operator::Gt),
            TokenKind::Uint(4),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("print".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Str("two".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar)
        ),
        line!(
            0,
            TokenKind::Keyword(Keyword::Else),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("print".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Str("default".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar)
        )
    );

    let recv = NodeIfStmnt::new(code).expect("did not parse NodeIfStmnt");

    let exp = NodeIfStmnt {
        condition: NodeExpr {
            expr: vecdeq![
                NodeExprInner::Value(NodeValue::Uint(2)),
                NodeExprInner::Value(NodeValue::Uint(3)),
                NodeExprInner::BinOpr(BinOprType::Gt)
            ],
            span: SpanMock::new(),
        },
        body: vec![NodeStmnt::FuncCall(NodeFuncCallStmnt(NodeAttrRes {
            resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                name: "print".to_string(),
                namespace: None,
                args: vec![NodeExpr {
                    expr: vecdeq![NodeExprInner::Value(NodeValue::Str("one".to_string()))],
                    span: SpanMock::new(),
                }],
                span: SpanMock::new(),
            })],
            span: SpanMock::new(),
        }))],
        branches: vec![
            NodeIfBranch::Elif(NodeElifStmnt {
                condition: NodeExpr {
                    expr: vecdeq![
                        NodeExprInner::Value(NodeValue::Uint(3)),
                        NodeExprInner::Value(NodeValue::Uint(4)),
                        NodeExprInner::BinOpr(BinOprType::Gt)
                    ],
                    span: SpanMock::new(),
                },
                body: vec![NodeStmnt::FuncCall(NodeFuncCallStmnt(NodeAttrRes {
                    resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                        name: "print".to_string(),
                        namespace: None,
                        args: vec![NodeExpr {
                            expr: vecdeq![NodeExprInner::Value(NodeValue::Str("two".to_string()))],
                            span: SpanMock::new(),
                        }],
                        span: SpanMock::new(),
                    })],
                    span: SpanMock::new(),
                }))],
            }),
            NodeIfBranch::Else(NodeElseStmnt {
                body: vec![NodeStmnt::FuncCall(NodeFuncCallStmnt(NodeAttrRes {
                    resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                        name: "print".to_string(),
                        namespace: None,
                        args: vec![NodeExpr {
                            expr: vecdeq![NodeExprInner::Value(NodeValue::Str(
                                "default".to_string()
                            ))],
                            span: SpanMock::new(),
                        }],
                        span: SpanMock::new(),
                    })],
                    span: SpanMock::new(),
                }))],
            }),
        ],
    };

    assert_eq!(exp, recv);
}

#[test]
fn parse_while_statement() {
    // equivalent to the code:
    // ```
    // while !(2 < 3):
    //     print("something's wrong")
    //     break
    //     continue
    // ```
    let code = line_reader!(
        line!(
            0,
            TokenKind::Keyword(Keyword::While),
            TokenKind::Operator(Operator::Bang),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Uint(2),
            TokenKind::Operator(Operator::Lt),
            TokenKind::Uint(3),
            TokenKind::Delimiter(Delimiter::ClosePar),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("print".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Str("something's wrong".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar)
        ),
        line!(4, TokenKind::Keyword(Keyword::Break)),
        line!(4, TokenKind::Keyword(Keyword::Continue))
    );

    let recv = NodeWhileLoop::new(code).expect("did not parse NodeWhileLoop");

    let exp = NodeWhileLoop {
        condition: NodeExpr {
            expr: vecdeq![
                NodeExprInner::Value(NodeValue::Uint(2)),
                NodeExprInner::Value(NodeValue::Uint(3)),
                NodeExprInner::BinOpr(BinOprType::Lt),
                NodeExprInner::UnaryOpr(UnaryOprType::Bang)
            ],
            span: SpanMock::new(),
        },
        body: vec![
            NodeStmnt::FuncCall(NodeFuncCallStmnt(NodeAttrRes {
                resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                    name: "print".to_string(),
                    namespace: None,
                    args: vec![NodeExpr {
                        expr: vecdeq![NodeExprInner::Value(NodeValue::Str(
                            "something's wrong".to_string()
                        ))],
                        span: SpanMock::new(),
                    }],
                    span: SpanMock::new(),
                })],
                span: SpanMock::new(),
            })),
            NodeStmnt::BreakStmnt(NodeBreakStmnt {
                span: SpanMock::new(),
            }),
            NodeStmnt::ContStmnt(NodeContStmnt {
                span: SpanMock::new(),
            }),
        ],
    };

    assert_eq!(exp, recv);
}

#[test]
fn parse_try_catch_block() {
    // equivalent to the code:
    // ```
    // try:
    //     print(21 * 2)
    //     throw 'unexpected error'
    // catch (exc: exception):
    //     print('Received the exception: ' + exc)
    // ```
    let code = line_reader!(
        line!(
            0,
            TokenKind::Keyword(Keyword::Try),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("print".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Uint(21),
            TokenKind::Operator(Operator::Mul),
            TokenKind::Uint(2),
            TokenKind::Delimiter(Delimiter::ClosePar)
        ),
        line!(
            4,
            TokenKind::Keyword(Keyword::Throw),
            TokenKind::Str("unexpected error".to_string())
        ),
        line!(
            0,
            TokenKind::Keyword(Keyword::Catch),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("exc".to_string()),
            TokenKind::Special(Special::Colon),
            TokenKind::Type(Type::Exception),
            TokenKind::Delimiter(Delimiter::ClosePar),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("print".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Str("Received the exception: ".to_string()),
            TokenKind::Operator(Operator::Add),
            TokenKind::Identifier("exc".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar)
        )
    );

    let recv = NodeTryCatch::new(code).expect("did not compile NodeTryCatch");

    let exp = NodeTryCatch {
        try_body: vec![
            NodeStmnt::FuncCall(NodeFuncCallStmnt(NodeAttrRes {
                resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                    name: "print".to_string(),
                    namespace: None,
                    args: vec![NodeExpr {
                        expr: vecdeq![
                            NodeExprInner::Value(NodeValue::Uint(21)),
                            NodeExprInner::Value(NodeValue::Uint(2)),
                            NodeExprInner::BinOpr(BinOprType::Mul)
                        ],
                        span: SpanMock::new(),
                    }],
                    span: SpanMock::new(),
                })],
                span: SpanMock::new(),
            })),
            NodeStmnt::Throw(NodeThrow(NodeExpr {
                expr: vecdeq![NodeExprInner::Value(NodeValue::Str(
                    "unexpected error".to_string()
                ))],
                span: SpanMock::new(),
            })),
        ],
        try_span: SpanMock::new(),
        exception_var: NodeVarCall {
            name: "exc".to_string(),
            span: SpanMock::new(),
        },
        catch_body: vec![NodeStmnt::FuncCall(NodeFuncCallStmnt(NodeAttrRes {
            resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                name: "print".to_string(),
                namespace: None,
                args: vec![NodeExpr {
                    expr: vecdeq![
                        NodeExprInner::Value(NodeValue::Str(
                            "Received the exception: ".to_string()
                        )),
                        NodeExprInner::Resolution(NodeAttrRes {
                            resolution: vec![NodeAttribute::VarCall(NodeVarCall {
                                name: "exc".to_string(),
                                span: SpanMock::new()
                            })],
                            span: SpanMock::new(),
                        }),
                        NodeExprInner::BinOpr(BinOprType::Add)
                    ],
                    span: SpanMock::new(),
                }],
                span: SpanMock::new(),
            })],
            span: SpanMock::new(),
        }))],
    };

    assert_eq!(exp, recv);
}

#[test]
fn parse_list() {
    // equivalent to the code:
    // ```
    // let a = [1, 2 * 3, (4 + 10) / 2] * 5
    // ```

    let code = token_reader!(
        TokenKind::Keyword(Keyword::Let),
        TokenKind::Identifier("a".to_string()),
        TokenKind::Operator(Operator::Eq),
        TokenKind::Delimiter(Delimiter::OpenBracket),
        TokenKind::Uint(1),
        TokenKind::Special(Special::Comma),
        TokenKind::Uint(2),
        TokenKind::Operator(Operator::Mul),
        TokenKind::Uint(3),
        TokenKind::Special(Special::Comma),
        TokenKind::Delimiter(Delimiter::OpenPar),
        TokenKind::Uint(4),
        TokenKind::Operator(Operator::Add),
        TokenKind::Uint(10),
        TokenKind::Delimiter(Delimiter::ClosePar),
        TokenKind::Operator(Operator::Div),
        TokenKind::Uint(2),
        TokenKind::Delimiter(Delimiter::CloseBracket),
        TokenKind::Operator(Operator::Mul),
        TokenKind::Uint(5)
    );

    let recv = NodeVarDef::new(code).expect("could not parse NodeVarDef");

    let exp = NodeVarDef {
        name: "a".to_string(),
        ty: Type::Any,
        is_const: false,
        value: NodeExpr {
            expr: vecdeq![
                NodeExprInner::List(NodeList {
                    elements: vec![
                        NodeExpr {
                            expr: vecdeq![NodeExprInner::Value(NodeValue::Uint(1))],
                            span: SpanMock::new()
                        },
                        NodeExpr {
                            expr: vecdeq![
                                NodeExprInner::Value(NodeValue::Uint(2)),
                                NodeExprInner::Value(NodeValue::Uint(3)),
                                NodeExprInner::BinOpr(BinOprType::Mul)
                            ],
                            span: SpanMock::new()
                        },
                        NodeExpr {
                            expr: vecdeq![
                                NodeExprInner::Value(NodeValue::Uint(4)),
                                NodeExprInner::Value(NodeValue::Uint(10)),
                                NodeExprInner::BinOpr(BinOprType::Add),
                                NodeExprInner::Value(NodeValue::Uint(2)),
                                NodeExprInner::BinOpr(BinOprType::Div),
                            ],
                            span: SpanMock::new()
                        }
                    ],
                    span: SpanMock::new(),
                }),
                NodeExprInner::Value(NodeValue::Uint(5)),
                NodeExprInner::BinOpr(BinOprType::Mul),
            ],
            span: SpanMock::new(),
        },
        span: SpanMock::new(),
    };

    assert_eq!(exp, recv);
}

#[test]
fn parse_class_def() {
    // equivalent to the code:
    // ```
    // class Example:
    //     arg: uint
    //
    //     fn new(val: uint) -> Example:
    //         return Example { arg: val }
    //
    //     fn compute(self) -> uint:
    //         return fib(self.arg)
    // ```

    let code = line_reader!(
        line!(
            0,
            TokenKind::Keyword(Keyword::Class),
            TokenKind::Identifier("Example".to_string()),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            4,
            TokenKind::Identifier("arg".to_string()),
            TokenKind::Special(Special::Colon),
            TokenKind::Type(Type::Uint)
        ),
        line!(
            4,
            TokenKind::Keyword(Keyword::Fn),
            TokenKind::Identifier("new".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("val".to_string()),
            TokenKind::Special(Special::Colon),
            TokenKind::Type(Type::Uint),
            TokenKind::Delimiter(Delimiter::ClosePar),
            TokenKind::Special(Special::RightArrow),
            TokenKind::Identifier("Example".to_string()),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            8,
            TokenKind::Keyword(Keyword::Return),
            TokenKind::Identifier("Example".to_string()),
            TokenKind::Delimiter(Delimiter::OpenBrace),
            TokenKind::Identifier("arg".to_string()),
            TokenKind::Special(Special::Colon),
            TokenKind::Identifier("val".to_string()),
            TokenKind::Delimiter(Delimiter::CloseBrace)
        ),
        line!(
            4,
            TokenKind::Keyword(Keyword::Fn),
            TokenKind::Identifier("compute".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("self".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar),
            TokenKind::Special(Special::RightArrow),
            TokenKind::Type(Type::Uint),
            TokenKind::Special(Special::Colon)
        ),
        line!(
            8,
            TokenKind::Keyword(Keyword::Return),
            TokenKind::Identifier("fib".to_string()),
            TokenKind::Delimiter(Delimiter::OpenPar),
            TokenKind::Identifier("self".to_string()),
            TokenKind::Special(Special::Dot),
            TokenKind::Identifier("arg".to_string()),
            TokenKind::Delimiter(Delimiter::ClosePar)
        )
    );

    let recv = NodeClass::new(code).expect("did not parse NodeClass");

    let exp = NodeClass {
        name: "Example".to_string(),
        members: vec![Member {
            name: "arg".to_string(),
            ty: Type::Uint,
            span: SpanMock::new(),
        }],
        methods: vec![
            NodeFuncDef {
                name: "new".to_string(),
                ret_type: Type::Custom(Box::new("Example".to_string())),
                namespace: Some("Example".to_string()),
                args: vecdeq![Arg {
                    name: "val".to_string(),
                    ty: Type::Uint
                }],
                body: vec![NodeStmnt::RetStmnt(NodeRetStmnt {
                    value: NodeExpr {
                        expr: vecdeq![NodeExprInner::InlineClass(NodeInlineClass {
                            class: "Example".to_string(),
                            members: hash_map!(
                            "arg".to_string() => (NodeExpr {
                                expr: vecdeq![NodeExprInner::Resolution(NodeAttrRes{
                                    resolution: vec![NodeAttribute::VarCall(
                                                    NodeVarCall {
                                                        name: "val".to_string(),
                                                        span: SpanMock::new(),
                                                    }
                                                    )],
                                    span: SpanMock::new(),
                                })],
                                span: SpanMock::new(),
                            }, SpanMock::new())
                            ),
                            span: SpanMock::new(),
                        })],
                        span: SpanMock::new(),
                    },
                    span: SpanMock::new(),
                })],
                span: SpanMock::new(),
            },
            NodeFuncDef {
                name: "compute".to_string(),
                ret_type: Type::Uint,
                namespace: Some("Example".to_string()),
                args: vecdeq![Arg {
                    name: "self".to_string(),
                    ty: Type::Custom(Box::new("Example".to_string()))
                }],
                body: vec![NodeStmnt::RetStmnt(NodeRetStmnt {
                    value: NodeExpr {
                        expr: vecdeq![NodeExprInner::Resolution(NodeAttrRes {
                            resolution: vec![NodeAttribute::FuncCall(NodeFuncCall {
                                name: "fib".to_string(),
                                namespace: None,
                                args: vec![NodeExpr {
                                    expr: vecdeq![NodeExprInner::Resolution(NodeAttrRes {
                                        resolution: vec![
                                            NodeAttribute::VarCall(NodeVarCall {
                                                name: "self".to_string(),
                                                span: SpanMock::new(),
                                            }),
                                            NodeAttribute::VarCall(NodeVarCall {
                                                name: "arg".to_string(),
                                                span: SpanMock::new()
                                            })
                                        ],
                                        span: SpanMock::new(),
                                    })],
                                    span: SpanMock::new(),
                                }],
                                span: SpanMock::new(),
                            })],
                            span: SpanMock::new(),
                        })],
                        span: SpanMock::new(),
                    },
                    span: SpanMock::new(),
                })],
                span: SpanMock::new(),
            },
        ],
        span: SpanMock::new(),
    };

    assert_eq!(exp, recv);
}
