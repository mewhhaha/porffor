#[test]
fn stages_async_generator_array_spread_yields_in_source_order() {
    let program = lower_script("async function* stream() { yield [0, ...yield, 3]; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator should be registered");

    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .expect("async generator should have a resumable plan")
            .state_count,
        3
    );
    let [StatementIr::LexicalBlock(statements)] = function.body.statements.as_slice() else {
        panic!(
            "expected one staged array lexical block, got {:#?}",
            function.body.statements
        );
    };
    let [array_init, next_index_init, prefix, received_init, inner_yield, outer_received_init, outer_yield, completion] =
        statements.as_slice()
    else {
        panic!("expected the exact staged array sequence, got {statements:#?}");
    };

    let StatementIr::Lexical {
        name: array_binding,
        init:
            TypedExpr {
                expr: ExprIr::ArrayLiteral(initial_elements),
                ..
            },
        ..
    } = array_init
    else {
        panic!("expected the accumulator array initialization, got {array_init:#?}");
    };
    assert!(initial_elements.is_empty());
    let StatementIr::Lexical {
        name: next_index_binding,
        init:
            TypedExpr {
                expr: ExprIr::Number(initial_next_index),
                ..
            },
        ..
    } = next_index_init
    else {
        panic!("expected the accumulator index initialization, got {next_index_init:#?}");
    };
    assert_eq!(*initial_next_index, 0.0f64.to_bits());
    assert_ne!(array_binding, next_index_binding);

    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::ArrayAccumulation(prefix),
        ..
    }) = prefix
    else {
        panic!("expected the pre-suspension accumulation, got {prefix:#?}");
    };
    let ArrayAccumulationTargetIr::SuspensionOwned(prefix_slots) = prefix.target() else {
        panic!(
            "expected suspension-owned prefix, got {:#?}",
            prefix.target()
        );
    };
    assert_eq!(prefix_slots.array().as_str(), array_binding);
    assert_eq!(prefix_slots.next_index().as_str(), next_index_binding);
    assert!(matches!(
        prefix.elements(),
        [ArrayAccumulationElementIr::Value(TypedExpr {
            expr: ExprIr::Number(value),
            ..
        })] if *value == 0.0f64.to_bits()
    ));

    let StatementIr::Lexical {
        name: received_binding,
        init: TypedExpr {
            expr: ExprIr::Undefined,
            ..
        },
        ..
    } = received_init
    else {
        panic!("expected the yielded-value binding, got {received_init:#?}");
    };
    let StatementIr::GeneratorYield {
        value: TypedExpr {
            expr: ExprIr::Undefined,
            ..
        },
        resume_mode: GeneratorResumeModeIr::AssignIdentifier(inner_resume_binding),
        ..
    } = inner_yield
    else {
        panic!("expected the inner array suspension, got {inner_yield:#?}");
    };
    assert_eq!(inner_resume_binding, received_binding);

    let StatementIr::GeneratorYield {
        value:
            TypedExpr {
                expr: ExprIr::ArrayAccumulation(final_accumulation),
                ..
            },
        resume_mode: GeneratorResumeModeIr::AssignIdentifier(outer_received),
        ..
    } = outer_yield
    else {
        panic!("expected the final accumulated array yield, got {outer_yield:#?}");
    };
    assert!(
        matches!(outer_received_init, StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. } if name == outer_received)
    );
    assert!(
        matches!(completion, StatementIr::Expression(TypedExpr { expr: ExprIr::Identifier(name), .. }) if name == outer_received)
    );
    let ArrayAccumulationTargetIr::SuspensionOwned(final_slots) = final_accumulation.target()
    else {
        panic!(
            "expected suspension-owned final accumulation, got {:#?}",
            final_accumulation.target()
        );
    };
    assert_eq!(final_slots, prefix_slots);
    let [ArrayAccumulationElementIr::Spread(ArraySpreadIr {
        value: spread_value,
        protocol,
    }), ArrayAccumulationElementIr::Value(TypedExpr {
        expr: ExprIr::Number(final_value),
        ..
    })] = final_accumulation.elements()
    else {
        panic!(
            "expected resumed spread followed by the suffix, got {:#?}",
            final_accumulation.elements()
        );
    };
    assert_eq!(*protocol, ArraySpreadProtocol::ARRAY_ACCUMULATION);
    assert!(matches!(
        &spread_value.expr,
        ExprIr::Identifier(spread_binding) if spread_binding == received_binding
    ));
    assert_eq!(*final_value, 3.0f64.to_bits());
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == array_binding.as_str()));
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == next_index_binding.as_str()));
}

#[test]
fn stages_async_generator_object_spread_yields_in_source_order() {
    let program =
        lower_script("async function* stream() { yield { ...yield, fixed: 1, ...yield yield }; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator should be registered");

    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .expect("async generator should have a resumable plan")
            .state_count,
        5
    );
    let statements = async_loop_rows(&function.body.statements);
    let object = statements.iter().find_map(|statement| match statement {
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::ObjectLiteral(properties), .. }, .. }
            if properties.is_empty() => Some(name),
        _ => None,
    }).expect("the original object accumulator is allocated before all spreads");
    assert!(function.owned_env_bindings.iter().any(|binding| &binding.name == object));
    let definitions = statements.iter().filter_map(|statement| match statement {
        StatementIr::Expression(TypedExpr { expr: ExprIr::ObjectPropertyDefinition(definition), .. }) => Some(definition),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(definitions.len(), 3);
    assert!(definitions.iter().all(|definition| matches!(&definition.target().expr, ExprIr::Identifier(name) if name == object)));
    assert_eq!(statements.iter().filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. })).count(), 4);
}

#[test]
fn rejects_async_generator_spread_with_conditional_yield() {
    let program =
        lower_script("async function* stream(flag) { yield [...(flag ? yield [] : [])]; }");

    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program.script.as_ref().unwrap().functions.iter().find(|function| function.name == "stream").unwrap();
    let statements = async_loop_rows(&function.body.statements);
    let branch = statements.iter().find_map(|statement| match statement {
        StatementIr::AsyncGeneratorIf(plan) => Some(plan),
        _ => None,
    }).expect("the conditional spread keeps its checked branch owner");
    assert!(branch.then_branch().end_state() > branch.then_branch().entry_state());
    assert_eq!(branch.else_branch().entry_state(), branch.else_branch().end_state());
    let yields = statements.iter().filter_map(|statement| match statement {
        StatementIr::GeneratorYield { suspend_state, resume_state, .. } => Some((*suspend_state, *resume_state)),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(yields.len(), 2);
    assert!(yields[0].0 >= branch.then_branch().entry_state());
    assert!(yields[0].1 <= branch.then_branch().end_state());
    assert!(yields[1].0 >= branch.exit_state());
    assert!(function.resumable_plan.as_ref().unwrap().suspension_points.iter().all(|point| point.kind == ResumableSuspensionKindIr::Yield));
}

#[test]
fn first_async_generator_request_preserves_linear_body_start_states() {
    let program = lower_script(
        "let started = false;
             async function* stream(source) {
                 started = true;
                 await source;
                 yield source;
             }
             const iterator = stream(1);
             iterator.next();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");

    assert_eq!(
        function.resumable_plan,
        Some(ResumablePlanIr {
            entry_state: 0,
            state_count: 3,
            suspension_points: vec![
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Await,
                    suspend_state: 0,
                    resume_state: 1,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Yield,
                    suspend_state: 1,
                    resume_state: 2,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
            ],
            ..function.resumable_plan.as_ref().unwrap().clone()
        })
    );
    assert!(function.body.statements.iter().any(|statement| {
        matches!(statement, StatementIr::Expression(_))
            || matches!(
                statement,
                StatementIr::LexicalBlock(statements)
                    if statements
                        .iter()
                        .any(|statement| matches!(statement, StatementIr::Expression(_)))
            )
    }));
}

#[test]
fn allocates_mixed_async_generator_suspension_states_without_collisions() {
    let program = lower_script(
        "async function* stream(source) {
                 await source.ready;
                 yield 1;
                 for await (const value of source) { yield value; }
                 await source.done;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");

    assert_eq!(
        function.resumable_plan,
        Some(ResumablePlanIr {
            entry_state: 0,
            state_count: 13,
            suspension_points: vec![
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Await,
                    suspend_state: 0,
                    resume_state: 1,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Yield,
                    suspend_state: 1,
                    resume_state: 2,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::ForAwaitNext,
                    suspend_state: 4,
                    resume_state: 5,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Yield,
                    suspend_state: 7,
                    resume_state: 8,
                    resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
                },
                // The close phase follows the complete head, per-key
                // initialization and body. Its resume cannot alias Next
                // or the body Yield, including when the body is eager.
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::ForAwaitClose,
                    suspend_state: 9,
                    resume_state: 10,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Await,
                    suspend_state: 11,
                    resume_state: 12,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
            ],
            ..function.resumable_plan.as_ref().unwrap().clone()
        })
    );
    let [StatementIr::LexicalBlock(before_await), StatementIr::LexicalBlock(before_yield), StatementIr::AsyncGeneratorForOf(async_plan), StatementIr::LexicalBlock(after_await)] = function.body.statements.as_slice()
    else {
        panic!(
            "async-generator body should preserve await, yield, for-await, await: {:?}",
            function.body.statements
        );
    };
    assert!(before_await.iter().any(|row| matches!(
        row,
        StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            ..
        }
    )));
    assert!(before_yield.iter().any(|row| matches!(
        row,
        StatementIr::GeneratorYield {
            suspend_state: 1,
            resume_state: 2,
            ..
        }
    )));
    assert!(after_await.iter().any(|row| matches!(
        row,
        StatementIr::AsyncAwait {
            suspend_state: 11,
            resume_state: 12,
            ..
        }
    )));
    assert_eq!(async_plan.entry_state(), 2);
    assert_eq!(async_plan.execution(), ResumableRegionProtocolIr::AsyncGenerator);
    assert_eq!(async_plan.initialization().entry_state(), 6);
    assert_eq!(async_plan.body().entry_state(), 7);
    assert_eq!(async_plan.body().end_state(), 8);
    assert_eq!(async_plan.exit_state(), 11);
    assert!(matches!(async_plan.protocol(), AsyncGeneratorIteratorProtocolIr::Awaited {
        next_suspend_state: 4, next_resume_state: 5, close_suspend_state: 9, close_resume_state: 10,
    }));
    assert!(async_loop_rows(&async_plan.body().block().statements).into_iter().any(|statement|
        matches!(statement, StatementIr::GeneratorYield { suspend_state: 7, resume_state: 8, .. })));
}

#[test]
fn allocates_disjoint_for_await_states_when_the_body_never_suspends() {
    // The four states a for-await loop re-enters on must be pairwise
    // distinct; the backend's entry test admits three of them and then
    // re-dispatches on which one it saw, so an alias silently routes a
    // `next()` resume into the iterator-close path.
    let program = lower_script(
        "async function* stream(source) {
                 for await (const value of source) { sink(value); }
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");

    let [StatementIr::AsyncGeneratorForOf(async_plan)] = function.body.statements.as_slice()
    else {
        panic!(
            "async-generator body should be a single for-await: {:?}",
            function.body.statements
        );
    };
    let AsyncGeneratorIteratorProtocolIr::Awaited { next_resume_state, close_resume_state, .. } = async_plan.protocol() else {
        panic!("for-await owns Next and Close Await phases");
    };
    let states = [async_plan.entry_state(), next_resume_state, close_resume_state, async_plan.exit_state()];
    assert_eq!(states, [0, 3, 7, 8], "{async_plan:?}");
    assert_eq!(states.into_iter().collect::<BTreeSet<_>>().len(), 4);
    assert_eq!((async_plan.initialization().entry_state(), async_plan.body().entry_state(), async_plan.body().end_state()), (4, 5, 5));
    for binding in [async_plan.head_binding(), async_plan.incoming_binding(), async_plan.value_binding()] {
        assert!(function.owned_env_bindings.contains(binding));
    }
}

#[test]
fn async_generator_return_without_value_completes_without_implicit_await() {
    let program = lower_script("async function* stream() { return; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");

    assert_eq!(
        function.resumable_plan,
        Some(ResumablePlanIr {
            entry_state: 0,
            state_count: 1,
            suspension_points: Vec::new(),
            ..function.resumable_plan.as_ref().unwrap().clone()
        })
    );
    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::Return(TypedExpr {
            kind: ValueKind::Undefined,
            ..
        })]
    ));
}

#[test]
fn async_generator_return_value_ends_with_implicit_await() {
    let program = lower_script("async function* stream(value) { return value; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");

    assert_eq!(
        function.resumable_plan,
        Some(ResumablePlanIr {
            entry_state: 0,
            state_count: 2,
            suspension_points: vec![ResumableSuspensionPointIr {
                kind: ResumableSuspensionKindIr::Await,
                suspend_state: 0,
                resume_state: 1,
                resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
            }],
            ..function.resumable_plan.as_ref().unwrap().clone()
        })
    );
    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            resume_mode: AsyncResumeModeIr::Return,
            ..
        }]
    ));
}

#[test]
fn async_generator_return_await_yield_orders_yield_before_both_awaits() {
    let program = lower_script("async function* stream(value) { return await (yield value); }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");
    let plan = function
        .resumable_plan
        .as_ref()
        .expect("async generator should have a resumable plan");

    assert_eq!(plan.state_count, 4);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|suspension| suspension.kind)
            .collect::<Vec<_>>(),
        vec![
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Await,
        ]
    );
    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield {
                        suspend_state: 0,
                        resume_state: 1,
                        resume_mode: GeneratorResumeModeIr::AssignIdentifier(_),
                        ..
                    },
                    StatementIr::Lexical { .. },
                    StatementIr::AsyncAwait {
                        suspend_state: 1,
                        resume_state: 2,
                        resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                        ..
                    },
                    StatementIr::AsyncAwait {
                        suspend_state: 2,
                        resume_state: 3,
                        resume_mode: AsyncResumeModeIr::Return,
                        ..
                    }
                ]
            )
    ));
}

#[test]
fn async_generator_return_yield_await_orders_await_before_yield_and_return_await() {
    let program = lower_script("async function* stream(value) { return yield (await value); }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");
    let plan = function
        .resumable_plan
        .as_ref()
        .expect("async generator should have a resumable plan");

    assert_eq!(plan.state_count, 4);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|suspension| suspension.kind)
            .collect::<Vec<_>>(),
        vec![
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
        ]
    );
    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical { .. },
                    StatementIr::AsyncAwait {
                        suspend_state: 0,
                        resume_state: 1,
                        resume_mode: AsyncResumeModeIr::AssignIdentifier(_),
                        ..
                    },
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield {
                        suspend_state: 1,
                        resume_state: 2,
                        resume_mode: GeneratorResumeModeIr::AssignIdentifier(_),
                        ..
                    },
                    StatementIr::AsyncAwait {
                        suspend_state: 2,
                        resume_state: 3,
                        resume_mode: AsyncResumeModeIr::Return,
                        ..
                    }
                ]
            )
    ));
}

/// A composite `return` operand used to be refused outright; it now stages
/// through the ordinary async prefix, so the `await` inside it becomes its
/// own suspension and the residual `+` is what the implicit return awaits.
#[test]
fn async_generator_return_stages_composite_suspension_boundaries() {
    let program =
        lower_script("async function* stream(left, right) { return (await left) + right; }");

    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");
    let plan = function
        .resumable_plan
        .as_ref()
        .expect("async generator should have a resumable plan");

    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|suspension| suspension.kind)
            .collect::<Vec<_>>(),
        vec![
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Await,
        ]
    );
    let [StatementIr::LexicalBlock(statements)] = function.body.statements.as_slice() else {
        panic!(
            "staged return should lower to one block: {:?}",
            function.body.statements
        );
    };
    let awaits = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                resume_mode,
                ..
            } => Some((*suspend_state, *resume_state, resume_mode)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        matches!(
            awaits.as_slice(),
            [
                (0, 1, AsyncResumeModeIr::AssignIdentifier(_)),
                (1, 2, AsyncResumeModeIr::Return),
            ]
        ),
        "{awaits:?} from {statements:?}"
    );
}
