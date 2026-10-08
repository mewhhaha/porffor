use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorForOfIr, DestructuringPropertyKeyIr, DestructuringTargetIr, ExprIr,
    FunctionIr, GeneratorSuspensionPointIr, NativeErrorKind, ObjectDestructuringOperationView,
    ResumableRegionProtocolIr, StatementIr, TypedExpr,
};

fn lower_values(source: &str) -> FunctionIr {
    let unit = parse(source, ParseOptions::script()).expect("generator source must parse");
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR must exist")
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .expect("values must be lowered")
}

fn lexical_name(statements: &[StatementIr]) -> &str {
    let mut rows = Vec::new();
    statement_rows(statements, &mut rows);
    rows.into_iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .expect("a direct lexical declaration must remain explicit")
}

#[test]
fn branch_lexicals_have_distinct_activation_slots_on_both_sides_of_yield() {
    let function = lower_values(
        "function* values(flag) {
             let value = 99;
             if (flag) {
                 const value = 7;
                 yield value;
                 let after = value;
             } else {
                 let value = 8;
                 yield value;
                 const after = value;
             }
             return value;
         }",
    );
    let branch = function
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::GeneratorIf { .. }))
        .expect("lexical branches must use resumable conditional IR");
    let StatementIr::GeneratorIf {
        then_before_yield,
        then_yield_statement,
        then_after_yield,
        else_before_yield,
        else_yield_statement,
        else_after_yield,
        then_resume_state,
        else_resume_state,
        ..
    } = branch
    else {
        unreachable!("the search selected GeneratorIf");
    };
    assert!(then_yield_statement.is_some());
    assert!(else_yield_statement.is_some());
    assert_ne!(then_resume_state, else_resume_state);

    let names = [
        lexical_name(&function.body.statements),
        lexical_name(then_before_yield),
        lexical_name(then_after_yield),
        lexical_name(else_before_yield),
        lexical_name(else_after_yield),
    ];
    assert_eq!(
        names.into_iter().collect::<BTreeSet<_>>().len(),
        names.len()
    );
    let slots = names.map(|name| {
        let matching = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "{name}: {:?}",
            function.owned_env_bindings
        );
        matching[0].slot
    });
    assert_eq!(
        slots.into_iter().collect::<BTreeSet<_>>().len(),
        slots.len()
    );
}

#[test]
fn branch_declarations_preserve_unplanned_suspension_and_environment_rejections() {
    for (source, yield_count, captured) in [
        ("function* values(flag) { if (flag) { const value = yield 1; yield value; } }", 2, false),
        ("function* values(flag) { if (flag) { let value = 1; { yield value; } } }", 1, false),
        ("function* values(flag) { if (flag) { let value = 1; yield value; yield 2; } }", 2, false),
        ("function* values(flag) { if (flag) { let value = 1; const read = () => value; yield read(); } }", 1, true),
        ("function* values(flag) { if (flag) { class Value {} yield Value; } }", 1, false),
    ] {
        let unit = parse(source, ParseOptions::script()).expect("generator source must parse");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let script = program.script.unwrap();
        let function = script.functions.iter().find(|function| function.name == "values").unwrap();
        let mut rows = Vec::new();
        statement_rows(&function.body.statements, &mut rows);
        let branch = rows.iter().find_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorIf(branch) => Some(branch),
            _ => None,
        }).expect("completed branches must retain their checked If owner");
        assert_eq!(branch.entry_state(), 0);
        assert_eq!(branch.then_branch().entry_state(), 1);
        assert_eq!(branch.then_branch().end_state(), 1 + yield_count);
        assert_eq!(branch.else_branch().entry_state(), 2 + yield_count);
        assert_eq!(branch.else_branch().end_state(), 2 + yield_count);
        assert_eq!(branch.exit_state(), 3 + yield_count);
        let mut branch_rows = Vec::new();
        statement_rows(&branch.then_branch().block().statements, &mut branch_rows);
        let points = branch_rows.iter().filter_map(|statement| match statement {
            StatementIr::GeneratorYield { suspend_state, resume_state, .. } => Some(GeneratorSuspensionPointIr {
                suspend_state: *suspend_state,
                resume_state: *resume_state,
            }),
            _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(points.len(), yield_count as usize);
        assert_eq!(function.generator_plan.as_ref().unwrap().suspension_points, points);
        assert!(
            branch.else_branch().block().statements.iter().all(|statement| matches!(statement, StatementIr::Empty)),
            "the absent branch must not evaluate the declarations"
        );
        if captured {
            let value = lexical_name(&branch.then_branch().block().statements);
            let environment = branch.then_branch().block().lexical_environment.as_ref()
                .expect("the closure retains the original branch environment");
            assert!(environment.bindings.iter().any(|binding| binding.name == value));
            assert!(script.functions.iter().any(|function| function.captured_bindings.iter()
                .any(|binding| binding.name == value)));
        }
    }
}

fn statement_rows<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        output.push(statement);
        match statement {
            StatementIr::Block(block) => statement_rows(&block.statements, output),
            StatementIr::LexicalBlock(items) => statement_rows(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                statement_rows(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::Labelled { statement, .. } => {
                statement_rows(std::slice::from_ref(statement), output)
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    statement_rows(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                statement_rows(&plan.head().region().block().statements, output);
                statement_rows(&plan.initialization().block().statements, output);
                statement_rows(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                statement_rows(&plan.head().region().block().statements, output);
                statement_rows(&plan.initialization_region().block().statements, output);
                statement_rows(&plan.body().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                statement_rows(&plan.then_branch().block().statements, output);
                statement_rows(&plan.else_branch().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    statement_rows(&region.block().statements, output);
                }
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    statement_rows(&region.block().statements, output);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                statement_rows(&try_block.statements, output);
                statement_rows(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                statement_rows(&try_block.statements, output);
                statement_rows(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                statement_rows(&try_block.statements, output);
                statement_rows(&catch_block.statements, output);
                statement_rows(&finally_block.statements, output);
            }
            _ => {}
        }
    }
}

fn generator_for_of_plan(function: &FunctionIr) -> &AsyncGeneratorForOfIr {
    let mut rows = Vec::new();
    statement_rows(&function.body.statements, &mut rows);
    rows.into_iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncGeneratorForOf(plan)
                if plan.execution() == ResumableRegionProtocolIr::Generator =>
            {
                Some(plan.as_ref())
            }
            _ => None,
        })
        .expect("current generator owns a synchronous iterator plan")
}

fn initialization_value(plan: &AsyncGeneratorForOfIr) -> &TypedExpr {
    plan.initialization()
        .block()
        .statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            StatementIr::DeclarationEvaluation(value) | StatementIr::Expression(value) => {
                Some(value)
            }
            _ => None,
        })
        .expect("the original per-key initialization remains explicit")
}

fn property_head(
    plan: &AsyncGeneratorForOfIr,
) -> (
    &TypedExpr,
    &DestructuringPropertyKeyIr,
    lila_ir::Strictness,
    &TypedExpr,
) {
    let ExprIr::ObjectDestructuringOperation(operation) = &initialization_value(plan).expr else {
        panic!("the actual retained property Reference supplies PutValue");
    };
    let ObjectDestructuringOperationView::PutTarget {
        target:
            DestructuringTargetIr::AssignmentProperty {
                target,
                key,
                strictness,
            },
        value,
    } = operation.use_view()
    else {
        panic!("the ordinary property target remains explicit");
    };
    (target, key, *strictness, value)
}

fn retained_initializer<'a>(plan: &'a AsyncGeneratorForOfIr, read: &TypedExpr) -> &'a TypedExpr {
    let ExprIr::Identifier(name) = &read.expr else {
        panic!("the original operand is retained once")
    };
    plan.initialization()
        .block()
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                name: published,
                init,
                ..
            } if published == name => Some(init),
            _ => None,
        })
        .expect("the original raw Reference operand is published before PutValue")
}

#[test]
fn local_for_of_control_preserves_yielding_finally_and_following_states() {
    let function = lower_values(
        "function* values(source) {
             for (const value of source) {
                 try {
                     yield value;
                     if (value === 1) continue;
                     break;
                 } finally { yield \"cleanup\"; }
             }
             yield \"tail\";
         }",
    );
    let plan = generator_for_of_plan(&function);
    assert_eq!(plan.entry_state(), 0);
    assert_eq!(plan.body().entry_state(), 4);
    assert_eq!(plan.body().end_state(), 11);
    assert_eq!(plan.exit_state(), 12);
    let mut rows = Vec::new();
    statement_rows(&plan.body().block().statements, &mut rows);
    let clause = rows
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::TryFinally { generator_plan, .. } => *generator_plan,
            _ => None,
        })
        .expect("body retains its yielding finalizer owner");
    assert_eq!(clause.try_exit_state, 9);
    assert_eq!(clause.finally_entry_state, Some(9));
    assert_eq!(clause.finally_exit_state, Some(11));
    assert_eq!(clause.exit_state, 11);
    let generator = function
        .generator_plan
        .as_ref()
        .expect("generator state plan");
    assert_eq!(generator.state_count, 14);
    assert_eq!(
        generator.suspension_points,
        vec![
            GeneratorSuspensionPointIr {
                suspend_state: 4,
                resume_state: 5
            },
            GeneratorSuspensionPointIr {
                suspend_state: 9,
                resume_state: 10
            },
            GeneratorSuspensionPointIr {
                suspend_state: 12,
                resume_state: 13
            },
        ]
    );
    let slots = [
        plan.head_binding().name.as_str(),
        plan.incoming_binding().name.as_str(),
        plan.value_binding().name.as_str(),
    ]
    .map(|name| {
        let matching = function
            .owned_env_bindings
            .iter()
            .filter(|owned| owned.name == name)
            .collect::<Vec<_>>();
        assert_eq!(
            matching.len(),
            1,
            "{name}: {:?}",
            function.owned_env_bindings
        );
        matching[0].slot
    });
    assert_eq!(
        slots.into_iter().collect::<BTreeSet<_>>().len(),
        slots.len()
    );
}

#[test]
fn local_for_of_control_before_and_after_yield_keeps_captured_head_owner() {
    let function = lower_values(
        "function* values(source, readers, flag) {
             for (let value of source) {
                 readers.push(() => value);
                 if (flag) continue;
                 yield value;
                 if (value) break;
                 continue;
             }
         }",
    );
    let plan = generator_for_of_plan(&function);
    assert_eq!(plan.entry_state(), 0);
    assert_eq!(plan.body().end_state(), 11);
    assert_eq!(plan.exit_state(), 12);
    let environment = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .expect("captured let head must retain a fresh iteration environment");
    assert_eq!(environment.bindings.len(), 1);
    let StatementIr::Lexical { name, init, .. } = &plan.initialization().block().statements[0]
    else {
        panic!("the actual captured head is initialized before body entry");
    };
    assert_eq!(&environment.bindings[0].name, name);
    assert!(
        matches!(&init.expr, ExprIr::Identifier(read) if read == &plan.incoming_binding().name)
    );
}

#[test]
fn local_for_of_control_preserves_foreign_owner_and_other_protocol_rejections() {
    for source in [
        "function* values(source) { outer: for (var value of source) { yield value; break outer; } }",
        "function* values(source) { outer: for (var value of source) { yield value; continue outer; } }",
        "function* values(source) { for (var value of source) { yield value; while (value) { break; } } }",
        "function* values(source) { for (var value of source) { yield value; while (value) { continue; } } }",
        "function* values(source) { for (var value of source) { yield value; switch (value) { default: break; } } }",
        "function* values(source) { for (var value of source) { yield value; inner: { continue; } } }",
        "function* values(source) { for (var value of source) { for (var inner of source) { yield inner; } } }",
        "function* values(source) { for (var value of (yield source)) { yield value; break; } }",
        "function* values(source) { for (const [value] of source) { yield value; break; } }",
        "function* values(source) { while (true) { yield source; break; } }",
        "async function values(source) { for (var value of source) { await value; break; } }",
        "async function* values(source) { for (var value of source) { yield value; break; } }",
    ] {
        let unit = parse(source, ParseOptions::script()).expect("boundary source must parse");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let script = program.script.unwrap();
        let mut rows = Vec::new();
        for function in &script.functions {
            statement_rows(&function.body.statements, &mut rows);
        }
        assert!(rows.iter().any(|statement| matches!(statement,
            StatementIr::AsyncGeneratorForOf(_) | StatementIr::AsyncGeneratorLoop(_)
                | StatementIr::OrdinaryGeneratorLoop(_))), "{source}");
    }
}

#[test]
fn identifier_assignment_head_uses_existing_target_cell_and_no_persistent_sink() {
    let function = lower_values("function* values(source, readers) { let target = 0; for (target of source) { readers.push(() => target); yield target; target = 9; yield target; } }");
    let plan = generator_for_of_plan(&function);
    let sink = &plan.incoming_binding().name;
    assert!(plan.lexical_environment().is_none());
    assert!(function
        .owned_env_bindings
        .contains(plan.incoming_binding()));
    assert!(!function
        .captured_bindings
        .iter()
        .any(|binding| &binding.name == sink));
    let prefix = initialization_value(plan);
    let ExprIr::AssignIdentifier { name, value } = &prefix.expr else {
        panic!("existing declarative target: {prefix:?}");
    };
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == sink));
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == name));
    assert_ne!(name, sink);
    assert_eq!(plan.body().end_state(), 6);
    assert_eq!(plan.exit_state(), 7);
}

#[test]
fn assignment_head_retains_const_tdz_and_strict_unresolvable_write_failures() {
    for (source, kind) in [
        ("function* values(source) { const target = 1; for (target of source) { yield target; } }", NativeErrorKind::TypeError),
        ("function* values(source) { for (target of source) { yield 1; } let target; }", NativeErrorKind::ReferenceError),
    ] {
        let function = lower_values(source);
        let plan = generator_for_of_plan(&function);
        let prefix = initialization_value(plan);
        assert!(matches!(&prefix.expr, ExprIr::Comma { lhs, rhs }
            if matches!(&lhs.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
                && matches!(&rhs.expr, ExprIr::RuntimeThrow { name, .. } if *name == kind)), "{prefix:?}");
    }
    let function = lower_values(
        "function* values(source) { 'use strict'; for (unresolved of source) { yield 1; } }",
    );
    let plan = generator_for_of_plan(&function);
    let prefix = initialization_value(plan);
    assert!(
        matches!(&prefix.expr, ExprIr::GlobalPropertyWrite { name, strictness: lila_ir::Strictness::Strict, .. } if name == "unresolved")
    );
}

#[test]
fn assignment_head_preserves_current_loop_finalizer_states_and_other_head_refusals() {
    let function = lower_values("function* values(source) { let target; for (target of source) { try { yield target; continue; } finally { yield 'cleanup'; } } yield 'tail'; }");
    let plan = generator_for_of_plan(&function);
    assert_eq!(plan.body().end_state(), 8);
    assert_eq!(plan.exit_state(), 9);
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .last()
            .unwrap()
            .suspend_state,
        9
    );
    for source in [
        "function* values(source, target) { for (target[yield 'key'] of source) { yield 1; } }",
        "function* values(source, target) { for ([target] of source) { yield 1; } }",
        "function* values(source) { for (target of (yield source)) { yield 1; } }",
        "function* values(source) { outer: for (target of source) { yield 1; break outer; } }",
        "async function* values(source) { for (target of source) { yield 1; } }",
    ] {
        let unit = parse(source, ParseOptions::script()).expect("negative source must parse");
        let program = lower(&unit);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.unwrap();
        let mut rows = Vec::new();
        for function in &script.functions {
            statement_rows(&function.body.statements, &mut rows);
        }
        assert!(
            rows.iter()
                .any(|statement| matches!(statement, StatementIr::AsyncGeneratorForOf(_))),
            "{source}"
        );
    }
}

#[test]
fn assignment_head_precedes_shadowing_body_lexical_environment() {
    let function = lower_values("function* values(source) { let target = 0; for (target of source) { let target = 7; yield target; } yield target; }");
    let plan = generator_for_of_plan(&function);
    let prefix = initialization_value(plan);
    let ExprIr::AssignIdentifier { name: outer, value } = &prefix.expr else {
        panic!("outer located cell: {prefix:?}");
    };
    assert!(
        matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    );
    let StatementIr::Block(body) = &plan.body().block().statements[0] else {
        panic!("body lexical block must remain materialized");
    };
    let inner = lexical_name(&body.statements);
    assert_ne!(outer, inner);
    assert!(body.lexical_environment.is_none());
    let cells = [outer.as_str(), inner, plan.incoming_binding().name.as_str()].map(|name| {
        let matches = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "{name}: {:?}",
            function.owned_env_bindings
        );
        matches[0].slot
    });
    assert_eq!(
        cells.into_iter().collect::<BTreeSet<_>>().len(),
        cells.len()
    );
    let mut rows = Vec::new();
    statement_rows(&function.body.statements, &mut rows);
    let yielded = rows
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::GeneratorYield { value, .. } => Some(&value.expr),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        matches!(yielded.as_slice(), [ExprIr::Identifier(body), ExprIr::Identifier(tail)]
        if body == inner && tail == outer)
    );
    assert_eq!(plan.body().end_state(), 5);
    assert_eq!(plan.exit_state(), 6);
}

#[test]
fn named_generator_self_assignment_retains_sloppy_ignore_and_strict_native_error() {
    for (directive, ignored) in [("", true), ("'use strict';", false)] {
        let source = format!("const factory = function* self(source) {{ {directive} for (self of source) {{ yield self; }} }};");
        let unit = parse(&source, ParseOptions::script()).unwrap();
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let function = program
            .script
            .unwrap()
            .functions
            .into_iter()
            .find(|function| function.name == "self")
            .unwrap();
        let plan = generator_for_of_plan(&function);
        let prefix = initialization_value(plan);
        if ignored {
            assert!(
                matches!(&prefix.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
            );
        } else {
            assert!(matches!(&prefix.expr, ExprIr::Comma { lhs, rhs }
                if matches!(&lhs.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
                    && matches!(&rhs.expr, ExprIr::RuntimeThrow { name: NativeErrorKind::TypeError, .. })));
        }
        assert!(plan.lexical_environment().is_none());
    }
}

#[test]
fn ordinary_property_head_keeps_eager_reference_before_shadowing_body_and_all_resumes() {
    let function = lower_values("function* values(source, locate, key) { for (locate()[key()] of source) { let locate = 7; yield locate; yield 'after'; } }");
    let plan = generator_for_of_plan(&function);
    let sink = &plan.incoming_binding().name;
    assert!(plan.lexical_environment().is_none());
    assert!(function
        .owned_env_bindings
        .contains(plan.incoming_binding()));
    let (base_read, key, _, value) = property_head(plan);
    let base = retained_initializer(plan, base_read);
    assert!(
        matches!(&base.expr, ExprIr::CallIndirect { .. }),
        "raw base evaluated once: {base:?}"
    );
    let DestructuringPropertyKeyIr::Computed(raw_key) = key else {
        panic!("raw computed key");
    };
    assert!(matches!(
        &retained_initializer(plan, raw_key).expr,
        ExprIr::CallIndirect { .. }
    ));
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == sink));
    assert_eq!(value.kind, lila_ir::ValueKind::Dynamic);
    assert_eq!(value.possible_kinds, lila_ir::KindSet::all_runtime_tags());
    let StatementIr::Block(body) = &plan.body().block().statements[0] else {
        panic!("body lexical scope");
    };
    let shadow = lexical_name(&body.statements);
    assert!(!matches!(&base.expr, ExprIr::Identifier(name) if name == shadow));
    assert!(body.lexical_environment.is_none());
    let shadow_cells = function
        .owned_env_bindings
        .iter()
        .filter(|binding| binding.name == shadow)
        .collect::<Vec<_>>();
    assert_eq!(shadow_cells.len(), 1);
    assert_ne!(shadow_cells[0].slot, plan.incoming_binding().slot);
    assert_ne!(shadow, sink);
    let mut rows = Vec::new();
    statement_rows(&body.statements, &mut rows);
    let yielded = rows
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::GeneratorYield { value, .. } => Some(&value.expr),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(matches!(yielded.first(), Some(ExprIr::Identifier(name)) if name == shadow));
    assert_eq!(yielded.len(), 2);
    assert_eq!(plan.body().end_state(), 6);
    assert_eq!(plan.exit_state(), 7);
}

#[test]
fn ordinary_property_head_retains_raw_primitive_nullish_and_optional_value_bases() {
    for (base, kind) in [
        ("42", lila_ir::ValueKind::Number),
        ("null", lila_ir::ValueKind::Null),
        ("undefined", lila_ir::ValueKind::Undefined),
    ] {
        for (directive, strictness) in [
            ("", lila_ir::Strictness::Sloppy),
            ("'use strict';", lila_ir::Strictness::Strict),
        ] {
            let source = format!("function* values(source, key) {{ {directive} for (({base})[key] of source) {{ yield 1; }} }}");
            let function = lower_values(&source);
            let plan = generator_for_of_plan(&function);
            let (base, key, actual_strictness, _) = property_head(plan);
            assert_eq!(retained_initializer(plan, base).kind, kind);
            assert_eq!(actual_strictness, strictness);
            assert!(matches!(key, DestructuringPropertyKeyIr::Computed(_)));
        }
    }
    let function = lower_values("function* values(source, table, key) { for ((table?.holder)[key] of source) { yield 1; } }");
    let plan = generator_for_of_plan(&function);
    let (_, key, _, value) = property_head(plan);
    assert!(matches!(key, DestructuringPropertyKeyIr::Computed(_)));
    assert!(
        matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    );
}

#[test]
fn ordinary_property_head_preserves_setter_dependency_and_yielding_finalizer_owner() {
    let unit = parse("function* values(source) { let outcome = 1; const target = { set value(value) { outcome = 'setter'; } }; for (target.value of source) { try { yield outcome + 1; continue; } finally { yield 'cleanup'; } } yield 'tail'; }", ParseOptions::script()).unwrap();
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let setter = script
        .functions
        .iter()
        .find(|function| function.protocol == lila_ir::FunctionProtocolIr::ObjectSetter)
        .expect("actual source setter");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let plan = generator_for_of_plan(function);
    let (target_read, key, _, value) = property_head(plan);
    let target = retained_initializer(plan, target_read);
    let ExprIr::Identifier(target_name) = &target.expr else {
        panic!("the source setter receiver is retained before PutValue");
    };
    let mut rows = Vec::new();
    statement_rows(&function.body.statements, &mut rows);
    assert!(rows.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init, .. } if name == target_name
            && matches!(&init.expr, ExprIr::ObjectLiteral(properties)
                if properties.iter().any(|property| matches!(property,
                    lila_ir::ObjectPropertyIr::Setter { function, .. } if function.function_id() == &setter.id))))));
    assert!(matches!(key, DestructuringPropertyKeyIr::Static(name) if name == "value"));
    assert!(
        matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    );
    assert_eq!(plan.body().end_state(), 8);
    assert_eq!(plan.exit_state(), 9);
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .last()
            .unwrap()
            .suspend_state,
        9
    );
}

#[test]
fn ordinary_property_head_preserves_private_super_and_suspended_reference_refusals() {
    for (index, source) in [
        "class Holder { #value; *values(source) { for (this.#value of source) { yield 1; } } }",
        "class Base {} class Holder extends Base { *values(source) { for (super.value of source) { yield 1; } } }",
        "function* values(source, target) { for ((yield target).value of source) { yield 1; } }",
        "function* values(source, target) { for (target[yield 'key'] of source) { yield 1; } }",
    ].into_iter().enumerate() {
        let unit = parse(source, ParseOptions::script()).expect("boundary source must parse");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let script = program.script.unwrap();
        let function = script.functions.iter().find(|function| function.name == "values" || function.name.ends_with(".values")).unwrap();
        let plan = generator_for_of_plan(function);
        assert!(!plan.initialization().block().statements.is_empty());
        assert_eq!(plan.initialization().end_state() + 1, plan.body().entry_state());
        if index == 0 {
            let initializer = initialization_value(plan);
            assert!(matches!(&initializer.expr, ExprIr::ObjectDestructuringOperation(operation)
                if matches!(operation.use_view(), ObjectDestructuringOperationView::PutTarget {
                    target: DestructuringTargetIr::AssignmentPrivate { .. }, .. })));
        } else if index == 1 {
            assert!(matches!(&initialization_value(plan).expr, ExprIr::SuperPropertyMutation(_)));
        }
    }
}

#[test]
fn generator_lexical_patterns_own_complete_cells_even_without_captures() {
    for (source, mode, count) in [
        ("function* values(source) { for (let [first, second = first, ...rest] of source) { yield second; yield rest; } }", lila_ir::BindingMode::Let, 3),
        ("function* values(source, key) { for (const {[key()]: value = 7, ...rest} of source) { yield value; yield rest; } }", lila_ir::BindingMode::Const, 2),
    ] {
        let function = lower_values(source);
        let plan = generator_for_of_plan(&function);
        let name = &plan.incoming_binding().name;
        assert_eq!(plan.head_mode(), mode);
        let environment = plan.lexical_environment().unwrap().iteration_environment.as_ref()
            .expect("the complete lexical pattern retains its original iteration record");
        assert!(function.owned_env_bindings.contains(plan.incoming_binding()));
        assert!(!function.captured_bindings.iter().any(|binding| &binding.name == name));
        let initializer = initialization_value(plan);
        let mut bindings = Vec::new();
        let value = match &initializer.expr {
            ExprIr::ArrayDestructure { value, pattern, evaluation } => {
                assert_eq!(*evaluation, lila_ir::ArrayDestructuringEvaluationIr::BindingInitialization);
                pattern.visit_bindings(&mut |mode, name| bindings.push((mode, name.to_string())));
                value
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                pattern.visit_bindings(&mut |mode, name| bindings.push((mode, name.to_string())));
                assert!(matches!(&pattern.properties[0].key, lila_ir::DestructuringPropertyKeyIr::Computed(_)));
                assert!(pattern.properties[0].default.is_some());
                value
            }
            _ => panic!("the semantic destructuring operation must own every Get and default"),
        };
        assert!(matches!(&value.expr, ExprIr::Identifier(storage) if storage == name));
        assert_eq!(value.kind, lila_ir::ValueKind::Dynamic);
        assert_eq!(value.possible_kinds, lila_ir::KindSet::all_runtime_tags());
        assert!(bindings.iter().all(|(actual_mode, _)| *actual_mode == mode));
        let bindings = bindings.into_iter().map(|(_, name)| name).collect::<BTreeSet<_>>();
        assert_eq!(bindings.len(), count);
        let cells = environment.bindings.iter().filter(|binding| bindings.contains(&binding.name)).collect::<Vec<_>>();
        assert_eq!(cells.len(), count);
        assert_eq!(environment.bindings.len(), count);
        assert_eq!(cells.iter().map(|binding| binding.slot).collect::<BTreeSet<_>>().len(), count);
        assert!(cells.iter().all(|binding| &binding.name != name));
        assert!(!function.owned_env_bindings.iter().any(|binding| bindings.contains(&binding.name)));
        assert_eq!(plan.body().end_state(), 6);
        assert_eq!(plan.exit_state(), 7);
    }
}

#[test]
fn generator_object_default_keeps_one_property_operation_and_captured_iteration_owner() {
    let unit = parse("function* values(source) { for (let {value = 7} of source) { const read = () => value; yield read; value = 8; yield read; } }", ParseOptions::script()).unwrap();
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let plan = generator_for_of_plan(function);
    let initializer = initialization_value(plan);
    let ExprIr::ObjectDestructure { pattern, .. } = &initializer.expr else {
        panic!("a single semantic ObjectDestructure owns the Get");
    };
    assert_eq!(pattern.properties.len(), 1);
    assert!(pattern.properties[0].default.is_some());
    let environment = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .expect("captured iteration environment");
    let name = &environment.bindings[0].name;
    assert!(script.functions.iter().any(|candidate| candidate
        .captured_bindings
        .iter()
        .any(|binding| &binding.name == name)));
    assert!(script.functions.iter().all(|candidate| candidate
        .captured_bindings
        .iter()
        .all(|binding| binding.name != plan.incoming_binding().name)));
}

#[test]
fn generator_empty_patterns_initialize_before_yield_without_source_cell_or_fake_mode() {
    for source in [
        "function* values(source) { for (const {} of source) { yield 1; } }",
        "function* values(source) { for (let [] of source) { yield 1; } }",
    ] {
        let function = lower_values(source);
        let plan = generator_for_of_plan(&function);
        assert!(function
            .owned_env_bindings
            .contains(plan.incoming_binding()));
        assert!(plan
            .lexical_environment()
            .unwrap()
            .iteration_environment
            .is_none());
        assert!(plan
            .lexical_environment()
            .unwrap()
            .tdz_binding_names
            .is_empty());
        assert!(matches!(
            plan.initialization().block().statements[0],
            StatementIr::DeclarationEvaluation(_)
        ));
        assert_eq!(
            plan.initialization().end_state() + 1,
            plan.body().entry_state()
        );
    }
}

#[test]
fn lexical_pattern_prefix_precedes_body_shadow_and_preserves_yielding_finalizer_states() {
    let function = lower_values("function* values(source) { for (let {value} of source) { let value = 9; try { yield value; break; } finally { yield 'cleanup'; } } yield 'tail'; }");
    let plan = generator_for_of_plan(&function);
    assert!(matches!(
        plan.initialization().block().statements[0],
        StatementIr::DeclarationEvaluation(_)
    ));
    assert!(matches!(
        plan.body().block().statements[0],
        StatementIr::Block(_) | StatementIr::LexicalBlock(_)
    ));
    assert_eq!(plan.body().end_state(), 8);
    assert_eq!(plan.exit_state(), 9);
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .last()
            .unwrap()
            .suspend_state,
        9
    );
}

#[test]
fn lexical_pattern_admission_preserves_suspended_head_foreign_protocol_and_assignment_refusals() {
    for source in [
        "function* values(source) { for (let [value = yield 1] of source) { yield value; } }",
        "function* values(source) { for (const {[yield 'key']: value} of source) { yield value; } }",
        "function* values(source) { for (let [value] of (yield source)) { yield value; } }",
        "function* values(source) { for (var [value] of source) { yield value; } }",
        "function* values(source) { let value; for ([value] of source) { yield value; } }",
        "async function* values(source) { for (let [value] of source) { yield value; } }",
    ] {
        let unit = parse(source, ParseOptions::script()).expect("negative source must parse");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let script = program.script.unwrap();
        let mut rows = Vec::new();
        for function in &script.functions {
            statement_rows(&function.body.statements, &mut rows);
        }
        assert!(rows.iter().any(|statement| matches!(statement, StatementIr::AsyncGeneratorForOf(_))), "{source}");
    }
}
