fn assert_eager_for_await_protocol(function: &FunctionIr, plan: &AsyncGeneratorForOfIr) {
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    let AsyncGeneratorIteratorProtocolIr::Awaited {
        next_suspend_state,
        next_resume_state,
        close_suspend_state,
        close_resume_state,
    } = plan.protocol()
    else {
        panic!("for-await owns the original awaited Iterator Record protocol");
    };
    assert_eq!(
        (
            plan.entry_state(),
            plan.acquisition_state(),
            plan.advance_state()
        ),
        (0, 1, 2)
    );
    assert_eq!((next_suspend_state, next_resume_state), (2, 3));
    assert_eq!(
        (
            plan.initialization().entry_state(),
            plan.initialization().end_state()
        ),
        (4, 4)
    );
    assert_eq!((plan.body().entry_state(), plan.body().end_state()), (5, 5));
    assert_eq!(
        (close_suspend_state, close_resume_state, plan.exit_state()),
        (6, 7, 8)
    );
    for binding in [
        plan.head_binding(),
        plan.incoming_binding(),
        plan.value_binding(),
    ] {
        assert!(function.owned_env_bindings.contains(binding));
    }
    assert_eq!(
        [
            plan.head_binding(),
            plan.incoming_binding(),
            plan.value_binding()
        ]
        .into_iter()
        .map(|binding| binding.slot)
        .collect::<BTreeSet<_>>()
        .len(),
        3
    );
    assert_eq!(
        plan.suspensions()
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::ForAwaitNext,
            ResumableSuspensionKindIr::ForAwaitClose
        ]
    );
}

#[test]
fn records_for_await_array_iterator_resume_boundaries_and_owned_state() {
    let program = lower_script(
        "async function collect() {
                 let total = 0;
                 for await (const value of [Promise.resolve(1), 2]) {
                     total += value;
                 }
                 return total;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| {
            function.name == "collect"
                && function.protocol.execution_kind() == FunctionExecutionKind::Async
        })
        .expect("async function should be registered");
    let plan = complete_async_for_of_plan(function);
    assert_eager_for_await_protocol(function, plan);
}

#[test]
fn for_await_identifier_assignment_head_writes_the_existing_binding() {
    let program = lower_script(
        "async function collect() {
                 let outer = 0;
                 for await (outer of [7]) {}
                 return outer;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "collect")
        .expect("async function should be registered");
    let outer_storage = async_loop_rows(&function.body.statements)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, .. } if name == "outer" => Some(name),
            _ => None,
        })
        .expect("original outer declaration");
    let plan = complete_async_for_of_plan(function);
    assert_eager_for_await_protocol(function, plan);
    assert_ne!(&plan.incoming_binding().name, outer_storage);
    assert!(matches!(
                plan.initialization().block().statements.first(),
                Some(StatementIr::DeclarationEvaluation(TypedExpr {
                    expr: ExprIr::AssignIdentifier { name, value },
                    ..
                })) if name == outer_storage
                    && matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    ));
    assert!(async_loop_rows(&function.body.statements).into_iter().any(|statement|
        matches!(statement, StatementIr::Return(value) if matches!(&value.expr, ExprIr::Identifier(name) if name == outer_storage))));
}

#[test]
fn for_await_identifier_write_alone_captures_the_outer_binding() {
    let program = lower_script(
        "let async;
             async function collect() {
                 for await (async of [7]) {}
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "collect")
        .expect("async function should be registered");
    let capture = function
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "async")
        .expect("the assignment head alone must capture its outer target");
    assert_eq!(capture.mode, BindingMode::Let);
    let plan = complete_async_for_of_plan(function);
    assert_eager_for_await_protocol(function, plan);
    assert_ne!(plan.incoming_binding().name, capture.name);
    assert!(matches!(
                plan.initialization().block().statements.first(),
                Some(StatementIr::DeclarationEvaluation(TypedExpr {
                    expr: ExprIr::AssignIdentifier { name, value },
                    ..
                })) if name == &capture.name
                    && matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    ));
}

#[test]
fn for_await_identifier_heads_keep_assignment_and_declaration_failures_distinct() {
    let program = lower_script(
        "async function assignImmutable() {
                 const immutable = 0;
                 for await (immutable of [7]) {}
             }
             async function shadow() {
                 let outer = 0;
                 for await (let outer of [7]) {}
                 return outer;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let immutable_owner = script
        .functions
        .iter()
        .find(|function| function.name == "assignImmutable")
        .expect("immutable assignment owner should be registered");
    let plan = complete_async_for_of_plan(immutable_owner);
    assert_eager_for_await_protocol(immutable_owner, plan);
    assert!(matches!(
                plan.initialization().block().statements.first(),
                Some(StatementIr::DeclarationEvaluation(TypedExpr {
                    expr: ExprIr::Comma { lhs, rhs },
                    ..
                })) if matches!(&lhs.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
                    && matches!(
                        &rhs.expr,
                        ExprIr::RuntimeThrow {
                            name: NativeErrorKind::TypeError,
                            message: "assignment to immutable binding",
                        }
                    )
    ));

    let shadow_owner = script
        .functions
        .iter()
        .find(|function| function.name == "shadow")
        .expect("declaration owner should be registered");
    let outer_storage = async_loop_rows(&shadow_owner.body.statements)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, .. } if name == "outer" => Some(name),
            _ => None,
        })
        .expect("outer declaration should remain explicit");
    let plan = complete_async_for_of_plan(shadow_owner);
    assert_eager_for_await_protocol(shadow_owner, plan);
    let [StatementIr::Lexical {
        mode: BindingMode::Let,
        name,
        init,
    }] = plan.initialization().block().statements.as_slice()
    else {
        panic!("let declaration retains its actual lexical initialization");
    };
    assert!(name.starts_with("$forof.lex."));
    assert_ne!(name, outer_storage);
    assert!(
        matches!(&init.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    );
    assert!(matches!(
        plan.body().block().statements.as_slice(),
        [StatementIr::Block(BlockIr { statements, .. })] if statements.is_empty()
    ));
}

#[test]
fn records_for_await_sync_iterator_resume_boundaries_and_owned_state() {
    let program = lower_script(
        "async function collect(iterable) {
                 for await (const value of iterable) return value;
             }
             let iterable = {};
             iterable[Symbol.iterator] = function () { return this; };
             collect(iterable);",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| {
            function.name == "collect"
                && function.protocol.execution_kind() == FunctionExecutionKind::Async
        })
        .expect("async function should be registered");
    let plan = complete_async_for_of_plan(function);
    assert_eager_for_await_protocol(function, plan);
    assert!(async_loop_rows(&plan.body().block().statements)
        .into_iter()
        .any(|statement| matches!(statement, StatementIr::Return(_))));
}

#[test]
fn records_for_await_async_iterator_mode_as_owned_state() {
    let program = lower_script(
        "async function collect(iterable) {
                 for await (const value of iterable) return value;
             }
             let iterable = {};
             iterable[Symbol.asyncIterator] = function () { return this; };
             collect(iterable);",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| {
            function.name == "collect"
                && function.protocol.execution_kind() == FunctionExecutionKind::Async
        })
        .expect("async function should be registered");
    let plan = complete_async_for_of_plan(function);
    assert_eager_for_await_protocol(function, plan);
}

#[test]
fn for_await_iterator_method_retains_strict_this_after_observable_installation() {
    let program = lower_script(
        "String.prototype[Symbol.asyncIterator] = function strictAsyncIterator() {
                 'use strict';
                 return this;
             };
             String.prototype['Symbol.asyncIterator'] = function ordinaryStringProperty() {
                 return this;
             };
             async function collect() {
                 for await (const value of 'source') return value;
             }
             collect();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "strictAsyncIterator")
        .expect("strict async iterator method should be registered");
    let return_value = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(return_value) => Some(return_value),
            _ => None,
        })
        .expect("strict async iterator method should return this");

    assert!(matches!(return_value.expr, ExprIr::This));
    assert!(function.strict);
    assert!(return_value.possible_kinds.contains(ValueKind::String));
    let collect = program
        .script
        .as_ref()
        .expect("script IR")
        .functions
        .iter()
        .find(|function| function.name == "collect")
        .expect("async collection function");
    let plan = complete_async_for_of_plan(collect);
    assert_eager_for_await_protocol(collect, plan);
}
