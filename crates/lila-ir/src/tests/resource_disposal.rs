fn checked_resource_scope(
    statements: &[StatementIr],
    execution: ResumableRegionProtocolIr,
) -> &AsyncGeneratorResourceScopeIr {
    let [StatementIr::AsyncGeneratorResourceScope(scope)] = statements else {
        panic!("one complete source-owned resource scope");
    };
    assert_eq!(scope.execution(), execution);
    scope
}

fn checked_resource_scope_suffix(scope: &AsyncGeneratorResourceScopeIr) -> &[StatementIr] {
    let [StatementIr::EmptyStatementCompletion(declaration), suffix @ ..] =
        scope.body().block().statements.as_slice()
    else {
        panic!("registration precedes the live resource suffix");
    };
    let StatementIr::LexicalBlock(initialization) = declaration.statement() else {
        panic!("the source declaration owns staged acquisition and registration");
    };
    let Some(StatementIr::AsyncGeneratorResourceRegistration(registration)) = initialization.last()
    else {
        panic!("the declaration registers its retained initializer");
    };
    assert_eq!(scope.capacity(), 1);
    assert_eq!(registration.capability_binding(), scope.capability_binding());
    suffix
}

fn checked_resource_suspension<'a>(
    statement: &'a StatementIr,
    owner: &FunctionIr,
) -> &'a StatementIr {
    let StatementIr::LexicalBlock(statements) = statement else {
        panic!("the suspension expression retains its resumed result");
    };
    let [StatementIr::Lexical { name, init, .. }, suspension, StatementIr::Expression(value)] =
        statements.as_slice()
    else {
        panic!("result storage precedes suspension and its resumed read");
    };
    assert_eq!(init.kind, ValueKind::Undefined);
    let resumed_name = match suspension {
        StatementIr::GeneratorYield {
            resume_mode: GeneratorResumeModeIr::AssignIdentifier(name), ..
        }
        | StatementIr::AsyncAwait {
            resume_mode: AsyncResumeModeIr::AssignIdentifier(name), ..
        } => name,
        _ => panic!("the suspension writes its retained result"),
    };
    assert_eq!(resumed_name, name);
    assert!(matches!(&value.expr, ExprIr::Identifier(read) if read == name));
    assert_eq!(owner.owned_env_bindings.iter().filter(|binding| &binding.name == name).count(), 1);
    suspension
}

#[test]
fn synchronous_using_declarators_are_one_non_empty_scope_resource_list() {
    let program =
        lower_script("function owner() { using first = null, second = undefined; return 1; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::SyncDisposableScope {
        execution,
        resources,
        body,
    }] = owner.body.statements.as_slice()
    else {
        panic!("using declaration must own the function-body suffix");
    };

    assert!(matches!(
        execution,
        SyncDisposableScopeExecutionIr::Immediate
    ));
    assert_eq!(resources.len(), 2);
    assert!(!resources.is_empty());
    assert_eq!(
        resources
            .iter()
            .map(|resource| resource.binding_name.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert!(matches!(
        resources
            .iter()
            .next()
            .map(|resource| &resource.initializer.expr),
        Some(ExprIr::Null)
    ));
    assert_eq!(
        resources
            .iter()
            .nth(1)
            .map(|resource| resource.initializer.kind),
        Some(ValueKind::Undefined)
    );
    assert!(matches!(
        body.statements.as_slice(),
        [StatementIr::Return(_)]
    ));
    assert!(owner
        .body
        .statements
        .iter()
        .all(|statement| !matches!(statement, StatementIr::TryFinally { .. })));
}

#[test]
fn synchronous_using_nests_only_the_reached_statement_list_suffix() {
    let program = lower_script(
        "function owner() { before(); using a = null; middle(); using b = null; after(); }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::Expression(_), StatementIr::SyncDisposableScope {
        execution: outer_execution,
        resources: outer,
        body: outer_body,
    }] = owner.body.statements.as_slice()
    else {
        panic!("statements before using must remain outside its scope");
    };
    assert!(matches!(
        outer_execution,
        SyncDisposableScopeExecutionIr::Immediate
    ));
    assert_eq!(
        outer
            .iter()
            .next()
            .map(|resource| resource.binding_name.as_str()),
        Some("a")
    );
    let [StatementIr::Expression(_), StatementIr::SyncDisposableScope {
        execution: inner_execution,
        resources: inner,
        body: inner_body,
    }] = outer_body.statements.as_slice()
    else {
        panic!("a later using declaration must own only its remaining suffix");
    };
    assert!(matches!(
        inner_execution,
        SyncDisposableScopeExecutionIr::Immediate
    ));
    assert_eq!(
        inner
            .iter()
            .next()
            .map(|resource| resource.binding_name.as_str()),
        Some("b")
    );
    assert!(matches!(
        inner_body.statements.as_slice(),
        [StatementIr::Expression(_)]
    ));
}

#[test]
fn plain_generator_synchronous_using_scope_owns_activation_capability() {
    let program = lower_script(
        "function * owner() { using outer = null; yield 1; { using inner = undefined; } }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("plain generator should be lowered");
    let outer = checked_resource_scope(&owner.body.statements, ResumableRegionProtocolIr::Generator);
    let AsyncGeneratorResourceCapabilityIr::Sync(outer_capability) = outer.capability() else {
        panic!("synchronous generator disposal capability");
    };
    let [StatementIr::GeneratorYield { .. }, StatementIr::Block(inner_block)] =
        checked_resource_scope_suffix(outer)
    else {
        panic!("yield must remain inside the live outer resource scope");
    };
    let [StatementIr::SyncDisposableScope {
        execution: SyncDisposableScopeExecutionIr::PlainGenerator(inner_capability),
        ..
    }] = inner_block.statements.as_slice()
    else {
        panic!("nested generator using must own a distinct capability");
    };

    assert_ne!(
        outer_capability.binding_name(),
        inner_capability.binding_name()
    );
    for name in [outer_capability.binding_name(), inner_capability.binding_name()] {
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == name)
                .count(),
            1,
            "plain-generator capability must have exactly one activation slot"
        );
    }
    assert!(owner.generator_plan.is_some());
}

#[test]
fn plain_async_function_synchronous_using_scope_owns_activation_capability() {
    let program = lower_script(
        "async function owner() {
                 using outer = null;
                 await 1;
                 { using inner = undefined; await 2; }
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("plain async function should be lowered");
    let outer = checked_resource_scope(&owner.body.statements, ResumableRegionProtocolIr::Async);
    let AsyncGeneratorResourceCapabilityIr::Sync(outer_capability) = outer.capability() else {
        panic!("synchronous async-function disposal capability");
    };
    let [StatementIr::AsyncAwait { .. }, StatementIr::Block(inner_block)] =
        checked_resource_scope_suffix(outer)
    else {
        panic!("await and nested block must remain inside the live outer resource scope");
    };
    let inner = checked_resource_scope(&inner_block.statements, ResumableRegionProtocolIr::Async);
    let AsyncGeneratorResourceCapabilityIr::Sync(inner_capability) = inner.capability() else {
        panic!("nested synchronous disposal capability");
    };
    assert!(matches!(
        checked_resource_scope_suffix(inner),
        [StatementIr::AsyncAwait { .. }]
    ));

    assert_ne!(
        outer_capability.binding_name(),
        inner_capability.binding_name()
    );
    for capability in [outer_capability, inner_capability] {
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == capability.binding_name())
                .count(),
            1,
            "async-function capability must have exactly one activation slot"
        );
    }
    assert_eq!(
        owner.protocol.execution_kind(),
        FunctionExecutionKind::Async
    );
    assert!(owner.resumable_plan.is_none());

    let ordinary = lower_script("function immediate() { using value = null; }");
    let immediate = ordinary
        .script
        .as_ref()
        .expect("ordinary script IR should exist")
        .functions
        .iter()
        .find(|function| function.name == "immediate")
        .expect("ordinary function should be lowered");
    assert!(matches!(
        immediate.body.statements.as_slice(),
        [StatementIr::SyncDisposableScope {
            execution: SyncDisposableScopeExecutionIr::Immediate,
            ..
        }]
    ));

    let generator = lower_script("function * generator() { using value = null; yield 1; }");
    let generator = generator
        .script
        .as_ref()
        .expect("generator script IR should exist")
        .functions
        .iter()
        .find(|function| function.name == "generator")
        .expect("plain generator should be lowered");
    let scope = checked_resource_scope(
        &generator.body.statements,
        ResumableRegionProtocolIr::Generator,
    );
    assert!(matches!(scope.capability(), AsyncGeneratorResourceCapabilityIr::Sync(_)));
}

#[test]
fn plain_async_function_await_using_owns_closed_finalizer_states() {
    let program = lower_script(
        "async function owner() {
                 await using outer = null;
                 await 1;
                 { await using inner = undefined; await 2; }
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("plain async function should be lowered");
    let outer = checked_resource_scope(&owner.body.statements, ResumableRegionProtocolIr::Async);
    let AsyncGeneratorResourceCapabilityIr::Async(outer_capability) = outer.capability() else {
        panic!("await using owns its async finalizer");
    };
    let [StatementIr::AsyncAwait { .. }, StatementIr::Block(inner_block)] =
        checked_resource_scope_suffix(outer)
    else {
        panic!("source Await and nested block must remain inside the outer scope");
    };
    let inner = checked_resource_scope(&inner_block.statements, ResumableRegionProtocolIr::Async);
    let AsyncGeneratorResourceCapabilityIr::Async(inner_capability) = inner.capability() else {
        panic!("nested await using owns its async finalizer");
    };
    assert!(matches!(
        checked_resource_scope_suffix(inner),
        [StatementIr::AsyncAwait { .. }]
    ));

    assert_eq!(outer.capacity(), 1);
    assert_eq!(inner.capacity(), 1);
    assert_ne!(
        outer_capability.binding_name(),
        inner_capability.binding_name()
    );
    for capability in [outer_capability, inner_capability] {
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == capability.binding_name())
                .count(),
            1,
            "each async-dispose capability must own one activation slot"
        );
        let finalizer = capability.finalizer();
        assert!(finalizer.entry_state() < finalizer.dispose_state());
        assert!(finalizer.dispose_state() < finalizer.resume_state());
        assert!(finalizer.resume_state() < finalizer.exit_state());
    }
    assert!(
        inner_capability.finalizer().exit_state() < outer_capability.finalizer().dispose_state(),
        "the nested finalizer must complete before the outer finalizer starts"
    );
    assert_eq!(
        owner.protocol.execution_kind(),
        FunctionExecutionKind::Async
    );

    let staged = lower_script(
        "async function owner() {
                 await using resource = await Promise.resolve(null);
             }",
    );
    assert!(staged.is_wasm_supported(), "{:?}", staged.diagnostics);
    let owner = staged.script.as_ref().unwrap().functions.iter()
        .find(|function| function.name == "owner").unwrap();
    let scope = checked_resource_scope(&owner.body.statements, ResumableRegionProtocolIr::Async);
    assert_eq!(scope.body().end_state(), scope.entry_state() + 1);
    assert!(checked_resource_scope_suffix(scope).is_empty());
}

#[test]
fn async_generator_await_using_owns_distinct_closed_finalizer_states() {
    let program = lower_script(
        "async function * owner() {
                 await using outer = null;
                 yield 1;
                 await 2;
                 { await using inner = undefined; yield 3; await 4; }
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("async generator should be lowered");
    let outer = checked_resource_scope(
        &owner.body.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    let AsyncGeneratorResourceCapabilityIr::Async(outer_capability) = outer.capability() else {
        panic!("async-generator await using owns its async finalizer");
    };
    let [yield_item, await_item, StatementIr::Block(inner_block)] =
        checked_resource_scope_suffix(outer)
    else {
        panic!("yield, await, and nested block must remain inside the live outer scope");
    };
    assert!(matches!(checked_resource_suspension(yield_item, owner), StatementIr::GeneratorYield { .. }));
    assert!(matches!(checked_resource_suspension(await_item, owner), StatementIr::AsyncAwait { .. }));
    let inner = checked_resource_scope(
        &inner_block.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    let AsyncGeneratorResourceCapabilityIr::Async(inner_capability) = inner.capability() else {
        panic!("nested await using owns its async finalizer");
    };
    let [yield_item, await_item] = checked_resource_scope_suffix(inner) else {
        panic!("both suspension expressions remain inside the inner scope");
    };
    assert!(matches!(checked_resource_suspension(yield_item, owner), StatementIr::GeneratorYield { .. }));
    assert!(matches!(checked_resource_suspension(await_item, owner), StatementIr::AsyncAwait { .. }));

    assert_eq!(outer.capacity(), 1);
    assert_eq!(inner.capacity(), 1);
    assert_ne!(
        outer_capability.binding_name(),
        inner_capability.binding_name()
    );
    for capability in [outer_capability, inner_capability] {
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == capability.binding_name())
                .count(),
            1,
            "each async-generator async-dispose capability must own one activation slot"
        );
        let finalizer = capability.finalizer();
        assert!(finalizer.entry_state() < finalizer.dispose_state());
        assert!(finalizer.dispose_state() < finalizer.resume_state());
        assert!(finalizer.resume_state() < finalizer.exit_state());
    }
    assert!(
        inner_capability.finalizer().exit_state() < outer_capability.finalizer().dispose_state(),
        "the nested finalizer must complete before the outer finalizer starts"
    );
    assert_eq!(
        owner.protocol.execution_kind(),
        FunctionExecutionKind::AsyncGenerator
    );
    assert!(owner.resumable_plan.is_some());

    let staged = lower_script(
        "async function * owner() {
                 await using resource = await Promise.resolve(null);
             }",
    );
    assert!(staged.is_wasm_supported(), "{:?}", staged.diagnostics);
    let owner = staged.script.as_ref().unwrap().functions.iter()
        .find(|function| function.name == "owner").unwrap();
    let scope = checked_resource_scope(
        &owner.body.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    assert_eq!(scope.body().end_state(), scope.entry_state() + 1);
    assert!(checked_resource_scope_suffix(scope).is_empty());
}

#[test]
fn async_generator_await_using_reserves_nested_finalizer_before_following_yield() {
    let program = lower_script(
        "async function * owner() {
                 await using outer = null;
                 yield 0;
                 { await using inner = undefined; }
                 yield 1;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let owner = program
        .script
        .as_ref()
        .expect("script IR should exist")
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("async generator should be lowered");
    let outer_scope = checked_resource_scope(
        &owner.body.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    let AsyncGeneratorResourceCapabilityIr::Async(outer) = outer_scope.capability() else {
        panic!("outer await using owns its async finalizer");
    };
    let [before, StatementIr::Block(inner_block), after] = checked_resource_scope_suffix(outer_scope)
    else {
        panic!("the nested resource block remains between its surrounding yield expressions");
    };
    assert!(matches!(checked_resource_suspension(before, owner), StatementIr::GeneratorYield {
        suspend_state: 0,
        resume_state: 1,
        ..
    }));
    assert!(matches!(checked_resource_suspension(after, owner), StatementIr::GeneratorYield {
        suspend_state: 4,
        resume_state: 5,
        ..
    }), "the following yield must resume after the nested finalizer states");
    let inner_scope = checked_resource_scope(
        &inner_block.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    let AsyncGeneratorResourceCapabilityIr::Async(inner) = inner_scope.capability() else {
        panic!("nested block retains its async-dispose capability");
    };
    assert!(checked_resource_scope_suffix(inner_scope).is_empty());

    assert_eq!(inner.finalizer().entry_state(), 1);
    assert_eq!(inner.finalizer().dispose_state(), 2);
    assert_eq!(inner.finalizer().resume_state(), 3);
    assert_eq!(inner.finalizer().exit_state(), 4);
    assert_eq!(outer.finalizer().entry_state(), 0);
    assert_eq!(outer.finalizer().dispose_state(), 6);
    assert_eq!(outer.finalizer().resume_state(), 7);
    assert_eq!(outer.finalizer().exit_state(), 8);

    let plan = owner
        .resumable_plan
        .as_ref()
        .expect("async generator must retain its unified resumable plan");
    assert_eq!(plan.state_count, 9);
    assert_eq!(
        plan.suspension_points,
        vec![
            ResumableSuspensionPointIr {
                kind: ResumableSuspensionKindIr::Yield,
                suspend_state: 0,
                resume_state: 1,
                resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
            },
            ResumableSuspensionPointIr {
                kind: ResumableSuspensionKindIr::Yield,
                suspend_state: 4,
                resume_state: 5,
                resume_environment: ResumableResumeEnvironmentIr::InvocationOuter,
            },
        ]
    );
    assert!(plan.resume_environment_plan().enclosing_scope_resume_states()
        .contains(&inner.finalizer().resume_state()));
}

#[test]
fn await_using_initializer_capture_hops_include_materialized_tdz_environment() {
    let program = lower_script(
        "async function probe(trace, invalid) {
                 try {
                     await using registered = {
                         get [Symbol.asyncDispose]() {
                             try { registered; } catch (error) {}
                             return async () => trace.push('dispose');
                         }
                     };
                     await using rejected = invalid;
                 } catch (error) {}
             }
             async function sibling(trace) {
                 await 0;
                 return trace;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "probe")
        .expect("plain async owner should be lowered");
    let getter = script
        .functions
        .iter()
        .find(|function| function.protocol == FunctionProtocolIr::ObjectGetter)
        .expect("async disposer getter should be lowered");
    let disposer = script
        .functions
        .iter()
        .find(|function| function.protocol == FunctionProtocolIr::AsyncArrow)
        .expect("nested async disposer should be lowered");
    let registered = getter
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "registered")
        .expect("getter should capture the uninitialized resource binding");
    let trace = disposer
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "trace")
        .expect("nested disposer should capture the owner parameter");

    assert!(getter.owned_env_bindings.is_empty(), "{getter:#?}");
    assert_eq!(registered.hops, 0);
    assert!(disposer.owned_env_bindings.is_empty(), "{disposer:#?}");
    // The resumable arrow retains its own empty activation frame, then
    // crosses the try-block TDZ environment to reach the owner parameter.
    assert_eq!(trace.hops, 2);
    assert_eq!(registered.slot, trace.slot);
    assert!(owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == "trace" && binding.slot == trace.slot));
    assert!(
        !owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == registered.name),
        "the captured resource cell belongs only to the try-block environment"
    );
    let [StatementIr::TryCatch { try_block, .. }] = owner.body.statements.as_slice() else {
        panic!("owner should retain its try statement");
    };
    assert!(try_block
        .lexical_environment
        .as_ref()
        .is_some_and(|environment| {
            environment
                .bindings
                .iter()
                .any(|binding| binding.name == registered.name && binding.slot == registered.slot)
        }));
}

#[test]
fn async_generator_synchronous_using_scope_owns_activation_capability() {
    let program = lower_script(
        "async function * owner() {
                 using outer = null;
                 yield 1;
                 await 2;
                 { using inner = undefined; yield 3; await 4; }
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("async generator should be lowered");
    let outer = checked_resource_scope(
        &owner.body.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    let AsyncGeneratorResourceCapabilityIr::Sync(outer_capability) = outer.capability() else {
        panic!("async-generator using owns its synchronous capability");
    };
    let [yield_item, await_item, StatementIr::Block(inner_block)] =
        checked_resource_scope_suffix(outer)
    else {
        panic!("yield, await, and nested block must remain inside the live outer scope");
    };
    assert!(matches!(checked_resource_suspension(yield_item, owner), StatementIr::GeneratorYield { .. }));
    assert!(matches!(checked_resource_suspension(await_item, owner), StatementIr::AsyncAwait { .. }));
    let inner = checked_resource_scope(
        &inner_block.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    let AsyncGeneratorResourceCapabilityIr::Sync(inner_capability) = inner.capability() else {
        panic!("nested async-generator using owns its synchronous capability");
    };
    let [yield_item, await_item] = checked_resource_scope_suffix(inner) else {
        panic!("both suspension expressions remain inside the inner scope");
    };
    assert!(matches!(checked_resource_suspension(yield_item, owner), StatementIr::GeneratorYield { .. }));
    assert!(matches!(checked_resource_suspension(await_item, owner), StatementIr::AsyncAwait { .. }));

    assert_ne!(
        outer_capability.binding_name(),
        inner_capability.binding_name()
    );
    for capability in [outer_capability, inner_capability] {
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == capability.binding_name())
                .count(),
            1,
            "async-generator capability must have exactly one activation slot"
        );
    }
    assert_eq!(
        owner.protocol.execution_kind(),
        FunctionExecutionKind::AsyncGenerator
    );
    assert!(owner.resumable_plan.is_some());

    let staged =
        lower_script("async function * owner() { using resource = await Promise.resolve(null); }");
    assert!(staged.is_wasm_supported(), "{:?}", staged.diagnostics);
    let owner = staged.script.as_ref().unwrap().functions.iter()
        .find(|function| function.name == "owner").unwrap();
    let scope = checked_resource_scope(
        &owner.body.statements, ResumableRegionProtocolIr::AsyncGenerator,
    );
    assert_eq!(scope.body().end_state(), scope.entry_state() + 1);
    assert!(checked_resource_scope_suffix(scope).is_empty());
}

#[test]
fn synchronous_using_entry_is_the_only_runtime_binding_initializer() {
    fn contains_lexical(statement: &StatementIr, name: &str) -> bool {
        match statement {
            StatementIr::Lexical {
                name: binding_name, ..
            } => binding_name == name,
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => statements
                .iter()
                .any(|statement| contains_lexical(statement, name)),
            StatementIr::SyncDisposableScope { body, .. }
            | StatementIr::AsyncDisposableScope { body, .. }
            | StatementIr::Block(body) => body
                .statements
                .iter()
                .any(|statement| contains_lexical(statement, name)),
            _ => false,
        }
    }

    let program = lower_script("function owner() { { using resource = null; resource; } }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let StatementIr::Block(block) = &owner.body.statements[0] else {
        panic!("inner source block should remain explicit");
    };
    let StatementIr::SyncDisposableScope { resources, .. } = &block.statements[0] else {
        panic!("inner using should lower to a dedicated scope");
    };
    let binding_name = &resources
        .iter()
        .next()
        .expect("resource list is statically non-empty")
        .binding_name;
    assert!(!contains_lexical(&owner.body.statements[0], binding_name));
}

#[test]
fn synchronous_using_classic_for_owns_one_non_empty_initializer_capability() {
    let program = lower_script(
        "function owner() { for (using first = null, second = undefined; false;) {} }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::For {
        init: Some(ForInitIr::SyncDisposable(resources)),
        ..
    }] = owner.body.statements.as_slice()
    else {
        panic!("classic using head must remain a direct For with a disposable initializer");
    };

    assert_eq!(resources.len(), 2);
    assert!(!resources.is_empty());
    let names = resources
        .iter()
        .map(|resource| resource.binding_name.as_str())
        .collect::<Vec<_>>();
    assert!(names[0].ends_with("first"), "{names:?}");
    assert!(names[1].ends_with("second"), "{names:?}");
    assert!(matches!(
        resources
            .iter()
            .next()
            .map(|resource| &resource.initializer.expr),
        Some(ExprIr::Null)
    ));
}

#[test]
fn synchronous_using_classic_for_remains_the_direct_label_target() {
    let program = lower_script(
        "function owner() { outer: for (using resource = null; false;) { continue outer; } }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::Labelled {
        labels, statement, ..
    }] = owner.body.statements.as_slice()
    else {
        panic!("source label should remain explicit");
    };
    assert_eq!(
        labels.iter().map(String::as_str).collect::<Vec<_>>(),
        ["outer"]
    );
    assert!(matches!(
        statement.as_ref(),
        StatementIr::For {
            init: Some(ForInitIr::SyncDisposable(_)),
            ..
        }
    ));
}

#[test]
fn synchronous_using_classic_for_retains_captured_head_environment() {
    let program = lower_script(
        "function owner() { for (using resource = null; false;) { (() => resource); } }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::For {
        init: Some(ForInitIr::SyncDisposable(resources)),
        lexical_environment: Some(environment),
        ..
    }] = owner.body.statements.as_slice()
    else {
        panic!("captured using head must retain its For lexical environment");
    };
    let resource_name = &resources
        .iter()
        .next()
        .expect("resource list is statically non-empty")
        .binding_name;
    assert!(environment
        .bindings
        .iter()
        .any(|binding| &binding.name == resource_name));
    assert!(environment.per_iteration_slots.is_empty());
}

#[test]
fn synchronous_using_classic_for_initializer_observes_its_own_tdz() {
    let program = lower_script("function owner() { for (using resource = resource; false;) {} }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::For {
        init: Some(ForInitIr::SyncDisposable(resources)),
        ..
    }] = owner.body.statements.as_slice()
    else {
        panic!("classic using head must own its resource initializer");
    };
    assert!(matches!(
        resources
            .iter()
            .next()
            .map(|resource| &resource.initializer.expr),
        Some(ExprIr::RuntimeThrow {
            name: NativeErrorKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn plain_async_classic_for_await_using_owns_closed_initializer_capability() {
    let program = lower_script(
            "async function owner() { outer: for (await using first = null, second = undefined; false; first) { (() => first); continue outer; } }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let [StatementIr::Labelled {
        labels, statement, ..
    }] = owner.body.statements.as_slice()
    else {
        panic!("source label should remain explicit");
    };
    assert_eq!(
        labels.iter().map(String::as_str).collect::<Vec<_>>(),
        ["outer"]
    );
    let StatementIr::AsyncGeneratorLoop(plan) = statement.as_ref() else {
        panic!("await using head belongs to the complete classic For");
    };
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    let init = plan.resource().expect("one initializer disposal capability");
    let environment = plan.lexical_environment().expect("captured head environment");
    let update = plan.update().expect("source update");
    assert_eq!(update.value().kind, ValueKind::Undefined, "For discards the update value");
    let [StatementIr::Expression(update)] = update.region().block().statements.as_slice() else {
        panic!("the update still evaluates its original binding read");
    };

    assert_eq!(init.capacity(), 2);
    let registrations = plan.initialization().unwrap().block().statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncGeneratorResourceRegistration(registration) => Some(registration),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(registrations.len(), 2);
    assert!(registrations.iter().all(|registration|
        registration.capability_binding() == init.capability_binding()
            && registration.hint() == ResourceDisposalHintIr::Async));
    let names = registrations.iter().map(|registration| registration.binding_name())
        .collect::<Vec<_>>();
    assert!(names[0].ends_with("first"), "{names:?}");
    assert!(names[1].ends_with("second"), "{names:?}");
    assert!(environment
        .bindings
        .iter()
        .any(|binding| binding.name == names[0]));
    assert!(environment.per_iteration_slots.is_empty());
    assert!(matches!(&update.expr, ExprIr::Identifier(name) if name == names[0]));
    assert_eq!(
        update.kind,
        ValueKind::Null,
        "resource metadata must leave TDZ before test/update/body lowering"
    );

    let AsyncGeneratorResourceCapabilityIr::Async(capability) = init.capability() else {
        panic!("await using head owns its async finalizer");
    };
    assert_eq!(
        owner
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == capability.binding_name())
            .count(),
        1,
        "classic-for capability must own exactly one activation slot"
    );
    let finalizer = capability.finalizer();
    assert!(finalizer.entry_state() < finalizer.dispose_state());
    assert!(finalizer.dispose_state() < finalizer.resume_state());
    assert!(finalizer.resume_state() < finalizer.exit_state());

    let suspended = lower_script(
        "async function owner() { for (await using resource = null; false;) { await 0; } }",
    );
    assert!(suspended.is_wasm_supported(), "{:?}", suspended.diagnostics);
    let owner = suspended.script.as_ref().unwrap().functions.iter()
        .find(|function| function.name == "owner").unwrap();
    let [StatementIr::AsyncGeneratorLoop(plan)] = owner.body.statements.as_slice() else {
        panic!("one complete suspended classic For");
    };
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    assert_eq!(plan.body().end_state(), plan.body().entry_state() + 1);
    let AsyncGeneratorResourceCapabilityIr::Async(capability) =
        plan.resource().unwrap().capability() else { panic!("async disposal"); };
    assert!(capability.finalizer().dispose_state() > plan.body().end_state());
}

#[test]
fn plain_async_for_of_await_using_owns_repeating_iteration_capability() {
    let program = lower_script(
        "async function owner() {
                 for (await using resource of [resource]) {
                     (() => resource);
                     resource = null;
                 }
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("plain async owner should be lowered");
    let [StatementIr::AsyncGeneratorForOf(plan)] = owner.body.statements.as_slice() else {
        panic!("await using owns one complete iterator lifetime");
    };
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    let resource = plan.resource().expect("per-iteration disposal capability");
    let [StatementIr::AsyncGeneratorResourceRegistration(head)] =
        plan.initialization().block().statements.as_slice()
    else { panic!("original incoming value is registered once"); };
    assert_eq!(head.capability_binding(), resource.capability_binding());
    let environment = plan.lexical_environment().expect("original per-key environment");
    let [StatementIr::Lexical { init: iterable, .. }] =
        plan.head().region().block().statements.as_slice()
    else { panic!("one complete iterable evaluation"); };
    let [body] = plan.body().block().statements.as_slice() else {
        panic!("original iterator body");
    };

    assert!(head.binding_name().starts_with("$forof.lex."));
    assert!(head.binding_name().ends_with(".resource"));
    assert!(environment
        .tdz_binding_names
        .iter()
        .any(|name| name == "$tdz.resource"));
    assert!(matches!(
        &iterable.expr,
        ExprIr::ArrayLiteral(elements)
            if matches!(
                elements.as_slice(),
                [TypedExpr {
                    expr: ExprIr::RuntimeThrow {
                        name: NativeErrorKind::ReferenceError,
                        ..
                    },
                    ..
                }]
            )
    ));

    let iteration_binding = environment
        .iteration_environment
        .as_ref()
        .and_then(|environment| {
            environment
                .bindings
                .iter()
                .find(|binding| binding.name == head.binding_name())
        })
        .expect("captured resource must own fresh iteration storage");
    let capture = script
        .functions
        .iter()
        .flat_map(|function| &function.captured_bindings)
        .find(|binding| binding.source_name == "resource")
        .expect("body arrow must capture the resource iteration binding");
    assert_eq!(capture.name, iteration_binding.name);
    assert_eq!(capture.slot, iteration_binding.slot);
    assert_eq!(capture.mode, BindingMode::Const);
    assert!(matches!(
        body,
        StatementIr::Block(BlockIr { statements, .. })
            if statements.iter().any(|statement| matches!(
                statement,
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::Comma {
                        lhs,
                        rhs,
                    },
                    ..
                }) if matches!(lhs.expr, ExprIr::Null)
                    && matches!(
                        rhs.expr,
                        ExprIr::RuntimeThrow {
                            name: NativeErrorKind::TypeError,
                            ref message,
                        } if *message == "assignment to immutable binding"
                    )
            ))
    ));

    let AsyncGeneratorResourceCapabilityIr::Async(capability) = resource.capability() else {
        panic!("await using iteration owns its async finalizer");
    };
    let owned_names = [
        capability.binding_name(),
        plan.head_binding().name.as_str(),
        plan.incoming_binding().name.as_str(),
        plan.value_binding().name.as_str(),
    ];
    assert_eq!(
        owned_names.iter().copied().collect::<BTreeSet<_>>().len(),
        4
    );
    for name in owned_names {
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == name)
                .count(),
            1,
            "every activation-backed role must own exactly one slot"
        );
    }
    let finalizer = capability.finalizer();
    assert!(finalizer.entry_state() < finalizer.dispose_state());
    assert!(finalizer.dispose_state() < finalizer.resume_state());
    assert!(finalizer.resume_state() < finalizer.exit_state());

    let suspended = lower_script(
        "async function owner(values) {
                 for (await using resource of values) { await 0; }
             }",
    );
    assert!(suspended.is_wasm_supported(), "{:?}", suspended.diagnostics);
    let owner = suspended.script.as_ref().unwrap().functions.iter()
        .find(|function| function.name == "owner").unwrap();
    let [StatementIr::AsyncGeneratorForOf(plan)] = owner.body.statements.as_slice() else {
        panic!("one complete suspended resource iterator");
    };
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    assert_eq!(plan.body().end_state(), plan.body().entry_state() + 1);
    let AsyncGeneratorResourceCapabilityIr::Async(capability) =
        plan.resource().unwrap().capability() else { panic!("async disposal"); };
    assert!(capability.finalizer().dispose_state() > plan.body().end_state());
}

#[test]
fn synchronous_using_for_of_is_a_closed_generic_iterator_head() {
    let program = lower_script("for (using resource of [null]) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let [StatementIr::ForOfIterator {
        head: ForOfIteratorHeadIr::SyncDisposable(head),
        ..
    }] = script.body.statements.as_slice()
    else {
        panic!(
            "a using head must force generic synchronous iteration: {:?}",
            script.body.statements
        );
    };
    assert!(head.binding_name().starts_with("$forof.lex."));
    assert!(head.binding_name().ends_with(".resource"));
}

#[test]
fn synchronous_using_for_of_iterable_observes_its_own_tdz() {
    let program = lower_script("let resource = null; for (using resource of [resource]) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::ForOfIterator { iterable, .. } = &script.body.statements[1] else {
        panic!("using head must lower to generic iterator IR");
    };
    let ExprIr::ArrayLiteral(elements) = &iterable.expr else {
        panic!("expected the source array iterable to stay explicit");
    };
    assert!(matches!(
        elements.as_slice(),
        [TypedExpr {
            expr: ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            },
            ..
        }]
    ));
}

#[test]
fn synchronous_using_for_of_retains_fresh_captured_iteration_storage() {
    let program = lower_script(
            "let callbacks = []; for (using resource of [null, null]) { callbacks.push(() => resource); }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::ForOfIterator {
        head: ForOfIteratorHeadIr::SyncDisposable(head),
        lexical_environment: Some(environment),
        ..
    } = &script.body.statements[1]
    else {
        panic!("captured using head must retain its iteration environment");
    };
    assert!(environment
        .tdz_binding_names
        .iter()
        .any(|name| name == "$tdz.resource"));
    let iteration_binding = environment
        .iteration_environment
        .as_ref()
        .and_then(|environment| {
            environment
                .bindings
                .iter()
                .find(|binding| binding.name == head.binding_name())
        })
        .expect("using head must own fresh captured iteration storage");
    let capture = script
        .functions
        .iter()
        .flat_map(|function| &function.captured_bindings)
        .find(|binding| binding.source_name == "resource")
        .expect("body arrow must capture the using iteration binding");
    assert_eq!(capture.name, iteration_binding.name);
    assert_eq!(capture.slot, iteration_binding.slot);
    assert_eq!(capture.mode, BindingMode::Const);
}

#[test]
fn pattern_looking_using_for_of_head_is_ordinary_element_assignment() {
    let program = lower_script("for (using [resource] of [[null]]) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let [StatementIr::ForOfIterator {
        head:
            ForOfIteratorHeadIr::Assignment {
                binding: head,
                async_plan: None,
                protocol,
            },
        body,
        ..
    }] = script.body.statements.as_slice()
    else {
        panic!(
            "using[resource] is an element-access assignment head: {:?}",
            script.body.statements
        );
    };
    assert_eq!(*protocol, IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL);
    assert_eq!(head.mode, BindingMode::Let);
    assert!(head.name.starts_with("$forof.access"));
    assert!(matches!(
        body.as_ref(),
        StatementIr::Block(BlockIr { statements, .. })
            if matches!(
                statements.first(),
                Some(StatementIr::DeclarationEvaluation(TypedExpr {
                    expr: ExprIr::PropertyWrite { .. },
                    ..
                }))
            )
    ));
}
