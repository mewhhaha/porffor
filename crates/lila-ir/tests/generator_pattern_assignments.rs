use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ArrayDestructuringEvaluationIr, BindingMode, DestructuringTargetIr, ExprIr, FunctionIr,
    GeneratorResumeModeIr, OrdinaryGeneratorLoopIr, StatementIr, TypedExpr, ValueKind,
};
use std::collections::BTreeSet;

fn assignment_function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("assignment pattern parses");
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
        .find(|function| function.name == "values")
        .expect("ordinary generator")
}

fn assignment_rows<'a>(source: &'a [StatementIr], rows: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::LexicalBlock(body) => assignment_rows(body, rows),
            StatementIr::Block(body) => assignment_rows(&body.statements, rows),
            statement => rows.push(statement),
        }
    }
}

#[test]
fn resumed_assignment_pattern_consumes_the_received_value_and_real_property_target() {
    let function = assignment_function(
        "function* values(object) { return ([object.value] = yield 'source'); }",
    );
    let mut rows = Vec::new();
    assignment_rows(&function.body.statements, &mut rows);
    let (yield_index, received) = rows
        .iter()
        .enumerate()
        .find_map(|(index, row)| {
            let StatementIr::GeneratorYield {
                resume_mode: GeneratorResumeModeIr::AssignIdentifier(name),
                ..
            } = row
            else {
                return None;
            };
            Some((index, name))
        })
        .expect("received whole RHS");
    let (assignment_index, value, pattern) = rows
        .iter()
        .enumerate()
        .find_map(|(index, row)| {
            let StatementIr::Return(TypedExpr {
                expr:
                    ExprIr::ArrayDestructure {
                        value,
                        pattern,
                        evaluation: ArrayDestructuringEvaluationIr::AssignmentEvaluation,
                    },
                ..
            }) = row
            else {
                return None;
            };
            Some((index, value, pattern))
        })
        .expect("actual destructuring assignment semantic consumer");
    assert!(yield_index < assignment_index);
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == received));
    assert!(matches!(
        &pattern.elements[0],
        lila_ir::ArrayDestructuringElementIr::Target {
            target: DestructuringTargetIr::AssignmentProperty { .. },
            ..
        }
    ));
    let StatementIr::Return(result) = rows[assignment_index] else {
        unreachable!()
    };
    assert!(
        result.heap_shape.is_none(),
        "target effects may mutate the returned RHS"
    );

    let function = assignment_function(
        "function* values() { let rest; return ({...rest} = (yield 1, function target() {})); }",
    );
    rows.clear();
    assignment_rows(&function.body.statements, &mut rows);
    let result = rows
        .iter()
        .find_map(|row| match row {
            StatementIr::Return(result) => Some(result),
            _ => None,
        })
        .expect("whole function RHS result");
    let ExprIr::ObjectDestructure { value, .. } = &result.expr else {
        panic!("real object assignment consumer");
    };
    assert_eq!(result.kind, ValueKind::Function);
    assert_eq!(result.function_targets, value.function_targets);
    assert!(result.function_targets.exact_single_target().is_some());
    assert!(result.heap_shape.is_none());
}

fn classic_loop(function: &FunctionIr) -> &OrdinaryGeneratorLoopIr {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorLoop(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual checked classic For owner")
}

fn initialization_bindings(plan: &OrdinaryGeneratorLoopIr) -> Vec<(BindingMode, String)> {
    let mut rows = Vec::new();
    assignment_rows(
        &plan
            .initialization()
            .expect("For initialization")
            .block()
            .statements,
        &mut rows,
    );
    let mut bindings = Vec::new();
    for row in rows {
        let StatementIr::DeclarationEvaluation(TypedExpr { expr, .. }) = row else {
            continue;
        };
        let mut visit = |mode, name: &str| bindings.push((mode, name.to_string()));
        match expr {
            ExprIr::ArrayDestructure {
                pattern,
                evaluation,
                ..
            } => {
                assert_eq!(
                    *evaluation,
                    ArrayDestructuringEvaluationIr::BindingInitialization
                );
                pattern.visit_bindings(&mut visit);
            }
            ExprIr::ObjectDestructure { pattern, .. } => pattern.visit_bindings(&mut visit),
            _ => {}
        }
    }
    bindings
}

#[test]
fn classic_for_pattern_initialization_owns_exact_head_cells_and_mutable_iteration_slots() {
    for source in [
        "function* values(source) { for (let {i, box: {step}} = yield source; i < 2; i++) { yield () => [i, step]; } }",
        "function* values(source) { for (let [i, step] = (yield 1, yield source); i < 2; i++) { yield () => [i, step]; } }",
        "function* values(flag, source) { for (let [i, step] = flag ? (yield 1, yield source) : yield source; i < 2; i++) { yield () => [i, step]; } }",
        "function* values(source) { for (let [i, step] = source; i < 2; i++) { yield () => [i, step]; } }",
    ] {
        let function = assignment_function(source);
        let plan = classic_loop(&function);
        let bindings = initialization_bindings(plan);
        assert_eq!(bindings.len(), 2);
        assert!(bindings.iter().all(|(mode, _)| *mode == BindingMode::Let));
        let environment = plan.lexical_environment().expect("captured head Environment Record");
        let slots = bindings.iter().map(|(_, name)| {
            environment.bindings.iter().find(|binding| &binding.name == name)
                .expect("pattern publication and capture analysis use one storage name").slot
        }).collect::<BTreeSet<_>>();
        assert_eq!(environment.per_iteration_slots.iter().copied().collect::<BTreeSet<_>>(), slots);
        let initialization = plan.initialization().unwrap();
        assert_eq!(initialization.end_state().checked_add(1), Some(plan.test().region().entry_state()));
    }

    let function = assignment_function(
        "function* values(source) { for (const {i} = yield source; i; ) { yield () => i; break; } }",
    );
    let plan = classic_loop(&function);
    assert_eq!(initialization_bindings(plan)[0].0, BindingMode::Const);
    assert!(plan
        .lexical_environment()
        .expect("captured const head")
        .per_iteration_slots
        .is_empty());

    let function = assignment_function(
        "function* values(source) { for (let i = 0, {step} = yield source; i < 2; i++) { yield () => [i, step]; } }",
    );
    let plan = classic_loop(&function);
    assert_eq!(initialization_bindings(plan).len(), 1);
    assert_eq!(
        plan.lexical_environment()
            .expect("mixed captured head")
            .per_iteration_slots
            .len(),
        2
    );

    let function = assignment_function(
        "function* values(source) { for (let [i] = yield source; yield i; i++) { yield i; } }",
    );
    let plan = classic_loop(&function);
    for (_, name) in initialization_bindings(plan) {
        assert!(
            function
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == name),
            "uncaptured source head binding survives in the actual activation"
        );
    }
    for source in [
        "function* values(source) { for (let [i = yield 1] = yield source; i; ) { yield i; } }",
        "function* values(source) { for (let [i = yield 1] = source, later = yield source; i; ) { yield i; } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("unowned head pattern parses");
        assert!(lower(&parsed).is_wasm_supported(), "pattern-owned head continuation: {source}");
    }
}

#[test]
fn complete_pattern_assignment_rhs_regions_do_not_admit_pattern_owned_suspensions() {
    for source in [
        "function* values(source) { let first; return ([first] = (yield 1, yield source)); }",
        "function* values(flag, source) { let selected; return ({selected} = flag ? (yield 1, yield source) : yield source); }",
        "function* values(source, object, key) { return ({[key()]: object.value = fallback(), ...object.rest} = yield source); }",
        "function* values(source, object) { [object.value] = yield source; return object.value; }",
        "function* values(object) { [object.value] = `${yield 'part'}`; return object.value; }",
        "function* values(source) { let first; return ([first] = yield* source); }",
    ] {
        let function = assignment_function(source);
        assert!(!function.generator_plan.as_ref().expect("source plan").suspension_points.is_empty());
    }
    for (source, supported) in [
        (
            "function* values(source) { let first; ([first = yield 1] = yield source); }",
            true,
        ),
        (
            "function* values(source, object) { ({[yield 1]: object.value} = yield source); }",
            true,
        ),
        (
            "function* values(source, object) { ([object[yield 1]] = yield source); }",
            true,
        ),
    ] {
        if supported {
            assignment_function(source);
            continue;
        }
        let parsed = parse(source, ParseOptions::script()).expect("unowned pattern parses");
        let program = lower(&parsed);
        assert!(
            !program.is_wasm_supported(),
            "unowned pattern continuation: {source}"
        );
    }
}
