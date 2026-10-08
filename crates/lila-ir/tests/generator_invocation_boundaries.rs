use lila_front::{parse, ParseOptions};
use lila_ir::lower;

#[test]
fn invocation_expansion_keeps_mixed_and_unowned_conditional_yields_explicitly_unsupported() {
    for source in [
        "async function* values() { return (object?.[yield await 1])(yield 2); }",
        "async function* values() { return call(await 1, yield 2); }",
        "async function* values() { return new Constructor(await 1, yield 2); }",
        "async function* values() { return tag`${await 1}${yield 2}`; }",
        "async function* values() { yield call(await 1, yield 2); }",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let function = program
            .script
            .unwrap()
            .functions
            .into_iter()
            .find(|function| function.name == "values")
            .unwrap();
        assert!(function.resumable_plan.is_some());
    }
    for (source, yields) in [
        ("function* values() { while (true) { return (object?.[yield 1])(yield 2); } }", 2),
        ("function* values() { return call(...((yield 1) ? yield 2 : yield 3)); }", 3),
        ("function* values() { return new Constructor((yield 1) ? yield 2 : yield 3); }", 3),
        ("function* values() { return tag`${(yield 1) ? yield 2 : yield 3}`; }", 3),
        ("function* values() { return call(yield 1, {value: yield 2, __proto__: null}); }", 2),
        ("function* values() { return new Constructor(yield 1, {value: yield 2, '__proto__': null}); }", 2),
        ("function* values() { return tag`${yield 1}${{value: yield 2, __proto__: null}}`; }", 2),
    ] {
        let unit = parse(source, ParseOptions::script()).expect("boundary source must parse");
        let program = lower(&unit);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let function = program.script.unwrap().functions.into_iter()
            .find(|function| function.name == "values").unwrap();
        let plan = function.generator_plan.as_ref().expect("checked ordinary source tape");
        assert_eq!(plan.suspension_points.len(), yields, "{source}");
        let resumes = plan.suspension_points.iter().map(|point| {
            assert!(point.suspend_state < point.resume_state);
            assert!(point.resume_state < plan.state_count);
            point.resume_state
        }).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(resumes.len(), yields);
    }
}
