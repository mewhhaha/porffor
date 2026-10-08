use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{lower, AsyncGeneratorForOfIr, FunctionIr, LexicalEnvironmentIr, StatementIr};

fn lower_stream(source: &str) -> FunctionIr {
    let unit = parse(source, ParseOptions::script()).expect("regression source must parse");
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR must exist")
        .functions
        .into_iter()
        .find(|function| function.name == "stream")
        .expect("stream must be lowered")
}

fn loop_plan(items: &[StatementIr]) -> Option<&AsyncGeneratorForOfIr> {
    items.iter().find_map(|statement| match statement {
        StatementIr::AsyncGeneratorForOf(plan) => Some(plan.as_ref()),
        StatementIr::Block(block) => loop_plan(&block.statements),
        StatementIr::LexicalBlock(items) => loop_plan(items),
        _ => None,
    })
}

fn loop_binding(function: &FunctionIr) -> (&str, Option<&LexicalEnvironmentIr>) {
    let plan = loop_plan(&function.body.statements).expect("a planned for-await loop must exist");
    let name = match plan.initialization().block().statements.as_slice() {
        [StatementIr::Lexical { name, .. }] => name.as_str(),
        [StatementIr::Var(declarations)] if declarations.len() == 1 => {
            declarations[0].name.as_str()
        }
        _ => panic!("the original head declaration remains explicit"),
    };
    (
        name,
        plan.lexical_environment()
            .and_then(|environment| environment.iteration_environment.as_ref()),
    )
}

#[test]
fn uncaptured_for_await_heads_have_unique_activation_slots() {
    for mode in ["let", "const", "var"] {
        let function = lower_stream(&format!(
            "async function* stream(source) {{ for await ({mode} value of source) {{ yield value * 2; yield value + 1; }} }}"
        ));
        let (name, iteration_environment) = loop_binding(&function);
        let plan = loop_plan(&function.body.statements).unwrap();
        if mode == "var" {
            assert!(iteration_environment.is_none());
            assert_eq!(
                function
                    .owned_env_bindings
                    .iter()
                    .filter(|binding| binding.name == name)
                    .count(),
                1
            );
        } else {
            let environment = iteration_environment
                .expect("the complete lexical head retains its original iteration record");
            assert_eq!(
                environment
                    .bindings
                    .iter()
                    .filter(|binding| binding.name == name)
                    .count(),
                1
            );
            assert!(!function
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == name));
        }
        for binding in [
            plan.head_binding(),
            plan.incoming_binding(),
            plan.value_binding(),
        ] {
            assert!(function.owned_env_bindings.contains(binding));
            assert_ne!(binding.name, name);
        }
        let slots = function
            .owned_env_bindings
            .iter()
            .map(|binding| binding.slot)
            .collect::<BTreeSet<_>>();
        assert_eq!(slots.len(), function.owned_env_bindings.len());
    }
}

#[test]
fn a_shadowing_head_does_not_share_the_outer_activation_slot() {
    let function = lower_stream(
        "async function* stream(source) { let value = 99; for await (const value of source) { yield value; yield value + 1; } yield value; }",
    );
    let (name, environment) = loop_binding(&function);
    let head = environment
        .expect("the lexical head has its original iteration record")
        .bindings
        .iter()
        .find(|binding| binding.name == name)
        .expect("head must survive suspension");
    let StatementIr::Lexical {
        name: outer_name, ..
    } = &function.body.statements[0]
    else {
        panic!("outer declaration must remain explicit");
    };
    let outer = function
        .owned_env_bindings
        .iter()
        .find(|binding| &binding.name == outer_name)
        .expect("outer must survive suspension");
    assert_ne!(head.name, outer.name);
    assert!(!function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == head.name));
    let plan = loop_plan(&function.body.statements).unwrap();
    for binding in [
        plan.head_binding(),
        plan.incoming_binding(),
        plan.value_binding(),
    ] {
        assert!(function.owned_env_bindings.contains(binding));
        assert_ne!(binding.slot, outer.slot);
    }
}

#[test]
fn captured_heads_keep_their_single_per_iteration_cell() {
    let function = lower_stream(
        "async function* stream(source) { for await (const value of source) { const read = () => value; yield read(); } }",
    );
    let (name, iteration_environment) = loop_binding(&function);
    let environment =
        iteration_environment.expect("captured head must have a per-iteration environment");
    assert_eq!(
        environment
            .bindings
            .iter()
            .filter(|binding| binding.name == name)
            .count(),
        1
    );
    assert!(!function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == name));
}
