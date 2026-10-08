#[test]
fn plain_async_for_loop_await_lowers_to_a_resumable_loop() {
    // Without this the loop lowers to a straight-line `StatementIr::For`
    // holding the await: the async driver re-enters the body from the top,
    // so the loop restarts at iteration zero and the suspension, already
    // past its state guard, never fires again.
    let program = lower_script(
            "(async function(){ let t = 0; for (let i = 0; i < 3; i++) { t += await Promise.resolve(i); } print(t); })();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .first()
        .expect("async function expression should be collected");
    assert_eq!(
        function.protocol.execution_kind(),
        FunctionExecutionKind::Async
    );
    assert!(function.resumable_plan.is_none());

    let [StatementIr::Lexical { name, .. }, StatementIr::AsyncGeneratorLoop(plan), StatementIr::Expression(_)] = function.body.statements.as_slice()
    else {
        panic!(
            "expected a resumable await loop between the accumulator and the print: {:#?}",
            function.body.statements
        );
    };
    assert_eq!(name, "t");
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    assert_eq!(plan.kind(), GeneratorLoopKindIr::For);
    assert!(plan.initialization().is_some());
    assert!(plan.update().is_some());
    assert_eq!(plan.suspensions().len(), 1);
    let point = &plan.suspensions()[0];
    assert_eq!(point.kind, ResumableSuspensionKindIr::Await);
    assert_eq!(point.suspend_state, plan.body().entry_state());
    assert_eq!(point.resume_state, plan.body().end_state());
    assert_eq!(plan.continue_state(), plan.update().unwrap().region().entry_state());
}

#[test]
fn plain_async_while_loop_await_lowers_to_a_resumable_loop() {
    let program = lower_script(
            "(async function(){ let n = 0; while (n < 3) { n++; await Promise.resolve(0); } print(n); })();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .first()
        .expect("async function expression should be collected");
    assert!(
        function
            .body
            .statements
            .iter()
            .any(|statement| matches!(statement, StatementIr::AsyncGeneratorLoop(plan)
                if plan.execution() == ResumableRegionProtocolIr::Async
                    && plan.kind() == GeneratorLoopKindIr::While)),
        "{:#?}",
        function.body.statements
    );
}

fn async_loop_statement_rows<'a>(items: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for item in items {
        output.push(item);
        match item {
            StatementIr::Block(block) => async_loop_statement_rows(&block.statements, output),
            StatementIr::LexicalBlock(items) => async_loop_statement_rows(items, output),
            StatementIr::EmptyStatementCompletion(item) => async_loop_statement_rows(std::slice::from_ref(item.statement()), output),
            StatementIr::Labelled { statement, .. } => async_loop_statement_rows(std::slice::from_ref(statement), output),
            StatementIr::AsyncGeneratorForOf(plan) => {
                async_loop_statement_rows(&plan.head().region().block().statements, output);
                async_loop_statement_rows(&plan.initialization().block().statements, output);
                async_loop_statement_rows(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() { async_loop_statement_rows(&region.block().statements, output); }
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() { async_loop_statement_rows(&region.block().statements, output); }
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                async_loop_statement_rows(&plan.then_branch().block().statements, output);
                async_loop_statement_rows(&plan.else_branch().block().statements, output);
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                async_loop_statement_rows(&plan.condition().region().block().statements, output);
                async_loop_statement_rows(&plan.then_branch().block().statements, output);
                async_loop_statement_rows(&plan.else_branch().block().statements, output);
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                async_loop_statement_rows(&plan.head().region().block().statements, output);
                async_loop_statement_rows(&plan.initialization_region().block().statements, output);
                async_loop_statement_rows(&plan.body().block().statements, output);
            }
            StatementIr::If { then_branch, else_branch, .. }
            | StatementIr::AsyncFunctionIf { then_branch, else_branch, .. } => {
                async_loop_statement_rows(std::slice::from_ref(then_branch), output);
                if let Some(branch) = else_branch { async_loop_statement_rows(std::slice::from_ref(branch), output); }
            }
            StatementIr::TryCatch { try_block, catch_block, .. } => {
                async_loop_statement_rows(&try_block.statements, output);
                async_loop_statement_rows(&catch_block.statements, output);
            }
            StatementIr::TryFinally { try_block, finally_block, .. } => {
                async_loop_statement_rows(&try_block.statements, output);
                async_loop_statement_rows(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally { try_block, catch_block, finally_block, .. } => {
                async_loop_statement_rows(&try_block.statements, output);
                async_loop_statement_rows(&catch_block.statements, output);
                async_loop_statement_rows(&finally_block.statements, output);
            }
            _ => {}
        }
    }
}
fn async_loop_rows(items: &[StatementIr]) -> Vec<&StatementIr> {
    let mut output = Vec::new();
    async_loop_statement_rows(items, &mut output);
    output
}
fn complete_async_for_of_in(items: &[StatementIr]) -> Option<&AsyncGeneratorForOfIr> {
    async_loop_rows(items).into_iter().find_map(|statement| match statement {
        StatementIr::AsyncGeneratorForOf(plan) => Some(plan.as_ref()),
        _ => None,
    })
}
fn complete_async_for_of_plan(function:&FunctionIr)->&AsyncGeneratorForOfIr {
    complete_async_for_of_in(&function.body.statements).expect("checked complete iterator")
}
fn assert_complete_async_for_of_storage(function:&FunctionIr,plan:&AsyncGeneratorForOfIr) {
    assert_eq!(plan.execution(),ResumableRegionProtocolIr::Async);
    for binding in [plan.head_binding(),plan.incoming_binding(),plan.value_binding()] {assert!(function.owned_env_bindings.contains(binding));}
    assert_eq!([plan.head_binding(),plan.incoming_binding(),plan.value_binding()].into_iter().map(|binding|binding.slot).collect::<BTreeSet<_>>().len(),3);
    assert!(plan.initialization().end_state()<plan.body().entry_state());
    assert!(plan.suspensions().iter().any(|point|point.kind==ResumableSuspensionKindIr::Await && point.suspend_state>=plan.body().entry_state() && point.resume_state<=plan.body().end_state()));
}
fn complete_async_for_of_put(plan:&AsyncGeneratorForOfIr)->&DestructuringTargetIr {
    async_loop_rows(&plan.initialization().block().statements).into_iter().find_map(|statement|match statement {
        StatementIr::DeclarationEvaluation(TypedExpr {expr:ExprIr::ObjectDestructuringOperation(operation),..})
        | StatementIr::Expression(TypedExpr {expr:ExprIr::ObjectDestructuringOperation(operation),..})=>match operation.use_view() {
            ObjectDestructuringOperationView::PutTarget {target,value}=>{assert!(matches!(&value.expr,ExprIr::Identifier(name) if name==&plan.incoming_binding().name));Some(target)},
            ObjectDestructuringOperationView::GetV {..}|ObjectDestructuringOperationView::Rest {..}=>None,
        },_=>None,
    }).expect("original captured per-key Put")
}

#[test]
fn plain_async_for_of_body_await_owns_a_synchronous_iterator_record() {
    let program=lower_script("(async function(){ for (const x of \"ab\") { await 0; x; } })();");
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    assert_eq!(plan.head_mode(),BindingMode::Const);
    assert!(matches!(plan.protocol(),AsyncGeneratorIteratorProtocolIr::Sync));
    assert_eq!(plan.head().region().entry_state(),0);
    assert!(matches!(plan.initialization().block().statements.as_slice(),[StatementIr::Lexical {mode:BindingMode::Const,init,..}] if matches!(&init.expr,ExprIr::Identifier(name) if name==&plan.incoming_binding().name)));
}

#[test]
fn plain_async_for_of_assignment_head_runs_before_the_body_await() {
    let program=lower_script("(async function(){ let x; for (x of \"ab\") { await 0; x; } })();");
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    assert!(matches!(plan.initialization().block().statements.as_slice(),[StatementIr::DeclarationEvaluation(TypedExpr {expr:ExprIr::AssignIdentifier {value,..},..})] if matches!(&value.expr,ExprIr::Identifier(name) if name==&plan.incoming_binding().name)));
}

#[test]
fn plain_async_for_of_static_member_head_writes_before_the_body_await() {
    let program=lower_script(
        "(async function(){ const target = {}; for (target.value of [1]) { await 0; } })();",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let target=complete_async_for_of_put(plan);
    assert!(matches!(target,DestructuringTargetIr::AssignmentProperty {key:DestructuringPropertyKeyIr::Static(key),..} if key=="value"));
    assert!(plan.initialization().block().statements.iter().any(|statement|matches!(statement,StatementIr::Lexical {init:TypedExpr {expr:ExprIr::Identifier(name),..},..} if name=="target")));
}

#[test]
fn plain_async_for_of_computed_member_head_captures_its_reference() {
    let program=lower_script(
        "function owner() {
                let target = {};
                let key = \"value\";
                async function assign(iterable) {
                    for (target[key] of iterable) { await 0; }
                }
                return assign;
            }
            owner();",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let captures=function.captured_bindings.iter().map(|binding|binding.source_name.as_str()).collect::<BTreeSet<_>>();
    assert!(captures.contains("target")&&captures.contains("key"));
    assert!(matches!(complete_async_for_of_put(plan),DestructuringTargetIr::AssignmentProperty {key:DestructuringPropertyKeyIr::Computed(_),..}));
    for source in ["target","key"] {assert!(plan.initialization().block().statements.iter().any(|statement|matches!(statement,StatementIr::Lexical {init:TypedExpr {expr:ExprIr::Identifier(name),..},..} if name==source)));}
}

#[test]
fn plain_async_for_of_private_member_head_writes_before_the_body_await() {
    let program=lower_script(
        "class C {
                #value = 0;
                async assign() {
                    for (this.#value of [1]) { await 0; }
                }
            }",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    assert!(matches!(complete_async_for_of_put(plan),DestructuringTargetIr::AssignmentPrivate {private_name_id,..} if private_name_id==&function.private_name_ids["value"]));
    assert!(plan.initialization().block().statements.iter().any(|statement|matches!(statement,StatementIr::Lexical {init:TypedExpr {expr:ExprIr::This,..},..})));
}

#[test]
fn plain_async_for_of_var_pattern_bindings_survive_the_body_await() {
    let program=lower_script(
        "(async function(){
                for (var [selected = 3, ...remaining] of [[undefined, 4, 5]]) {
                    await 0;
                    selected;
                    remaining;
                }
            })();",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let [StatementIr::DeclarationEvaluation(TypedExpr {expr:ExprIr::ArrayDestructure {pattern,evaluation:ArrayDestructuringEvaluationIr::BindingInitialization,..},..})]=plan.initialization().block().statements.as_slice() else {panic!("original Array BindingInitialization");};
    let mut names=Vec::new();pattern.visit_bindings(&mut |mode,name|{assert_eq!(mode,BindingMode::Var);names.push(name.to_owned());});
    assert_eq!(names.into_iter().collect::<BTreeSet<_>>(),BTreeSet::from(["selected".to_owned(),"remaining".to_owned()]));
    for name in ["selected","remaining"] {assert!(function.owned_env_bindings.iter().any(|row|row.name==name));}
}

#[test]
fn plain_async_for_of_nested_lexical_pattern_owns_every_iteration_binding() {
    let program=lower_script(
        "async function collect(values) {
                for (const { first, nested: [second, { third }], ...rest } of values) {
                    await 0;
                }
            }",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let [StatementIr::DeclarationEvaluation(TypedExpr {expr:ExprIr::ObjectDestructure {pattern,..},..})]=plan.initialization().block().statements.as_slice() else {panic!("original Object BindingInitialization");};
    let mut names=Vec::new();pattern.visit_bindings(&mut |mode,name|{assert_eq!(mode,BindingMode::Const);names.push(name.to_owned());});
    let environment=plan.lexical_environment().expect("original lexical head").iteration_environment.as_ref().expect("original per-key record");
    assert_eq!(names.into_iter().collect::<BTreeSet<_>>(),environment.bindings.iter().map(|row|row.name.clone()).collect());
    for name in ["first","second","third","rest"] {assert!(environment.bindings.iter().any(|row|row.name.ends_with(&format!(".{name}"))));}
}

#[test]
fn plain_async_for_of_lexical_pattern_forward_default_reads_the_iteration_tdz() {
    let program=lower_script(
            "async function defaults(values) { for (let { first = second, second = 1 } of values) { await 0; } }",
        );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let [StatementIr::DeclarationEvaluation(TypedExpr {expr:ExprIr::ObjectDestructure {pattern,..},..})]=plan.initialization().block().statements.as_slice() else {panic!("original Object BindingInitialization");};
    let [first,second]=pattern.properties.as_slice() else {panic!("two declarations");};
    assert!(matches!(&first.target,DestructuringTargetIr::Binding {mode:BindingMode::Let,name} if name.ends_with(".first")));
    assert!(matches!(&second.target,DestructuringTargetIr::Binding {mode:BindingMode::Let,name} if name.ends_with(".second")));
    assert!(matches!(&first.default.as_ref().expect("forward default").expr,ExprIr::RuntimeThrow {name:NativeErrorKind::ReferenceError,..}));
}

#[test]
fn plain_async_for_of_empty_lexical_patterns_keep_semantic_initialization() {
    let program=lower_script(
        "async function emptyArray(values) {
                for (let [] of values) { await 0; }
            }
            async function emptyObject(values) {
                for (const {} of values) { await 0; }
            }",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    for function in program.script.as_ref().unwrap().functions.iter().filter(|function|["emptyArray","emptyObject"].contains(&function.name.as_str())) {
        let plan=complete_async_for_of_plan(function);assert_complete_async_for_of_storage(function,plan);
        let environment=plan.lexical_environment().expect("empty lexical head witness");
        assert!(environment.tdz_binding_names.is_empty()&&environment.tdz_environment.is_none()&&environment.iteration_environment.is_none());
        let [StatementIr::DeclarationEvaluation(initialization)]=plan.initialization().block().statements.as_slice() else {panic!("semantic empty BindingInitialization");};
        match &initialization.expr {
            ExprIr::ArrayDestructure {pattern,evaluation:ArrayDestructuringEvaluationIr::BindingInitialization,..}=>assert!(pattern.elements.is_empty()),
            ExprIr::ObjectDestructure {pattern,..}=>assert!(pattern.properties.is_empty()&&pattern.rest.is_none()),
            other=>panic!("empty pattern lost its semantic operation: {other:?}"),
        }
    }
}

#[test]
fn plain_async_for_of_assignment_pattern_keeps_typed_targets_before_await() {
    let program=lower_script(
        "class C {
                #value;
                async assign(iterable) {
                    let identifier;
                    const target = {};
                    for ([identifier, target.value, this.#value] of iterable) {
                        await 0;
                    }
                }
            }",
    );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let [StatementIr::DeclarationEvaluation(TypedExpr {expr:ExprIr::ArrayDestructure {pattern,evaluation:ArrayDestructuringEvaluationIr::AssignmentEvaluation,..},..})]=plan.initialization().block().statements.as_slice() else {panic!("original AssignmentEvaluation");};
    assert!(matches!(pattern.elements[0],ArrayDestructuringElementIr::Target {target:DestructuringTargetIr::AssignmentIdentifier(_),..}));
    assert!(matches!(pattern.elements[1],ArrayDestructuringElementIr::Target {target:DestructuringTargetIr::AssignmentProperty {..},..}));
    assert!(matches!(pattern.elements[2],ArrayDestructuringElementIr::Target {target:DestructuringTargetIr::AssignmentPrivate {..},..}));
}

#[test]
fn plain_async_for_of_assignment_patterns_capture_top_level_and_nested_objects() {
    let program=lower_script(
            "function owner() {
                let objectSourceKey = 'value';
                let objectTarget = {};
                let objectTargetKey = 'slot';
                let objectFallback = 1;
                let objectRestTarget = {};
                let nestedObjectSourceKey = 'value';
                let nestedObjectTarget = {};
                let nestedObjectTargetKey = 'slot';
                let nestedObjectFallback = 3;
                let nestedObjectRestTarget = {};
                async function assignObject(iterable) {
                    for ({ [objectSourceKey]: objectTarget[objectTargetKey] = objectFallback, ...objectRestTarget.rest } of iterable) {
                        await 0;
                    }
                }
                async function assignArray(iterable) {
                    for ([{ [nestedObjectSourceKey]: nestedObjectTarget[nestedObjectTargetKey] = nestedObjectFallback, ...nestedObjectRestTarget.rest }] of iterable) {
                        await 0;
                    }
                }
                return [assignObject, assignArray];
            }
            owner();",
        );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    for (name,captures) in [("assignObject",["objectSourceKey","objectTarget","objectTargetKey","objectFallback","objectRestTarget"]),("assignArray",["nestedObjectSourceKey","nestedObjectTarget","nestedObjectTargetKey","nestedObjectFallback","nestedObjectRestTarget"])] {
        let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.name==name).unwrap();
        assert_complete_async_for_of_storage(function,complete_async_for_of_plan(function));
        let actual=function.captured_bindings.iter().map(|row|row.source_name.as_str()).collect::<BTreeSet<_>>();
        for name in captures {assert!(actual.contains(name));}
    }
}

#[test]
fn plain_async_for_of_captured_binding_carries_iteration_environment() {
    let program=lower_script(
            "(async function(){ const out = []; for (const v of [1, 2]) { out.push(() => v); await 0; } })();",
        );
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async && complete_async_for_of_in(&function.body.statements).is_some()).expect("actual async iterator function");
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    let environment=plan.lexical_environment().expect("original lexical head").iteration_environment.as_ref().expect("captured per-key record");
    assert_eq!(environment.bindings.len(),1);
    assert!(environment.bindings[0].name.ends_with(".v"));
}

#[test]
fn async_loops_own_every_direct_await_continuation_and_the_final_exit() {
    for source in [
        "async function sequence() { for (let i = 0; i < 2; i++) { await 0; await 1; } }",
        "async function sequence() { let i = 0; while (i++ < 2) { await 0; await 1; } }",
        "async function* sequence() { for (let i = 0; i < 2; i++) { await 0; await 1; } }",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let function = program
            .script
            .as_ref()
            .expect("script ir should exist")
            .functions
            .iter()
            .find(|function| function.name == "sequence")
            .expect("async function should be collected");
        let plan = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::AsyncGeneratorLoop(plan) => Some(plan),
                _ => None,
            })
            .expect("loop should own a resumable iteration plan");
        let entry = plan.body().entry_state();
        let continuations = plan.suspensions().iter().map(|point| {
            assert_eq!(point.kind, ResumableSuspensionKindIr::Await);
            (point.suspend_state, point.resume_state)
        }).collect::<Vec<_>>();
        assert_eq!(continuations, [(entry, entry + 1), (entry + 1, entry + 2)], "{source}");
        assert_eq!(plan.body().end_state(), entry + 2, "{source}");
        assert!(plan.exit_state() > plan.body().end_state(), "{source}");
    }
}

#[test]
fn direct_await_sequences_reject_discontinuous_states_and_nested_suspensions() {
    let program = lower_script("async function sequence() { await 0; await 1; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .unwrap();
    let awaits = async_loop_rows(&function.body.statements).into_iter()
        .filter(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
        .collect::<Vec<_>>();
    assert_eq!(awaits.len(), 2, "expected two direct awaits: {:?}", function.body.statements);
    let first = awaits[0];
    let second = awaits[1];
    assert_eq!(
        direct_await_sequence_resume_state(first, &[second.clone()], 0),
        Ok(2)
    );
    assert_eq!(
        direct_await_sequence_resume_state(&StatementIr::Empty, &[], 0),
        Err(AwaitSequenceError::FirstAwaitRequired)
    );
    assert_eq!(
        direct_await_sequence_resume_state(
            first,
            &[StatementIr::LexicalBlock(vec![second.clone()])],
            0
        ),
        Err(AwaitSequenceError::NestedSuspension)
    );
    let mut discontinuous = second.clone();
    let StatementIr::AsyncAwait {
        suspend_state,
        resume_state,
        ..
    } = &mut discontinuous
    else {
        panic!("the second statement must be an await");
    };
    *suspend_state = 2;
    *resume_state = 3;
    assert_eq!(
        direct_await_sequence_resume_state(first, &[discontinuous], 0),
        Err(AwaitSequenceError::StateMismatch {
            expected_suspend_state: 1,
            suspend_state: 2,
            resume_state: 3,
        })
    );
}

#[test]
fn complete_async_classic_loops_admit_nested_regions_and_abrupt_completions() {
    for source in [
        "(async function(){ for (let i = 0; i < 2; i++) { try { await 0; } catch (e) {} } })();",
        "(async function(){ for (let i = 0; i < 2; i++) { await 0; break; } })();",
        "(async function(){ for (let i = 0; i < 2; i++) { await 0; try { await 1; } catch (e) {} } })();",
        "(async function(){ let n = 0; do { n++; await 0; } while (n < 2); })();",
        "(async function(){ for (const k in { a: 1 }) { await 0; } })();",
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
}

#[test]
fn async_iterator_initialization_owns_await_before_original_put_and_body() {
    let source="(async function(){ const target = {}; for (target[await 0] of [1]) { await 0; } })();";
    let program=lower_script(source);
    assert!(program.is_wasm_supported(),"{source}: {:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.protocol.execution_kind()==FunctionExecutionKind::Async).unwrap();
    let plan=complete_async_for_of_plan(function);
    assert_complete_async_for_of_storage(function,plan);
    assert!(matches!(complete_async_for_of_put(plan),DestructuringTargetIr::AssignmentProperty {key:DestructuringPropertyKeyIr::Computed(_),..}));
    let tape=plan.suspensions();
    assert_eq!(tape.len(),2);
    assert!(tape[0].resume_state<=plan.initialization().end_state());
    assert!(tape[1].suspend_state>=plan.body().entry_state());
}

#[test]
fn nested_complete_async_iterators_keep_next_body_and_close_point_kinds() {
    let program=lower_script("async function nested(input) { for await (const value of await input) { try { for await (const [item = await 1] of value) { await item; } } finally { await 2; } } }");
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.name=="nested").unwrap();
    let plan=complete_async_for_of_plan(function);
    let kinds=plan.suspensions().iter().map(|point|point.kind).collect::<Vec<_>>();
    assert_eq!(kinds,[ResumableSuspensionKindIr::Await,ResumableSuspensionKindIr::ForAwaitNext,ResumableSuspensionKindIr::ForAwaitNext,ResumableSuspensionKindIr::Await,ResumableSuspensionKindIr::Await,ResumableSuspensionKindIr::ForAwaitClose,ResumableSuspensionKindIr::Await,ResumableSuspensionKindIr::ForAwaitClose]);
    for point in plan.suspensions() {
        assert_eq!(point.resume_environment,match point.kind {
            ResumableSuspensionKindIr::ForAwaitNext|ResumableSuspensionKindIr::ForAwaitClose=>ResumableResumeEnvironmentIr::SavedLexicalChain,
            ResumableSuspensionKindIr::Await=>ResumableResumeEnvironmentIr::InvocationOuter,
            ResumableSuspensionKindIr::Yield=>panic!("plain Async cannot mint a Yield point"),
        });
    }
}

#[test]
fn ordinary_iterator_head_and_array_default_preserve_unadopted_yield_protocol() {
    let program=lower_script("function* values(input) { for (const [value = yield 'default'] of yield input) { if (value) { yield value; } else { yield 0; } } }");
    assert!(program.is_wasm_supported(),"{:?}",program.diagnostics);
    let function=program.script.as_ref().unwrap().functions.iter().find(|function|function.name=="values").unwrap();
    let plan=function.body.statements.iter().find_map(|statement|match statement {StatementIr::AsyncGeneratorForOf(plan)=>Some(plan),_=>None}).expect("complete ordinary owner");
    assert_eq!(plan.execution(),ResumableRegionProtocolIr::Generator);
    assert!(matches!(plan.protocol(),AsyncGeneratorIteratorProtocolIr::Sync));
    assert_eq!(plan.suspensions().len(),4);
    assert!(plan.suspensions().iter().all(|point|point.kind==ResumableSuspensionKindIr::Yield));
    assert!(plan.suspensions()[0].resume_state<=plan.head().region().end_state());
    assert!(plan.suspensions()[1].resume_state<=plan.initialization().end_state());
    assert!(plan.suspensions()[2].suspend_state>=plan.body().entry_state());
}

/// Formerly refused shapes must use their actual checked owner once admitted.
#[test]
fn a_refused_generator_declaration_reports_its_yield_shape() {
    for (source, message) in [
        (
            "function* g() { for (let [i] = [yield 1]; i < 3; i++) { yield i; } } g();",
            "classic loop source requires an owned staged expression, lexical binding and control region",
        ),
        (
            "function* g() { switch (1) { case 1: yield 1; } } g();",
            "a yield inside a statement kind with no resumable lowering",
        ),
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "formerly {message}: {source}: {:?}", program.diagnostics);
        let function = program.script.as_ref().unwrap().functions.iter()
            .find(|function| function.name == "g").unwrap();
        assert!(async_loop_rows(&function.body.statements).into_iter().any(|statement|
            matches!(statement, StatementIr::AsyncGeneratorLoop(plan)
                if plan.execution() == ResumableRegionProtocolIr::Generator)
                || matches!(statement, StatementIr::OrdinaryGeneratorLoop(_))
                || matches!(statement, StatementIr::OrdinaryGeneratorSwitch(_))),
            "the admitted shape must carry its actual resumable owner");
        assert!(
            !program
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("function or class declaration")),
            "a generator must not be reported as a function or class declaration: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn async_generator_loop_reuses_one_await_state_across_iterations() {
    let program = lower_script(
        "async function* callAsync(iterations, pushAwait) {
                 for (let i = 0; i < iterations; i++) {
                     await pushAwait(i);
                 }
                 return 0;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "callAsync")
        .expect("async generator declaration should be collected");

    let statements = async_loop_rows(&function.body.statements);
    let plan = statements.iter().find_map(|statement| match statement {
        StatementIr::AsyncGeneratorLoop(plan) => Some(plan.as_ref()),
        _ => None,
    }).expect("a reusable loop Await");
    let (suspend_state, resume_state) = statements.iter().find_map(|statement| match statement {
        StatementIr::AsyncAwait { suspend_state, resume_state, resume_mode: AsyncResumeModeIr::Return, .. } => Some((*suspend_state, *resume_state)),
        _ => None,
    }).expect("following return Await");
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::AsyncGenerator);
    assert_eq!(plan.suspensions().len(), 1);
    assert_eq!(plan.suspensions()[0].kind, ResumableSuspensionKindIr::Await);
    assert_eq!(plan.suspensions()[0].suspend_state, plan.body().entry_state());
    assert_eq!(plan.suspensions()[0].resume_state, plan.body().end_state());
    assert_eq!(suspend_state, plan.exit_state());
    assert_eq!(resume_state, plan.exit_state() + 1);
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding == plan.value_binding()));
}

#[test]
fn mixed_classic_loops_own_break_continue_and_suspended_update() {
    for source in [
        "async function* stream() { for (;;) { await 0; break; } }",
        "async function* stream() { for (let i = 0; i < 2; i++) { await 0; continue; } }",
        "async function* stream() { for (let i = 0; i < 2; await 0) {} }",
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
}
