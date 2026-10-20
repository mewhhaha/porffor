#[test]
fn generator_eager_operator_values_share_the_complete_source_suspension_plan() {
    for (expression, expected_yields) in [
        ("(yield 1) + (yield 2)", 2),
        ("(yield 1) - (yield 2)", 2),
        ("(yield 1) * (yield 2)", 2),
        ("(yield 1) / (yield 2)", 2),
        ("(yield 1) % (yield 2)", 2),
        ("(yield 1) ** (yield 2)", 2),
        ("(yield 1) < (yield 2)", 2),
        ("(yield 1) >= (yield 2)", 2),
        ("(yield 1) == (yield 2)", 2),
        ("(yield 1) !== (yield 2)", 2),
        ("(yield 1) in (yield 2)", 2),
        ("(yield 1) instanceof (yield 2)", 2),
        ("(yield 1) & (yield 2)", 2),
        ("(yield 1) | (yield 2)", 2),
        ("(yield 1) ^ (yield 2)", 2),
        ("(yield 1) << (yield 2)", 2),
        ("(yield 1) >> (yield 2)", 2),
        ("(yield 1) >>> (yield 2)", 2),
        ("((yield 1), (yield 2))", 2),
        ("+(yield 1)", 1),
        ("-(yield 1)", 1),
        ("~(yield 1)", 1),
        ("!(yield 1)", 1),
        ("typeof (yield 1)", 1),
        ("void (yield 1)", 1),
        ("delete (yield 1)", 1),
        ("delete (yield 1)[yield 2]", 2),
        ("delete ((object?.[yield 1]), 0)", 1),
        ("delete object?.[yield 1]", 1),
        ("delete (object?.[yield 1])", 1),
        ("delete (yield 1)?.name", 1),
        ("`A${yield 1}B${yield 2}C`", 2),
        ("`A${(yield 1) + (yield 2)}B`", 2),
    ] {
        let program = lower_script(&format!("function* value() {{ return {expression}; }}"));
        assert!(
            program.is_wasm_supported(),
            "{expression}: {:?}",
            program.diagnostics
        );
        let function = program
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .find(|f| f.name == "value")
            .unwrap();
        let plan = function.generator_plan.as_ref().unwrap();
        assert_eq!(
            plan.suspension_points.len(),
            expected_yields,
            "{expression}"
        );
        assert_eq!(
            plan.suspension_points
                .iter()
                .map(|point| point.resume_state)
                .collect::<BTreeSet<_>>()
                .len(),
            expected_yields,
            "each operand owns its resume: {expression}"
        );
    }
}
