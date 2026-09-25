use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, StatementIr};

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
    statements
        .iter()
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
fn branch_declaration_initializer_yield_remains_unplanned() {
    let source = "function* values(flag) { if (flag) { const value = yield 1; yield value; } }";
    let unit = parse(source, ParseOptions::script()).expect("generator source must parse");
    let program = lower(&unit);
    assert!(!program.is_wasm_supported(), "{source}");
    assert!(
        program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains(
                "a nested declaration contains a yield, which has no linear suspension plan"
            )),
        "{source}: {:?}",
        program.diagnostics
    );
}

#[test]
fn branch_declarations_with_nested_yields_have_structured_continuations() {
    for (source, expected_yields) in [
        (
            "function* values(flag) { if (flag) { let value = 1; { yield value; } } }",
            1,
        ),
        (
            "function* values(flag) { if (flag) { let value = 1; yield value; yield 2; } }",
            2,
        ),
        (
            "function* values(flag) { if (flag) { let value = 1; const read = () => value; yield read(); } }",
            1,
        ),
        (
            "function* values(flag) { if (flag) { class Value {} yield Value; } }",
            1,
        ),
    ] {
        let function = lower_values(source);
        let plan = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::GeneratorStructuredIf { plan, .. } => Some(plan),
                _ => None,
            })
            .expect("nested branch yields must have a checked continuation");
        let generator = function.generator_plan.as_ref().expect("generator plan");
        assert_eq!(generator.suspension_points.len(), expected_yields, "{source}");
        assert_eq!(plan.entry_state(), generator.entry_state, "{source}");
        assert_eq!(plan.exit_state() + 1, generator.state_count, "{source}");
        assert!(plan.then_entry_state() < plan.then_exit_state(), "{source}");
        assert!(matches!(plan.then_branch(), StatementIr::Block(_)), "{source}");
    }
}
