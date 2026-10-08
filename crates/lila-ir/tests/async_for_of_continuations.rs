use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorForOfIr, AsyncGeneratorIteratorProtocolIr, FunctionIr,
    ResumableRegionProtocolIr, ScriptIr, StatementIr,
};

fn lower_script(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("async loop parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

fn function(script: &ScriptIr) -> &FunctionIr {
    script
        .functions
        .iter()
        .find(|function| function.name == "task")
        .expect("async function")
}

fn rows<'a>(items: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for item in items {
        output.push(item);
        match item {
            StatementIr::Block(block) => rows(&block.statements, output),
            StatementIr::LexicalBlock(items) => rows(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                rows(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::Labelled { statement, .. } => {
                rows(std::slice::from_ref(statement), output)
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                rows(&plan.head().region().block().statements, output);
                rows(&plan.initialization().block().statements, output);
                rows(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    rows(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                rows(&plan.head().region().block().statements, output);
                rows(&plan.initialization_region().block().statements, output);
                rows(&plan.body().block().statements, output);
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                ..
            } => {
                rows(std::slice::from_ref(then_branch), output);
                if let Some(branch) = else_branch {
                    rows(std::slice::from_ref(branch), output);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&catch_block.statements, output);
                rows(&finally_block.statements, output);
            }
            _ => {}
        }
    }
}

fn loop_plan(function: &FunctionIr) -> &AsyncGeneratorForOfIr {
    let mut statements = Vec::new();
    rows(&function.body.statements, &mut statements);
    statements
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncGeneratorForOf(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("resumable synchronous iterator")
}

fn body_rows(plan: &AsyncGeneratorForOfIr) -> Vec<&StatementIr> {
    let mut statements = Vec::new();
    rows(&plan.body().block().statements, &mut statements);
    statements
}

fn block_rows(items: &[StatementIr]) -> Vec<&StatementIr> {
    let mut statements = Vec::new();
    rows(items, &mut statements);
    statements
}

fn function_rows(function: &FunctionIr) -> Vec<&StatementIr> {
    let mut statements = Vec::new();
    rows(&function.body.statements, &mut statements);
    statements
}

fn head_name(plan: &AsyncGeneratorForOfIr) -> &str {
    let [StatementIr::Lexical { name, .. }] = plan.initialization().block().statements.as_slice()
    else {
        panic!("the actual lexical head initialization remains explicit");
    };
    name
}

#[test]
fn catch_clause_states_join_before_the_following_statement() {
    let script = lower_script(
        "async function task() { for (const value of [1, 2]) { try { await value; } catch (error) {} } await 1; }",
    );
    let function = function(&script);
    let plan = loop_plan(function);
    assert_eq!(plan.entry_state(), 0);
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    assert_eq!(plan.body().entry_state(), 4);
    assert_eq!(plan.body().end_state(), 7);
    assert_eq!(plan.exit_state(), 8);
    let try_plan = body_rows(plan)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::TryCatch { async_plan, .. } => *async_plan,
            _ => None,
        })
        .expect("body retains its try/catch owner");
    assert_eq!(try_plan.try_exit_state, 6);
    assert_eq!(try_plan.catch_entry_state, Some(6));
    assert_eq!(try_plan.catch_exit_state, Some(7));
    assert!(function_rows(function)
        .into_iter()
        .any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 8,
                resume_state: 9,
                ..
            }
        )));
    for name in [
        plan.head_binding().name.as_str(),
        plan.incoming_binding().name.as_str(),
        plan.value_binding().name.as_str(),
    ] {
        assert!(
            function
                .owned_env_bindings
                .iter()
                .any(|owned| owned.name == name),
            "Iterator Record slot {name} must survive every clause await"
        );
    }
}

#[test]
fn head_body_and_catch_parameter_environments_remain_distinct() {
    let script = lower_script(
        "async function task() { const readers = []; for (let index of [1, 2]) { let local = index * 10; readers.push(() => index + local); try { await Promise.reject(index); } catch (error) { readers.push(() => error); await 0; local++; } finally { await 0; } } return readers; }",
    );
    let plan = loop_plan(function(&script));
    let head_environment = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .expect("captured head owns a fresh environment");
    let captured_name = |source_name| {
        script
            .functions
            .iter()
            .flat_map(|function| &function.captured_bindings)
            .find(|binding| binding.source_name == source_name)
            .map(|binding| binding.name.as_str())
            .expect("closure capture retains its source binding")
    };
    assert!(head_environment
        .bindings
        .iter()
        .any(|binding| binding.name == captured_name("index")));
    let body = plan
        .body()
        .block()
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Block(block) if block.lexical_environment.is_some() => Some(block),
            _ => None,
        })
        .expect("captured body lexical block is retained");
    let body_environment = body.lexical_environment.as_ref().unwrap();
    assert!(body_environment
        .bindings
        .iter()
        .any(|binding| binding.name == captured_name("local")));
    let catch_environment = block_rows(&body.statements)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::TryCatchFinally {
                catch_parameter_environment,
                ..
            } => catch_parameter_environment.as_ref(),
            _ => None,
        })
        .expect("captured catch parameter has its own environment");
    assert!(catch_environment
        .bindings
        .iter()
        .any(|binding| binding.name == captured_name("error")));
}

#[test]
fn conditional_awaits_and_finally_keep_their_nested_owners() {
    let script = lower_script(
        "async function task(flag) { for (const value of [1, 2]) { try { if (flag) await value; else await 0; } finally { await 1; } } }",
    );
    let plan = loop_plan(function(&script));
    let (try_block, finally_block, try_plan) = body_rows(plan)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::TryFinally {
                try_block,
                finally_block,
                async_plan: Some(plan),
                ..
            } => Some((try_block, finally_block, plan)),
            _ => None,
        })
        .expect("try/finally owner");
    assert!(block_rows(&try_block.statements)
        .into_iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert!(block_rows(&finally_block.statements)
        .into_iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    assert_eq!(try_plan.exit_state, plan.body().end_state());
    assert_eq!(plan.exit_state(), plan.body().end_state() + 1);
}

#[test]
fn unrelated_control_shapes_keep_explicit_capability_diagnostics() {
    for (source, expected) in [
        (
            "async function task() { for (const value of [1]) { await value; while (value) { break; } } }",
            "body without foreign branch owners",
        ),
        (
            "async function* task() { for await (const value of [1]) { await value; } }",
            "explicit await in for-await-of body",
        ),
        (
            "async function task() { for (const value of [1]) { label: { await value; } } }",
            "invalid async for-of body continuation",
        ),
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid source");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "formerly bounded {expected}: {:?}", program.diagnostics);
        let script = program.script.unwrap();
        assert!(matches!(loop_plan(function(&script)).protocol(), AsyncGeneratorIteratorProtocolIr::Sync | AsyncGeneratorIteratorProtocolIr::Awaited { .. }));
    }
}

#[test]
fn local_control_keeps_awaited_finally_and_following_state_ownership() {
    let script = lower_script(
        "async function task(source) {
             for (const value of source) {
                 try { await value; if (value === 1) continue; break; }
                 finally { await 0; await 0; }
             }
             await 0;
         }",
    );
    let function = function(&script);
    let plan = loop_plan(function);
    assert_eq!(plan.entry_state(), 0);
    assert_eq!(plan.body().end_state(), 9);
    assert_eq!(plan.exit_state(), 10);
    let clause = body_rows(plan)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::TryFinally { async_plan, .. } => *async_plan,
            _ => None,
        })
        .expect("retained awaited finalizer");
    assert_eq!(clause.try_exit_state, 6);
    assert_eq!(clause.finally_entry_state, Some(6));
    assert_eq!(clause.finally_exit_state, Some(9));
    assert!(function_rows(function)
        .into_iter()
        .any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 10,
                resume_state: 11,
                ..
            }
        )));
    let record_slots = [
        plan.head_binding().name.as_str(),
        plan.incoming_binding().name.as_str(),
        plan.value_binding().name.as_str(),
    ]
    .map(|name| {
        let slots = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        assert_eq!(slots.len(), 1, "{name}: {:?}", function.owned_env_bindings);
        slots[0].slot
    });
    assert_eq!(
        record_slots
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}

#[test]
fn local_control_in_checked_async_branches_keeps_fresh_head_environment() {
    let script = lower_script(
        "async function task(source, readers, flag) {
             for (let value of source) {
                 readers.push(() => value);
                 if (flag) { await value; continue; }
                 else { await 0; break; }
             }
         }",
    );
    let plan = loop_plan(function(&script));
    let environment = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .expect("captured let head retains its fresh environment");
    assert_eq!(environment.bindings.len(), 1);
    assert_eq!(environment.bindings[0].name, head_name(plan));
    assert!(body_rows(plan)
        .into_iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert_eq!(plan.exit_state(), plan.body().end_state() + 1);
}

#[test]
fn async_local_control_preserves_labelled_and_foreign_owner_refusals() {
    for source in [
        "async function task(source) { outer: for (const value of source) { await value; break outer; } }",
        "async function task(source) { outer: for (const value of source) { await value; continue outer; } }",
        "async function task(source) { for (const value of source) { await value; while (value) { continue; } } }",
        "async function task(source) { for (const value of source) { await value; switch (value) { default: break; } } }",
        "async function task(source) { for (const value of source) { await value; inner: { continue; } } }",
        "async function task(source) { for (const value of source) { for (const inner of source) { await inner; } } }",
        "async function task(source) { for (const value of (await source)) { await value; break; } }",
        "async function task(source) { for await (const value of source) { await value; break; } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("boundary source must parse");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let script = program.script.unwrap();
        assert_eq!(loop_plan(function(&script)).execution(), ResumableRegionProtocolIr::Async);
    }
}

#[test]
fn plain_async_for_await_body_owns_next_body_close_and_following_states() {
    for mode in ["var", "let", "const"] {
        let script = lower_script(&format!(
            "async function task(source) {{ for await ({mode} value of source) {{ await value; await 0; }} await 1; }}"));
        let function = function(&script);
        let plan = loop_plan(function);
        let AsyncGeneratorIteratorProtocolIr::Awaited {
            next_suspend_state,
            next_resume_state,
            close_suspend_state,
            close_resume_state,
        } = plan.protocol()
        else {
            panic!("for-await consumes its awaited execution view")
        };
        assert_eq!(
            (
                plan.entry_state(),
                next_resume_state,
                plan.body().entry_state(),
                plan.body().end_state(),
                close_resume_state,
                plan.exit_state()
            ),
            (0, 3, 5, 7, 9, 10)
        );
        assert_eq!(next_suspend_state, plan.advance_state());
        assert_eq!(close_suspend_state, plan.body().end_state() + 1);
        assert!(function_rows(function)
            .into_iter()
            .any(|statement| matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 10,
                    resume_state: 11,
                    ..
                }
            )));
        let names = [
            plan.head_binding().name.as_str(),
            plan.incoming_binding().name.as_str(),
            plan.value_binding().name.as_str(),
        ];
        let slots = names.map(|name| {
            let bindings = function
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == name)
                .collect::<Vec<_>>();
            assert_eq!(
                bindings.len(),
                1,
                "{name}: {:?}",
                function.owned_env_bindings
            );
            bindings[0].slot
        });
        assert_eq!(
            slots
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
        assert!(plan.initialization().block().statements.iter().any(|statement| matches!(statement,
            StatementIr::Lexical { init, .. } if matches!(&init.expr, lila_ir::ExprIr::Identifier(name) if name == &plan.incoming_binding().name))
            || matches!(statement, StatementIr::Var(declarations) if declarations.iter().any(|declaration| matches!(declaration.init.as_ref().map(|init| &init.expr), Some(lila_ir::ExprIr::Identifier(name)) if name == &plan.incoming_binding().name)))));
    }
}

#[test]
fn awaited_protocol_keeps_captured_head_cells_and_nested_clause_owners() {
    let script = lower_script(
        "async function task(source, readers, flag) { for await (let value of source) { readers.push(() => value); try { if (flag) { await value; continue; } else { await 0; break; } } finally { await 1; } } await 2; }");
    let plan = loop_plan(function(&script));
    let environment = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .expect("captured head retains a fresh cell per iteration");
    assert_eq!(environment.bindings.len(), 1);
    assert_eq!(environment.bindings[0].name, head_name(plan));
    assert!(!function(&script)
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == head_name(plan)));
    let AsyncGeneratorIteratorProtocolIr::Awaited {
        close_suspend_state,
        close_resume_state,
        ..
    } = plan.protocol()
    else {
        panic!("awaited view")
    };
    assert_eq!(close_suspend_state, plan.body().end_state() + 1);
    assert_eq!(close_resume_state, close_suspend_state + 1);
    assert_eq!(plan.exit_state(), close_resume_state + 1);
    let clause = body_rows(plan)
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::TryFinally {
                try_block,
                async_plan: Some(clause),
                ..
            } => Some((try_block, clause)),
            _ => None,
        })
        .expect("actual retained try/finally owner");
    assert_eq!(clause.1.entry_state, plan.body().entry_state());
    assert_eq!(clause.1.exit_state, plan.body().end_state());
    assert!(block_rows(&clause.0.statements)
        .into_iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
}

#[test]
fn eager_nested_try_remains_admitted_and_materialized_body_or_catch_cells_refuse() {
    let script = lower_script(
        "async function task(source, flag, mark) { for await (const value of source) { if (flag) { try { mark(value); } catch (error) { mark(error); } } await value; } }");
    assert!(matches!(
        loop_plan(function(&script)).protocol(),
        AsyncGeneratorIteratorProtocolIr::Awaited { .. }
    ));
    for source in [
        "async function task(source, flag, readers) { for await (const value of source) { if (flag) { let local = value; readers.push(() => local); } await value; } }",
        "async function task(source, flag, readers) { for await (const value of source) { if (flag) { try { value; } catch (error) { readers.push(() => error); } } await value; } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid source");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let plan = loop_plan(function(&script));
        assert!(body_rows(plan).into_iter().any(|statement| match statement {
            StatementIr::Block(block) => block.lexical_environment.is_some(),
            StatementIr::TryCatch { catch_parameter_environment, .. } => catch_parameter_environment.is_some(),
            _ => false,
        }), "the captured body or catch cell remains in its original physical record");
    }
}

#[test]
fn awaited_body_retains_the_bounded_eager_head_iterable_and_control_domain() {
    for source in [
        "async function task(source) { for await (const [value] of source) { await value; } }",
        "async function task(source, value) { for await (value of source) { await value; } }",
        "async function task(source) { for await (const value of await source) { await value; } }",
        "async function task(source) { for await (const value of source) { while (value) { break; } await value; } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid source");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let plan = loop_plan(function(&script));
        assert!(matches!(plan.protocol(), AsyncGeneratorIteratorProtocolIr::Awaited { .. }));
        assert!(plan.initialization().end_state() < plan.body().entry_state());
    }
}
