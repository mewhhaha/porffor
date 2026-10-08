use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, StatementIr};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("generator Throw source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .unwrap()
}

#[test]
fn complete_throw_operand_publishes_only_after_all_source_suspensions() {
    let function = values("function* values() { throw (yield 'first', yield 'second'); }");
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 3);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        vec![(0, 1), (1, 2)]
    );
    let [StatementIr::LexicalBlock(items)] = function.body.statements.as_slice() else {
        panic!("the whole Throw operand must retain one staged item")
    };
    assert!(matches!(items.last(), Some(StatementIr::Throw(_))));
    assert_eq!(
        items
            .iter()
            .filter(|item| matches!(item, StatementIr::Throw(_)))
            .count(),
        1
    );
    assert_eq!(
        items
            .iter()
            .filter(|item| matches!(item, StatementIr::GeneratorYield { .. }))
            .count(),
        2
    );
}

#[test]
fn throwing_complete_values_uses_existing_function_and_region_owners() {
    for source in [
        "function* values() { throw yield 1; }",
        "function* values(target) { throw target.method(yield 1, yield 2); }",
        "function* values(target) { throw (yield 1) ? (yield 2, yield 3) : yield* target; }",
        "function* values(target) { throw (yield 1)?.[yield 2](yield 3); }",
        "function* values() { try { throw (yield 1, yield 2); } finally { yield 3; } }",
        "function* values() { for (var i = 0; i < 2; i++) { throw yield i; } }",
        "function* values() { switch (yield 1) { case 1: throw (yield 2, yield 3); } }",
        "function* values(items) { for (var item of items) { throw yield item; } }",
        "function* values(scope) { with (scope) throw (yield 1, value); }",
    ] {
        let function = values(source);
        assert!(!function
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .is_empty());
    }
}

#[test]
fn throw_staging_keeps_pattern_and_foreign_region_boundaries_explicit() {
    for (index, (source, supported)) in [
        ("function* values() { var x; throw ({x: x = yield 1} = {}); }", true),
        ("function* values(items) { for (var item of items) { throw (yield 1) ? yield 2 : yield 3; } }", true),
        ("function* values(scope) { with (scope) { switch (yield 1) { case 1: throw yield 2; } } }", true),
    ].into_iter().enumerate() {
        if supported {
            let function = values(source);
            if index == 1 {
                let iterator = function.body.statements.iter().find_map(|statement| match statement {
                    StatementIr::AsyncGeneratorForOf(plan) => Some(plan),
                    _ => None,
                }).expect("the formerly foreign Throw now has the complete iterator owner");
                assert!(iterator.initialization().end_state() < iterator.body().entry_state());
                assert_eq!(function.generator_plan.as_ref().unwrap().suspension_points.len(), 3);
            }
            continue;
        }
        let parsed = parse(source, ParseOptions::script()).expect("unsupported Throw source parses");
        let program = lower(&parsed);
        assert!(!program.is_wasm_supported(), "unowned source must remain explicit: {source}");
    }
}
