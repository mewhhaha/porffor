use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, ResumableClassDefinitionIr, StatementIr};

fn lower_classes(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("class fixture parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "classes")
        .expect("generator function")
}

fn collect_plans<'a>(
    statements: &'a [StatementIr],
    plans: &mut Vec<&'a ResumableClassDefinitionIr>,
) {
    for statement in statements {
        match statement {
            StatementIr::ResumableClassDefinition(plan) => {
                plans.push(plan);
                for prefix in plan.prefixes() {
                    collect_plans(prefix.statements(), plans);
                }
            }
            StatementIr::LexicalBlock(statements) => collect_plans(statements, plans),
            StatementIr::Block(block) => collect_plans(&block.statements, plans),
            _ => {}
        }
    }
}

#[test]
fn class_prefixes_keep_heritage_and_keys_in_one_ordered_activation_owner() {
    let function = lower_classes("function* classes() { let C = class Named extends (yield 1) { [yield 2]() {} [yield 3] = 4; }; return C; }");
    let mut plans = Vec::new();
    collect_plans(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 1);
    let plan = plans[0];
    assert_eq!((plan.entry_state(), plan.exit_state()), (0, 3));
    assert_eq!(
        (
            plan.heritage_prefix().entry_state(),
            plan.heritage_prefix().exit_state()
        ),
        (0, 1)
    );
    assert_eq!(plan.class().element_plan.definitions.len(), 2);
    for index in 0..2 {
        let prefix = plan
            .element_prefix(index)
            .expect("each computed key owns its suspension");
        assert_eq!(
            (prefix.entry_state(), prefix.exit_state()),
            (index as u32 + 1, index as u32 + 2)
        );
    }
    let names = [
        plan.constructor_binding(),
        plan.completion_binding(),
        plan.name_environment_binding()
            .expect("named class environment"),
    ];
    let slots = names.map(|name| {
        let owned = function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        assert_eq!(owned.len(), 1, "{name}");
        owned[0].slot
    });
    assert_eq!(slots.into_iter().collect::<BTreeSet<_>>().len(), 3);
}

#[test]
fn class_declarations_use_the_same_resumable_definition_contract() {
    let function = lower_classes("function* classes() { class Named { get [yield 1]() { return 2; } static [yield 3] = 4; } return Named; }");
    let mut plans = Vec::new();
    collect_plans(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 1);
    assert_eq!((plans[0].entry_state(), plans[0].exit_state()), (0, 2));
    assert!(plans[0].name_environment_binding().is_some());
    assert_eq!(plans[0].class().element_plan.static_elements.len(), 1);
}

#[test]
fn nested_class_prefixes_retain_distinct_constructor_and_environment_slots() {
    let function = lower_classes("function* classes() { let C = class Outer { [yield (class Inner { [yield 1]() {} })]() {} }; return C; }");
    let mut plans = Vec::new();
    collect_plans(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 2);
    assert_eq!((plans[0].entry_state(), plans[0].exit_state()), (0, 2));
    assert_eq!((plans[1].entry_state(), plans[1].exit_state()), (0, 1));
    assert_ne!(
        plans[0].constructor_binding(),
        plans[1].constructor_binding()
    );
    assert_ne!(
        plans[0].name_environment_binding(),
        plans[1].name_environment_binding()
    );
    assert!(plans.iter().all(|plan| function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == plan.constructor_binding())));
}
