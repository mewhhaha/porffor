use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ArrayDestructuringEvaluationIr, ExprIr, FunctionIr, GeneratorResumeModeIr, StatementIr,
    TypedExpr,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("pattern initializer parses");
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
        .expect("generator owner")
}

fn flatten<'a>(source: &'a [StatementIr], statements: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::LexicalBlock(body) => flatten(body, statements),
            StatementIr::Block(body) => flatten(&body.statements, statements),
            statement => statements.push(statement),
        }
    }
}

#[test]
fn binding_initialization_consumes_each_received_value_after_its_normal_resume() {
    let function = values(
        "function* values(source) { let [first = fallback()] = yield source; const {selected = first, ...rest} = yield source; return [first, selected, rest]; }",
    );
    let plan = function
        .generator_plan
        .as_ref()
        .expect("actual source plan");
    assert_eq!(plan.suspension_points.len(), 2);
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let received = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::GeneratorYield {
                resume_mode: GeneratorResumeModeIr::AssignIdentifier(name),
                ..
            } => Some((index, name)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(received.len(), 2);
    let mut initializations = Vec::new();
    for (index, statement) in statements.iter().enumerate() {
        let StatementIr::DeclarationEvaluation(TypedExpr { expr, .. }) = statement else {
            continue;
        };
        let (value, bindings) = match expr {
            ExprIr::ArrayDestructure {
                value,
                pattern,
                evaluation,
            } => {
                assert_eq!(
                    *evaluation,
                    ArrayDestructuringEvaluationIr::BindingInitialization
                );
                let mut bindings = Vec::new();
                pattern.visit_bindings(&mut |_, name| bindings.push(name.to_owned()));
                (value, bindings)
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                assert_eq!(pattern.properties.len(), 1, "one Get owns the default");
                assert!(pattern.properties[0].default.is_some());
                let mut bindings = Vec::new();
                pattern.visit_bindings(&mut |_, name| bindings.push(name.to_owned()));
                (value, bindings)
            }
            _ => continue,
        };
        let ExprIr::Identifier(name) = &value.expr else {
            panic!("received value is retained once")
        };
        for binding in bindings {
            assert!(
                function
                    .owned_env_bindings
                    .iter()
                    .any(|owned| owned.name == binding),
                "source binding {binding} belongs to the actual activation"
            );
        }
        initializations.push((index, name));
    }
    assert_eq!(initializations.len(), 2);
    for ((yield_index, received_name), (binding_index, value_name)) in
        received.iter().zip(&initializations)
    {
        assert!(
            yield_index < binding_index,
            "no binding publication before Normal resume"
        );
        assert_eq!(received_name, value_name);
        assert_eq!(
            function
                .owned_env_bindings
                .iter()
                .filter(|binding| &binding.name == *received_name)
                .count(),
            1
        );
    }
    assert!(
        initializations[0].0 < received[1].0,
        "first pattern completes before next initializer"
    );
}

#[test]
fn complete_initializer_regions_compose_without_admitting_suspensions_inside_patterns() {
    for source in [
        "function* values(source) { let [first] = (yield 1, yield source); return first; }",
        "function* values(flag, source) { const {selected} = flag ? (yield 1, yield source) : (yield source); return selected; }",
        "function* values(source) { var [first, ...rest] = yield source; return rest; }",
        "function* values(source) { let [first] = source, {selected} = yield source; return selected; }",
        "function* values(source, key) { const {[key()]: selected = 7, ...rest} = yield source; return rest; }",
        "function* values(source) { const [] = yield source; return 1; }",
    ] {
        let function = values(source);
        assert!(
            !function
                .generator_plan
                .as_ref()
                .expect("plan")
                .suspension_points
                .is_empty()
        );
    }
    for (source, supported) in [
        (
            "function* values(source) { let [first = (yield 1)] = yield source; }",
            true,
        ),
        (
            "function* values(source) { const {[(yield 1)]: selected} = yield source; }",
            true,
        ),
        (
            "function* values(source) { let [first = (yield 1)] = source, second = yield source; }",
            true,
        ),
    ] {
        if supported {
            values(source);
            continue;
        }
        let parsed = parse(source, ParseOptions::script()).expect("unowned pattern source parses");
        let program = lower(&parsed);
        assert!(
            !program.is_wasm_supported(),
            "unowned pattern continuation admitted: {source}"
        );
    }
}

#[test]
fn suspended_var_patterns_publish_all_names_to_variable_instantiation() {
    for (source, expected_names) in [
        (
            "function* values(source) { var {first = yield 1, nested: {second}, ...rest} = source; return [first, second, rest]; }",
            vec!["first", "second", "rest"],
        ),
        (
            "function* values(source) { var [first = yield 1, {second}, ...rest] = source; return [first, second, rest]; }",
            vec!["first", "second", "rest"],
        ),
        (
            "function* values(source) { var {first, nested: [second], ...rest} = yield source; return [first, second, rest]; }",
            vec!["first", "second", "rest"],
        ),
        (
            "function* values(source) { var [first, {second}, ...rest] = yield source; return [first, second, rest]; }",
            vec!["first", "second", "rest"],
        ),
    ] {
        let function = values(source);
        let mut statements = Vec::new();
        flatten(&function.body.statements, &mut statements);
        let declared_names = statements
            .iter()
            .filter_map(|statement| match statement {
                StatementIr::Var(declarations) => Some(declarations),
                _ => None,
            })
            .flatten()
            .map(|declaration| declaration.name.as_str())
            .collect::<Vec<_>>();
        for name in expected_names {
            assert!(
                declared_names.contains(&name),
                "{name} must start initialized to undefined before any pattern suspension: {source}",
            );
        }
    }
}
