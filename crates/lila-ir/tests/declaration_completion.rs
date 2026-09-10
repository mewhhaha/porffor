use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ArrayDestructuringEvaluationIr, EnvironmentIdentifierOperationIr, ExprIr,
    PreparedScriptKind, StatementIr, TypedExpr,
};

fn declaration_evaluations<'a>(statements: &'a [StatementIr], effects: &mut Vec<&'a TypedExpr>) {
    for statement in statements {
        match statement {
            StatementIr::DeclarationEvaluation(expression) => effects.push(expression),
            StatementIr::LexicalBlock(statements) => declaration_evaluations(statements, effects),
            StatementIr::Block(block) => declaration_evaluations(&block.statements, effects),
            _ => {}
        }
    }
}

#[test]
fn borrowed_eval_var_write_retains_declaration_completion_and_runtime_reference() {
    let program = lower(
        &parse(
            "eval('23; var value = 123; value = 456;');",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script
        .prepared_script_units()
        .find(|unit| matches!(unit.kind, PreparedScriptKind::DirectEval(_)))
        .expect("prepared direct eval unit");
    assert_eq!(unit.declarations.var_names, ["value"]);
    let [StatementIr::Expression(_), StatementIr::DeclarationEvaluation(initializer), StatementIr::Expression(assignment)] =
        unit.body.statements.as_slice()
    else {
        panic!(
            "declarations and assignments have distinct completions: {:?}",
            unit.body
        );
    };
    for (expression, expected) in [(initializer, 123f64), (assignment, 456f64)] {
        let ExprIr::EnvironmentIdentifier(reference) = &expression.expr else {
            panic!("the runtime environment Reference must remain authoritative");
        };
        assert_eq!(reference.name, "value");
        let EnvironmentIdentifierOperationIr::Assign { value } = &reference.operation else {
            panic!("initializer must still write its value");
        };
        assert!(matches!(value.expr, ExprIr::Number(bits) if bits == expected.to_bits()));
    }
}

#[test]
fn declaration_patterns_preserve_empty_completion_for_every_prepared_eval_unit() {
    for declaration in [
        "var [selected] = [7];",
        "var {selected} = {selected: 7};",
        "var {} = {};",
        "var [] = [];",
        "let [selected] = [7];",
        "const {['selected']: selected} = {selected: 7};",
        "let {} = {};",
        "const [] = [];",
    ] {
        let source = format!("eval({:?});", format!("23; {declaration}"));
        let program = lower(&parse(&source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.unwrap();
        let units: Vec<_> = script.prepared_script_units().collect();
        assert_eq!(units.len(), 2, "both direct and indirect identity branches");
        for unit in units {
            let mut effects = Vec::new();
            declaration_evaluations(&unit.body.statements, &mut effects);
            // A simple object var pattern can also lower into ordinary Var
            // declarators, whose normal completion is already empty.
            let ordinary_var = matches!(unit.kind, PreparedScriptKind::IndirectEval)
                && declaration == "var {selected} = {selected: 7};";
            assert_eq!(
                effects.len(),
                usize::from(!ordinary_var),
                "{declaration}: {:?}",
                unit.body
            );
            for effect in effects {
                assert!(matches!(
                    effect.expr,
                    ExprIr::ArrayDestructure {
                        evaluation: ArrayDestructuringEvaluationIr::BindingInitialization,
                        ..
                    } | ExprIr::ObjectDestructure { .. }
                        | ExprIr::SpecOperation { .. }
                ));
            }
        }
    }
}

#[test]
fn array_assignment_keeps_its_value_producing_evaluation() {
    let program = lower(
        &parse(
            "function owner(source) { let [bound] = source; var [variable] = source; [bound] = source; }",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .unwrap();
    let mut declarations = Vec::new();
    declaration_evaluations(&owner.body.statements, &mut declarations);
    assert_eq!(declarations.len(), 2);
    assert!(declarations.iter().all(|expression| matches!(
        expression.expr,
        ExprIr::ArrayDestructure {
            evaluation: ArrayDestructuringEvaluationIr::BindingInitialization,
            ..
        }
    )));
    assert!(matches!(
        owner.body.statements.last(),
        Some(StatementIr::Expression(TypedExpr {
            expr: ExprIr::ArrayDestructure {
                evaluation: ArrayDestructuringEvaluationIr::AssignmentEvaluation,
                ..
            },
            ..
        }))
    ));
}
