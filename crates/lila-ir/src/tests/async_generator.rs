#[test]
fn collects_nested_async_generator_declarations_with_exact_source() {
    let declaration = "async function* stream(source) { yield await source; }";
    let program = lower_script(&format!(
        "function outer() {{ {declaration} return stream; }}"
    ));
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("nested async generator declaration should be collected");

    assert_eq!(
        function.protocol.execution_kind(),
        FunctionExecutionKind::AsyncGenerator
    );
    assert!(!function.protocol.is_constructable());
    assert_eq!(
        function.to_string_representation,
        CallableToStringRepresentation::ExactSource(declaration.to_string())
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
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
                        ..
                    },
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield {
                        suspend_state: 1,
                        resume_state: 2,
                        ..
                    },
                    StatementIr::Expression(_)
                ]
            )
    ));
}

#[test]
fn async_generator_yield_await_stages_the_awaited_binding_into_yield() {
    let program = lower_script("async function* stream(source) { yield await source; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "stream")
        .expect("async generator declaration should be collected");

    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical { name: binding, .. },
                    StatementIr::AsyncAwait {
                        resume_mode: AsyncResumeModeIr::AssignIdentifier(await_binding),
                        ..
                    },
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield {
                        value: TypedExpr {
                            expr: ExprIr::Identifier(yield_binding),
                            ..
                        },
                        form: YieldForm::Plain,
                        ..
                    },
                    StatementIr::Expression(_)
                ] if binding == await_binding
                    && await_binding == yield_binding
            )
    ));
}

#[test]
fn async_generator_yield_star_preserves_the_delegation_boundary() {
    let program = lower_script("async function* outer(source) { yield* source; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "outer")
        .expect("async generator declaration should be collected");

    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)] if matches!(statements.as_slice(), [
        StatementIr::Lexical { .. },
        StatementIr::GeneratorYield {
            value: TypedExpr {
                expr: ExprIr::Identifier(source),
                ..
            },
            form: YieldForm::Delegate(_),
            suspend_state: 0,
            resume_state: 1,
            ..
        }, StatementIr::Expression(_)] if source == "source")
    ));
}

#[test]
fn async_generator_yield_star_assigns_its_completion_to_var() {
    let program = lower_script(
        "async function* outer(source) { var completion = yield* source; return completion; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "outer")
        .expect("async generator declaration should be collected");

    assert!(
        matches!(
            function.body.statements.as_slice(),
            [
                StatementIr::LexicalBlock(statements),
                StatementIr::AsyncAwait {
                    value: TypedExpr {
                        expr: ExprIr::Identifier(completion),
                        ..
                    },
                    resume_mode: AsyncResumeModeIr::Return,
                    ..
                }
            ] if matches!(
                statements.as_slice(),
                [
                    StatementIr::Var(declarations),
                    StatementIr::Lexical { .. },
                    StatementIr::Expression(TypedExpr { expr: ExprIr::EnvironmentIdentifier(capture), .. }),
                    StatementIr::Lexical { name: received, .. },
                    StatementIr::GeneratorYield {
                        form: YieldForm::Delegate(_),
                        resume_mode: GeneratorResumeModeIr::AssignIdentifier(binding),
                        ..
                    },
                    StatementIr::DeclarationEvaluation(TypedExpr { expr: ExprIr::EnvironmentIdentifier(write), .. })
                ] if matches!(declarations.as_slice(), [VarDeclaratorIr { name, init: None }] if name == completion)
                    && received == binding
                    && &capture.name == completion
                    && &write.name == completion
                    && matches!(&capture.operation, EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } if capture.access() == IdentifierReferenceCaptureAccess::WriteOnly)
                    && matches!(&write.operation, EnvironmentIdentifierOperationIr::PutCapturedReference { value, .. } if matches!(&value.expr, ExprIr::Identifier(value) if value == binding))
            )
        ),
        "{:#?}",
        function.body.statements
    );
}

#[test]
fn async_generator_yield_star_initializes_lexical_binding_after_delegation() {
    let program = lower_script(
        "async function* outer(source) { const completion = yield* source; return completion; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "outer")
        .expect("async generator declaration should be collected");

    assert!(
        matches!(
            function.body.statements.as_slice(),
            [
                StatementIr::LexicalBlock(statements),
                StatementIr::AsyncAwait {
                    value: TypedExpr {
                        expr: ExprIr::Identifier(returned_completion),
                        ..
                    },
                    resume_mode: AsyncResumeModeIr::Return,
                    ..
                }
            ] if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical {
                        mode: BindingMode::Let,
                        name: staged_completion,
                        init: TypedExpr {
                            expr: ExprIr::Undefined,
                            ..
                        },
                    },
                    StatementIr::GeneratorYield {
                        form: YieldForm::Delegate(_),
                        resume_mode: GeneratorResumeModeIr::AssignIdentifier(received_completion),
                        ..
                    },
                    StatementIr::Lexical {
                        mode: BindingMode::Const,
                        name: lexical_completion,
                        init: TypedExpr {
                            expr: ExprIr::Identifier(initializer_completion),
                            ..
                        },
                    }
                ] if staged_completion == received_completion
                    && received_completion == initializer_completion
                    && lexical_completion == returned_completion
            )
        ),
        "{:#?}",
        function.body.statements
    );
}

#[test]
fn nested_async_generator_yields_consume_resumable_states_in_execution_order() {
    let program = lower_script("async function* stream() { yield yield 1; }");
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
                    kind: ResumableSuspensionKindIr::Yield,
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
                    StatementIr::GeneratorYield {
                        suspend_state: 1,
                        resume_state: 2,
                        resume_mode: GeneratorResumeModeIr::AssignIdentifier(_),
                        ..
                    },
                    StatementIr::Expression(_)
                ]
            )
    ));
}

#[test]
fn async_generator_yield_branches_reserve_a_distinct_merge_state() {
    let program =
        lower_script("async function* choose(flag) { if (flag) yield 1; else yield 2; yield 3; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("async generator declaration should be collected");

    assert_eq!(
        function.resumable_plan,
        Some(ResumablePlanIr {
            entry_state: 0,
            state_count: 7,
            suspension_points: vec![
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Yield,
                    suspend_state: 1,
                    resume_state: 2,
                    resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Yield,
                    suspend_state: 3,
                    resume_state: 4,
                    resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
                },
                ResumableSuspensionPointIr {
                    kind: ResumableSuspensionKindIr::Yield,
                    suspend_state: 5,
                    resume_state: 6,
                    resume_environment: ResumableResumeEnvironmentIr::SavedLexicalChain,
                },
            ],
            ..function.resumable_plan.as_ref().unwrap().clone()
        })
    );
    assert!(matches!(
        function.body.statements.as_slice(),
        [
            StatementIr::AsyncGeneratorIf(plan),
            StatementIr::LexicalBlock(statements)
        ] if plan.entry_state() == 0 && plan.exit_state() == 5
            && plan.then_branch().entry_state() == 1 && plan.then_branch().end_state() == 2
            && plan.else_branch().entry_state() == 3 && plan.else_branch().end_state() == 4
            && statements.iter().any(|row| matches!(row, StatementIr::GeneratorYield { suspend_state: 5, resume_state: 6, .. }))
    ));
}

#[test]
fn async_generator_loop_exits_into_the_next_preplanned_suspension() {
    let program = lower_script(
        "async function* stream() { for (let i = 0; i < 3; i++) { yield i; } yield 9; }",
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

    let [StatementIr::AsyncGeneratorLoop(plan), StatementIr::LexicalBlock(after)] =
        function.body.statements.as_slice()
    else {
        panic!(
            "expected resumable loop followed by the next yield: {:#?}",
            function.body.statements
        );
    };
    assert_eq!(
        (plan.entry_state(), plan.exit_state(), plan.continue_state()),
        (0, 5, 4)
    );
    assert_eq!((plan.body().entry_state(), plan.body().end_state()), (2, 3));
    assert!(
        matches!(plan.body().block().statements.as_slice(), [StatementIr::LexicalBlock(body)]
        if body.iter().any(|row| matches!(row, StatementIr::GeneratorYield { suspend_state: 2, resume_state: 3, form: YieldForm::Plain, .. })))
    );
    assert!(after.iter().any(|row| matches!(
        row,
        StatementIr::GeneratorYield {
            suspend_state: 5,
            resume_state: 6,
            ..
        }
    )));
}

#[test]
fn async_generator_classic_for_keeps_loop_lexicals_in_activation_storage() {
    let program = lower_script(
        "async function* stream() {
                for (let index = 0; index < 1; index++) {
                    let beforeYield = index;
                    yield beforeYield;
                    let afterYield = beforeYield;
                }
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

    let [StatementIr::AsyncGeneratorLoop(plan)] = function.body.statements.as_slice() else {
        panic!(
            "expected one resumable classic for loop: {:#?}",
            function.body.statements
        );
    };
    let mut declarations = Vec::new();
    fn bindings<'a>(items: &'a [StatementIr], names: &mut Vec<&'a String>) {
        for statement in items {
            match statement {
                StatementIr::Lexical { name, .. } => names.push(name),
                StatementIr::LexicalBlock(items) => bindings(items, names),
                StatementIr::EmptyStatementCompletion(item) => {
                    bindings(std::slice::from_ref(item.statement()), names)
                }
                _ => {}
            }
        }
    }
    bindings(
        &plan.initialization().unwrap().block().statements,
        &mut declarations,
    );
    let index = declarations
        .first()
        .copied()
        .expect("original head storage");
    declarations.clear();
    bindings(&plan.body().block().statements, &mut declarations);
    let before_yield = declarations.first().copied().expect("lexical before yield");
    let after_yield = declarations.last().copied().expect("lexical after yield");

    for name in [index, before_yield, after_yield] {
        assert!(
            function
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == *name),
            "`{name}` must live in the async-generator activation: {:?}",
            function.owned_env_bindings
        );
    }
}
