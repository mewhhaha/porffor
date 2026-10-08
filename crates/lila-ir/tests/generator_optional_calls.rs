use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, FunctionIr, GeneratorResumeModeIr, OptionalCallReferenceCaptureIr,
    SpecOperationIr, StatementIr, TypedExpr, YieldForm,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("generator optional source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| matches!(function.name.as_str(), "values" | "C.values"))
        .expect("ordinary generator")
}

fn all<'a>(source: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in source {
        result.push(statement);
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                all(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::LexicalBlock(source) => all(source, result),
            StatementIr::Block(block) => all(&block.statements, result),
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                all(std::slice::from_ref(then_branch.as_ref()), result);
                if let Some(branch) = else_branch {
                    all(std::slice::from_ref(branch.as_ref()), result);
                }
            }
            StatementIr::OrdinaryGeneratorIf(source) => {
                all(&source.then_branch().block().statements, result);
                all(&source.else_branch().block().statements, result);
            }
            _ => {}
        }
    }
}

fn unconditional<'a>(source: &'a [StatementIr]) -> Vec<&'a StatementIr> {
    fn visit<'a>(source: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
        for statement in source {
            match statement {
                StatementIr::LexicalBlock(source) => visit(source, result),
                StatementIr::Block(block) => visit(&block.statements, result),
                StatementIr::EmptyStatementCompletion(item) => {
                    visit(std::slice::from_ref(item.statement()), result)
                }
                statement => result.push(statement),
            }
        }
    }
    let mut result = Vec::new();
    visit(source, &mut result);
    result
}

fn expression(statement: &StatementIr) -> Option<&TypedExpr> {
    match statement {
        StatementIr::Lexical { init, .. } => Some(init),
        StatementIr::Expression(value)
        | StatementIr::DeclarationEvaluation(value)
        | StatementIr::Return(value) => Some(value),
        _ => None,
    }
}

fn expressions<'a>(value: &'a TypedExpr, result: &mut Vec<&'a TypedExpr>) {
    result.push(value);
    match &value.expr {
        ExprIr::AssignIdentifier { value, .. } => expressions(value, result),
        ExprIr::Comma { lhs, rhs } => {
            expressions(lhs, result);
            expressions(rhs, result);
        }
        ExprIr::MaterializeBinding { value, body, .. } => {
            expressions(value, result);
            expressions(body, result);
        }
        _ => {}
    }
}

fn projected<'a>(source: &[&'a StatementIr]) -> Vec<&'a TypedExpr> {
    let mut result = Vec::new();
    for statement in source {
        if let Some(value) = expression(statement) {
            expressions(value, &mut result);
        }
    }
    result
}

fn identifier(value: &TypedExpr) -> &str {
    let ExprIr::Identifier(name) = &value.expr else {
        panic!("retained activation cell")
    };
    name
}

fn publication(source: &[StatementIr]) -> (&str, &TypedExpr) {
    let Some(StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, value },
        ..
    })) = source.last()
    else {
        panic!("Normal-only selected operand publication")
    };
    (name, value)
}

fn owned(function: &FunctionIr, names: &BTreeSet<&str>) {
    let mut slots = BTreeSet::new();
    for name in names {
        let found = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == *name)
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1, "{name}: {:?}", function.owned_env_bindings);
        assert!(
            slots.insert(found[0].slot),
            "distinct live operands alias an activation slot"
        );
    }
}

fn captures<'a>(source: &[&'a StatementIr]) -> Vec<&'a OptionalCallReferenceCaptureIr> {
    projected(source)
        .into_iter()
        .filter_map(|value| match &value.expr {
            ExprIr::CaptureOptionalCallReference(capture) => Some(capture),
            _ => None,
        })
        .collect()
}

fn calls<'a>(source: &[&'a StatementIr]) -> Vec<&'a TypedExpr> {
    projected(source)
        .into_iter()
        .filter(|value| matches!(value.expr, ExprIr::CallIndirect { .. }))
        .collect()
}

fn call(value: &TypedExpr) -> (&str, Option<&str>, &[TypedExpr]) {
    let ExprIr::CallIndirect {
        callee,
        this_arg,
        args,
        direct_eval,
        ..
    } = &value.expr
    else {
        panic!("actual Call")
    };
    assert!(direct_eval.is_none());
    (
        identifier(callee),
        this_arg.as_deref().map(identifier),
        args,
    )
}

#[test]
fn multiple_link_operands_use_flat_scalar_branches_with_owned_normal_publications() {
    let function = values(
        "function* values(object, first, key, second, spread) {
        const result = object?.make(yield first)?.[yield key](...spread, yield second)();
        yield 'after'; return result;
    }",
    );
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 14);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [(2, 3), (6, 7), (10, 11), (12, 13)]
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let mut cells = BTreeSet::new();
    let live = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Boolean(true),
                        ..
                    },
                ..
            } if name.starts_with("$generator.optional.live.") => Some(name.as_str()),
            _ => None,
        })
        .expect("one durable whole-suffix live cell");
    cells.insert(live);
    let mut layouts = Vec::new();
    let mut operand_results = BTreeSet::new();
    for statement in &statements {
        let StatementIr::OrdinaryGeneratorIf(source) = statement else {
            continue;
        };
        let skipped = source.then_branch();
        let selected = source.else_branch();
        layouts.push((
            source.entry_state(),
            skipped.entry_state(),
            skipped.end_state(),
            selected.entry_state(),
            selected.end_state(),
            source.exit_state(),
        ));
        assert_eq!(skipped.entry_state(), skipped.end_state());
        let ExprIr::StrictEquality { lhs, rhs, .. } = &source.condition().expr else {
            panic!("live skip predicate")
        };
        assert_eq!(identifier(lhs), live);
        assert!(matches!(rhs.expr, ExprIr::Boolean(false)));
        let (result, skipped_value) = publication(&skipped.block().statements);
        let skipped_value = identifier(skipped_value);
        assert!(
            statements.iter().any(|statement| matches!(statement,
            StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. }
                if name == skipped_value)),
            "the eager retained skipped value is undefined"
        );
        assert!(operand_results.insert(result));
        cells.insert(result);
        cells.insert(skipped_value);
        let mut skipped_statements = Vec::new();
        all(&skipped.block().statements, &mut skipped_statements);
        assert!(
            !skipped_statements.iter().any(|statement| matches!(
                statement,
                StatementIr::GeneratorYield { .. }
                    | StatementIr::AsyncAwait { .. }
                    | StatementIr::OrdinaryGeneratorIf(_)
            )),
            "skipped operand owns no suspension"
        );
        let mut selected_statements = Vec::new();
        all(&selected.block().statements, &mut selected_statements);
        let yielded = selected_statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| {
                if let StatementIr::GeneratorYield {
                    form: YieldForm::Plain,
                    suspend_state,
                    resume_state,
                    resume_mode: GeneratorResumeModeIr::AssignIdentifier(received),
                    ..
                } = statement
                {
                    Some((index, suspend_state, resume_state, received.as_str()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            yielded.len(),
            1,
            "the original selected source owns one plain Yield"
        );
        let (yield_index, suspend_state, resume_state, received) = yielded[0];
        assert_eq!(*suspend_state, selected.entry_state());
        assert_eq!(*resume_state, selected.end_state());
        cells.insert(received);
        assert!(selected_statements[..yield_index].iter().any(
            |statement| matches!(statement, StatementIr::Lexical { name, .. } if name == received)
        ));
        assert_eq!(publication(&selected.block().statements).0, result);
        assert!(!selected_statements[..=yield_index].iter().any(|statement| matches!(statement,
            StatementIr::Expression(TypedExpr { expr: ExprIr::AssignIdentifier { name, .. }, .. }) if name == result)),
            "only Normal resumption reaches the complete selected result publication");
    }
    assert!(
        !statements
            .iter()
            .any(|statement| matches!(statement, StatementIr::GeneratorIf { .. })),
        "optional operands use the complete ordinary continuation owner"
    );
    assert_eq!(
        layouts,
        [
            (0, 1, 1, 2, 3, 4),
            (4, 5, 5, 6, 7, 8),
            (8, 9, 9, 10, 11, 12),
        ]
    );
    let common = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Undefined,
                        ..
                    },
                ..
            } if name.starts_with("$generator.branch.result.")
                && !operand_results.contains(name.as_str()) =>
            {
                Some(name.as_str())
            }
            _ => None,
        })
        .expect("one completed whole-chain result cell");
    assert!(
        statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(saved), .. }, .. }
            if saved == common && name != common)),
        "actual declaration consumes completed chain result"
    );
    cells.insert(common);
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. } if name == common)));
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::If { condition, then_branch, else_branch: None }
        if matches!(&condition.expr, ExprIr::Identifier(name) if name == live) && {
            let mut guarded = Vec::new(); all(std::slice::from_ref(then_branch.as_ref()), &mut guarded);
            guarded.iter().any(|statement| matches!(statement,
                StatementIr::Expression(TypedExpr { expr: ExprIr::AssignIdentifier { name, .. }, .. }) if name == common))
        })), "only the live normal suffix publishes the common result");
    owned(&function, &cells);
}

#[test]
fn references_call_results_and_spread_snapshots_are_retained_across_later_yields() {
    let function = values(
        "function* values(object, first, key, second, spread) {
        return object?.make(yield first)?.[yield key](...spread, yield second)();
    }",
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let captures = captures(&statements);
    assert_eq!(captures.len(), 2);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 3);
    let first = call(call_values[0]);
    let second = call(call_values[1]);
    let final_call = call(call_values[2]);
    assert_eq!(first.1, Some(captures[0].receiver().storage_name()));
    assert_eq!(second.1, Some(captures[1].receiver().storage_name()));
    assert_ne!(first.1, second.1);
    assert!(final_call.1.is_none() && final_call.2.is_empty());
    let child = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical { name, init, .. } if init == call_values[0] => {
                Some((index, name))
            }
            _ => None,
        })
        .expect("first completed Call saved before later key");
    let ExprIr::OptionalPropertyChain { target, .. } = &captures[1].chain_expression().expr else {
        panic!("last property Reference")
    };
    assert_eq!(identifier(target), child.1);
    assert!(
        statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init, .. } if name == final_call.0 && init == call_values[1]))
    );
    let yield_indexes = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            matches!(statement, StatementIr::GeneratorYield { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(yield_indexes.len(), 3);
    assert!(yield_indexes[0] < child.0 && child.0 < yield_indexes[1]);
    let capture_indexes = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            (!captures_for_statement(statement).is_empty()).then_some(index)
        })
        .collect::<Vec<_>>();
    assert!(capture_indexes[0] < yield_indexes[0]);
    assert!(yield_indexes[1] < capture_indexes[1] && capture_indexes[1] < yield_indexes[2]);
    let ExprIr::CapturedArgumentList(list) = &second.2[0].expr else {
        panic!("selected private spread snapshot")
    };
    let list_name = identifier(list.binding());
    let spread_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::CaptureArgumentList(_), .. }, .. } if name == list_name)).unwrap();
    assert!(capture_indexes[1] < spread_index && spread_index < yield_indexes[2]);
    let names = [
        first.0,
        first.1.unwrap(),
        second.0,
        second.1.unwrap(),
        final_call.0,
        child.1,
        list_name,
    ];
    owned(&function, &names.into_iter().collect());
}

fn captures_for_statement(statement: &StatementIr) -> Vec<&OptionalCallReferenceCaptureIr> {
    captures(&[statement])
}

#[test]
fn first_optional_call_captures_ordinary_property_reference_before_its_argument_yield() {
    let function =
        values("function* values(object, argument) { return object.method?.(yield argument); }");
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    assert!(captures(&statements).is_empty());
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, _) = call(call_values[0]);
    let receiver = receiver.unwrap();
    let get = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::SpecOperation { operation: SpecOperationIr::GetV, operands }, .. }, .. }
            if name == callee && operands.len() == 2 && identifier(&operands[0]) == receiver)).unwrap();
    let yielded = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
        .unwrap();
    assert!(get < yielded);
    owned(&function, &[callee, receiver].into_iter().collect());
}

#[test]
fn constructor_value_finishes_optional_chain_before_unconditional_outer_argument_yield() {
    let function = values("function* values(factory, first, second) { return new (factory?.create(yield first))(yield second); }");
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 6);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [(2, 3), (4, 5)]
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let construct = projected(&statements)
        .into_iter()
        .find(|value| matches!(value.expr, ExprIr::Construct { .. }))
        .unwrap();
    let ExprIr::Construct { callee, args, .. } = &construct.expr else {
        unreachable!()
    };
    let callee = identifier(callee);
    let saved = statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, .. } if name == callee)
        })
        .unwrap();
    let outer_yield = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::GeneratorYield {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .unwrap();
    assert!(saved < outer_yield);
    assert_eq!(args.len(), 1);
    owned(
        &function,
        &[callee, identifier(&args[0])].into_iter().collect(),
    );
}

#[test]
fn grouped_terminal_call_publishes_a_value_and_pins_it_before_the_unconditional_outer_yield() {
    let function = values("function* values(object, key, inner, outer) { return (object?.[yield key](yield inner))(yield outer); }");
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 10);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [(2, 3), (6, 7), (8, 9)]
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let references = captures(&statements);
    assert_eq!(references.len(), 1);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 2);
    let inner = call(call_values[0]);
    let outer = call(call_values[1]);
    assert_eq!(inner.1, Some(references[0].receiver().storage_name()));
    assert!(outer.1.is_none());
    assert_eq!(inner.2.len(), 1);
    assert_eq!(outer.2.len(), 1);
    let (terminal_index, terminal) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical { name, init, .. } if init == call_values[0] => {
                Some((index, name.as_str()))
            }
            _ => None,
        })
        .expect("selected terminal Call value is retained");
    let (pin_index, common) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Identifier(common),
                        ..
                    },
                ..
            } if name == outer.0 => Some((index, common.as_str())),
            _ => None,
        })
        .expect("completed chain Value supplies the saved outer callee");
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. } if name == common)),
        "skipped inner suffix leaves the common value undefined");
    let publications = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::AssignIdentifier { name, value },
                ..
            }) if name == common => Some((index, value.as_ref())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(publications.len(), 1);
    assert_eq!(identifier(publications[0].1), terminal);
    let live = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Boolean(true),
                        ..
                    },
                ..
            } if name.starts_with("$generator.optional.live.") => Some(name.as_str()),
            _ => None,
        })
        .unwrap();
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::If { condition, then_branch, else_branch: None }
        if matches!(&condition.expr, ExprIr::Identifier(name) if name == live) && {
            let mut selected = Vec::new(); all(std::slice::from_ref(then_branch.as_ref()), &mut selected);
            selected.iter().any(|statement| matches!(statement,
                StatementIr::Expression(TypedExpr { expr: ExprIr::AssignIdentifier { name, .. }, .. }) if name == common))
        })), "terminal publication belongs only to the live normal suffix");
    let mut cells = [inner.0, inner.1.unwrap(), terminal, common, outer.0, live]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let layouts = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorIf(source) => {
                let skipped = source.then_branch();
                let selected = source.else_branch();
                assert_eq!(skipped.entry_state(), skipped.end_state());
                let (result, skipped_value) = publication(&skipped.block().statements);
                let skipped_value = identifier(skipped_value);
                assert!(statements.iter().any(|statement| matches!(statement,
                StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. }
                    if name == skipped_value)));
                assert_eq!(publication(&selected.block().statements).0, result);
                let mut selected_statements = Vec::new();
                all(&selected.block().statements, &mut selected_statements);
                let yielded = selected_statements
                    .iter()
                    .filter_map(|statement| match statement {
                        StatementIr::GeneratorYield {
                            form: YieldForm::Plain,
                            suspend_state,
                            resume_state,
                            resume_mode: GeneratorResumeModeIr::AssignIdentifier(received),
                            ..
                        } => Some((suspend_state, resume_state, received.as_str())),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(yielded.len(), 1);
                let (suspend_state, resume_state, received) = yielded[0];
                assert_eq!(*suspend_state, selected.entry_state());
                assert_eq!(*resume_state, selected.end_state());
                cells.extend([result, skipped_value, received]);
                Some((
                    source.entry_state(),
                    skipped.entry_state(),
                    skipped.end_state(),
                    selected.entry_state(),
                    selected.end_state(),
                    source.exit_state(),
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(layouts, [(0, 1, 1, 2, 3, 4), (4, 5, 5, 6, 7, 8)]);
    assert!(
        unconditional(&function.body.statements)
            .iter()
            .any(|statement| matches!(
                statement,
                StatementIr::GeneratorYield {
                    suspend_state: 8,
                    resume_state: 9,
                    form: YieldForm::Plain,
                    ..
                }
            )),
        "grouping ends inner shorting before the ordinary outer argument Yield"
    );
    let (outer_yield_index, received) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::GeneratorYield {
                suspend_state: 8,
                resume_state: 9,
                resume_mode: GeneratorResumeModeIr::AssignIdentifier(received),
                ..
            } => Some((index, received.as_str())),
            _ => None,
        })
        .unwrap();
    let argument = identifier(&outer.2[0]);
    let argument_index = statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == argument && source == received)
        })
        .unwrap();
    assert!(
        terminal_index < publications[0].0
            && publications[0].0 < pin_index
            && pin_index < outer_yield_index
            && outer_yield_index < argument_index
    );
    cells.insert(received);
    cells.insert(argument);
    owned(&function, &cells);
}

#[test]
fn grouped_terminal_call_tag_retains_the_actual_template_owner_before_outer_substitutions() {
    let function = values("function* values(object, inner, first, second) { return (object?.make(yield inner))`head${first}${yield second}tail`; }");
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 6);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [(2, 3), (4, 5)]
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 2);
    let inner = call(call_values[0]);
    let outer = call(call_values[1]);
    assert!(inner.1.is_some() && outer.1.is_none());
    assert_eq!(outer.2.len(), 3);
    let (pin, common) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Identifier(source),
                        ..
                    },
                ..
            } if name == outer.0 => Some((index, source.as_str())),
            _ => None,
        })
        .unwrap();
    let template_name = identifier(&outer.2[0]);
    let (template_index, template) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::TemplateObject(template),
                        ..
                    },
                ..
            } if name == template_name => Some((index, template)),
            _ => None,
        })
        .expect("the original parsed template supplies the cached object");
    let owner = function
        .template_source
        .as_ref()
        .expect("actual parsed Script template owner");
    assert_eq!(template.site_id.source(), owner.id());
    assert!(template.site_id.cache_slot() < owner.site_count());
    assert_eq!(template.raw, ["head", "", "tail"]);
    assert_eq!(
        template.cooked,
        [
            Some(String::from("head")),
            Some(String::new()),
            Some(String::from("tail"))
        ]
    );
    let first = identifier(&outer.2[1]);
    let first_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. } if name == first && source == "first")).unwrap();
    let (outer_yield, received) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::GeneratorYield {
                suspend_state: 4,
                resume_state: 5,
                resume_mode: GeneratorResumeModeIr::AssignIdentifier(received),
                ..
            } => Some((index, received.as_str())),
            _ => None,
        })
        .unwrap();
    let second = identifier(&outer.2[2]);
    let second_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. } if name == second && source == received)).unwrap();
    assert!(
        pin < template_index
            && template_index < first_index
            && first_index < outer_yield
            && outer_yield < second_index
    );
    assert!(unconditional(&function.body.statements).iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::TemplateObject(_), .. }, .. } if name == template_name)),
        "GetTemplateObject is unconditional after the completed inner group");
    owned(
        &function,
        &[
            common,
            inner.0,
            inner.1.unwrap(),
            outer.0,
            template_name,
            first,
            received,
            second,
        ]
        .into_iter()
        .collect(),
    );
}

#[test]
fn grouped_terminal_property_captures_only_the_saved_child_before_outer_spread_and_yield() {
    let function = values("function* values(object, first, last, spread, argument) { return (object?.[yield first]?.[yield last])(...spread, yield argument); }");
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 10);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [(2, 3), (6, 7), (8, 9)]
    );
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let references = captures(&statements);
    assert_eq!(
        references.len(),
        1,
        "only the terminal Property preserves the outer Reference"
    );
    let reference = references[0];
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    let receiver = receiver.expect("grouped property supplies its raw Reference receiver");
    assert_eq!(receiver, reference.receiver().storage_name());
    let ExprIr::OptionalPropertyChain { target, chain } = &reference.chain_expression().expr else {
        panic!("terminal selected Get")
    };
    assert_eq!(chain.len(), 1);
    let child = identifier(target);
    let child_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::OptionalPropertyChain { chain, .. }, .. }, .. }
            if name == child && chain.len() == 1)).expect("earlier Get is retained as a Value, never repeated at final capture");
    let receiver_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. } if name == receiver)).unwrap();
    let (capture_index, selected) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::CaptureOptionalCallReference(capture),
                        ..
                    },
                ..
            } if capture.receiver().storage_name() == receiver => Some((index, name.as_str())),
            _ => None,
        })
        .unwrap();
    let (pin_index, common) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::Identifier(common),
                        ..
                    },
                ..
            } if name == callee => Some((index, common.as_str())),
            _ => None,
        })
        .unwrap();
    let publications = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::AssignIdentifier { name, value },
                ..
            }) if name == common => Some((index, value.as_ref())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(publications.len(), 1);
    assert_eq!(identifier(publications[0].1), selected);
    assert!(unconditional(&function.body.statements).iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. } if name == receiver)),
        "shorting must retain an unconditionally declared undefined receiver");
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::If { then_branch, else_branch: None, .. } if {
            let mut guarded = Vec::new(); all(std::slice::from_ref(then_branch.as_ref()), &mut guarded);
            captures(&guarded).iter().any(|capture| capture.receiver().storage_name() == receiver)
        })), "only the selected live suffix writes the terminal receiver and performs Get");
    let yields = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::GeneratorYield { suspend_state, .. } => Some((index, *suspend_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        yields.iter().map(|(_, state)| *state).collect::<Vec<_>>(),
        [2, 6, 8]
    );
    let ExprIr::CapturedArgumentList(list) = &args[0].expr else {
        panic!("outer spread snapshot")
    };
    let spread = identifier(list.binding());
    let spread_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::CaptureArgumentList(_), .. }, .. } if name == spread)).unwrap();
    assert!(receiver_index < yields[0].0 && yields[0].0 < child_index && child_index < yields[1].0);
    assert!(
        yields[1].0 < capture_index
            && capture_index < publications[0].0
            && publications[0].0 < pin_index
    );
    assert!(pin_index < spread_index && spread_index < yields[2].0);
    assert!(
        unconditional(&function.body.statements)
            .iter()
            .any(|statement| matches!(
                statement,
                StatementIr::GeneratorYield {
                    suspend_state: 8,
                    resume_state: 9,
                    ..
                }
            )),
        "grouping makes the outer argument Yield unconditional after an inner nullish result"
    );
    assert_eq!(args.len(), 2);
    owned(
        &function,
        &[
            callee,
            receiver,
            child,
            selected,
            common,
            spread,
            identifier(&args[1]),
        ]
        .into_iter()
        .collect(),
    );
}

#[test]
fn grouped_terminal_property_tag_retains_receiver_and_original_template_before_substitutions() {
    let function = values("function* values(object, key, first, last) { return (object?.[yield key])`head${first}${yield last}tail`; }");
    assert_eq!(function.generator_plan.as_ref().unwrap().state_count, 6);
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    let references = captures(&statements);
    assert_eq!(references.len(), 1);
    let call_values = calls(&statements);
    assert_eq!(call_values.len(), 1);
    let (callee, receiver, args) = call(call_values[0]);
    let receiver = receiver.unwrap();
    assert_eq!(receiver, references[0].receiver().storage_name());
    assert_eq!(args.len(), 3);
    let pin_index = statements
        .iter()
        .position(
            |statement| matches!(statement, StatementIr::Lexical { name, .. } if name == callee),
        )
        .unwrap();
    let template_name = identifier(&args[0]);
    let (template_index, template) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::TemplateObject(template),
                        ..
                    },
                ..
            } if name == template_name => Some((index, template)),
            _ => None,
        })
        .unwrap();
    let owner = function.template_source.as_ref().unwrap();
    assert_eq!(template.site_id.source(), owner.id());
    assert!(template.site_id.cache_slot() < owner.site_count());
    assert_eq!(template.raw, ["head", "", "tail"]);
    let first = identifier(&args[1]);
    let first_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. } if name == first && source == "first")).unwrap();
    let outer_yield = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::GeneratorYield {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .unwrap();
    assert!(
        pin_index < template_index && template_index < first_index && first_index < outer_yield
    );
    assert!(unconditional(&function.body.statements).iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::TemplateObject(_), .. }, .. } if name == template_name)),
        "template acquisition remains unconditional after the completed group");
    owned(
        &function,
        &[callee, receiver, template_name, first, identifier(&args[2])]
            .into_iter()
            .collect(),
    );
}

#[test]
fn grouped_terminal_property_constructor_keeps_its_existing_receiver_free_value_route() {
    let function = values("function* values(object, key, argument) { return new (object?.[yield key])(yield argument); }");
    assert_eq!(function.generator_plan.as_ref().unwrap().state_count, 6);
    let mut statements = Vec::new();
    all(&function.body.statements, &mut statements);
    assert!(
        captures(&statements).is_empty(),
        "Construct consumes GetValue without a call receiver"
    );
    let construct = projected(&statements)
        .into_iter()
        .find(|value| matches!(value.expr, ExprIr::Construct { .. }))
        .unwrap();
    let ExprIr::Construct { callee, args, .. } = &construct.expr else {
        unreachable!()
    };
    let callee = identifier(callee);
    let pin = statements
        .iter()
        .position(
            |statement| matches!(statement, StatementIr::Lexical { name, .. } if name == callee),
        )
        .unwrap();
    let outer_yield = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::GeneratorYield {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .unwrap();
    assert!(pin < outer_yield);
    assert_eq!(args.len(), 1);
    owned(
        &function,
        &[callee, identifier(&args[0])].into_iter().collect(),
    );
}

#[test]
fn checked_chain_values_compose_through_actual_declaration_discard_and_invocation_routes() {
    for source in [
        "function* values(object) { const result = object?.value[yield 1]; return result; }",
        "function* values(object) { var result = object?.[yield 1][yield 2]; return result; }",
        "function* values(object) { object?.method(yield 1); }",
        "function* values(object) { return object?.[yield 1](); }",
        "function* values(object) { return object?.method(yield 1, yield 2); }",
        "function* values(object) { return object?.method(...(yield 1), yield 2); }",
        "function* values(object, consume) { return consume(object?.method(yield 1)); }",
        "function* values(object, tag) { return tag`${object?.method(yield 1)}`; }",
        "function* values(object) { return (object?.method(yield 1)).value; }",
        "function* values(object) { try { const result = object?.method(yield 1); return result; } catch (error) { return error; } finally { 0; } }",
        "function* values(argument) { return eval?.(42, yield argument); }",
        "function* values(object) { return (object?.method(yield 1))(2); }",
        "function* values(object) { return (object?.[yield 1])(2); }",
        "function* values(object) { return (object?.[yield 1])`tag`; }",
        "function* values(object) { return (object?.method(yield 1)?.[yield 2])(yield 3); }",
        "class C { #method; *values(object) { return object.#method?.(yield 1); } }",
        "class C { #method; *values(object) { return ((object.#method))?.(yield 1); } }",
        "class C { #method; *values(object) { return (object.#method?.(yield 1))(yield 2); } }",
        "class C { #method; *values(object) { return (object.#method?.(yield 1).method)(yield 2); } }",
        "class C { #value; *values(object) { return object.#value?.method(yield 1); } }",
        "class C { #value; *values(object) { return ((object.#value))?.method(yield 1); } }",
        "class C { #value; *values(object) { return object.#value?.[yield 1]; } }",
        "class C { #value; *values(object) { return (object.#value?.[yield 1])(yield 2); } }",
    ] { values(source); }
}

#[test]
fn unowned_operand_branches_protocols_references_and_iterator_regions_stay_refused() {
    // The original ordered source cohort retains explicit acceptance and
    // refusal obligations as complete optional regions acquire real consumers.
    for (source, supported) in [
        (
            "function* values(object) { return (yield object)?.method(); }",
            true,
        ),
        (
            "function* values(object) { return (yield object)?.method(yield 1); }",
            true,
        ),
        (
            "function* values(object) { return object?.method((yield 1) + (yield 2)); }",
            true,
        ),
        (
            "function* values(object, flag) { return object?.method(flag ? (yield 1) : 0); }",
            true,
        ),
        (
            "function* values(object) { return object?.[object?.[yield 1]]; }",
            true,
        ),
        (
            "function* values(object) { return object?.method(yield* [1]); }",
            true,
        ),
        (
            "function* values(object) { return object?.method(yield (yield 1)); }",
            true,
        ),
        (
            "function* values(object) { return delete object?.method(yield 1); }",
            true,
        ),
        (
            "function* values(object) { return (object?.method(yield 1))?.(yield 2); }",
            true,
        ),
        (
            "function* values(object) { return ((yield object)?.method())(2); }",
            true,
        ),
        (
            "function* values(object) { while (true) { return (object?.method(yield 1))(2); } }",
            true,
        ),
        (
            "async function* values(object) { return (object?.method(yield await 1))(2); }",
            true,
        ),
        (
            "function* values(object, flag) { return flag ? (object?.method(yield 1))(2) : 0; }",
            true,
        ),
        (
            "function* values(object, flag) { return flag && (object?.method(yield 1))(2); }",
            true,
        ),
        (
            "function* values(object, other) { return other?.((object?.method(yield 1))(2)); }",
            true,
        ),
        (
            "class C { #method; *values(object) { return object?.#method(yield 1); } }",
            true,
        ),
        (
            "class C extends Object { *values() { return super.method?.(yield 1); } }",
            true,
        ),
        (
            "function* values(object) { if (true) { const value = object?.method(yield 1); } }",
            true,
        ),
        (
            "function* values(object) { while (true) { object?.method(yield 1); break; } }",
            true,
        ),
        (
            "function* values(object) { for (; true; ) { object?.method(yield 1); break; } }",
            true,
        ),
        (
            "function* values(object) { do { object?.method(yield 1); } while (false); }",
            true,
        ),
        (
            "async function* values(object) { return object?.method(yield await 1); }",
            true,
        ),
    ] {
        if supported {
            values(source);
            continue;
        }
        let parsed = parse(source, ParseOptions::script()).expect("valid refused generator source");
        let program = lower(&parsed);
        assert!(
            !program.is_wasm_supported(),
            "incorrectly admitted {source}"
        );
        assert!(program.wasm_blocking_diagnostic().is_some());
        assert!(program.script.as_ref().expect("refused Script IR").functions.iter().all(|function| function.generator_plan.is_none()),
            "an unowned continuation source must not publish a complete ordinary generator plan: {source}");
    }
}
