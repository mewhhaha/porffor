#[test]
fn lowers_loop_ir() {
    let program = lower_script("let i = 0; while (i < 3) { i = i + 1; continue; } i;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(matches!(
        script.body.statements[1],
        StatementIr::While { .. }
    ));
    assert!(program.ir_summary().contains("whiles=1"));
    assert!(program.ir_summary().contains("continues=1"));
}

#[test]
fn lowers_array_spread_to_typed_accumulation_without_fabricated_shape() {
    let program = lower_script("let source = [17, NaN, 'tail']; let copy = [...source]; copy[1];");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: copy_init,
        ..
    } = &script.body.statements[1]
    else {
        panic!("expected spread copy declaration");
    };
    assert_eq!(name, "copy");
    assert!(copy_init.heap_shape.is_none());
    let ExprIr::ArrayAccumulation(accumulation) = &copy_init.expr else {
        panic!(
            "expected typed ArrayAccumulation, got {:#?}",
            copy_init.expr
        );
    };
    assert!(matches!(
        accumulation.target(),
        ArrayAccumulationTargetIr::Fresh
    ));
    let [ArrayAccumulationElementIr::Spread(spread)] = accumulation.elements() else {
        panic!("expected exactly one spread element");
    };
    assert_eq!(spread.protocol, ArraySpreadProtocol::ARRAY_ACCUMULATION);

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected spread copy element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn lowers_array_spread_with_unshaped_source_as_dynamic_elements() {
    let program =
        lower_script("let source = [].concat({ length: 1 }); let copy = [...source]; copy[0];");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: copy_init,
        ..
    } = &script.body.statements[1]
    else {
        panic!("expected spread copy declaration");
    };
    assert_eq!(name, "copy");
    assert!(
        copy_init.heap_shape.is_none(),
        "an unshaped spread input must not become an empty array shape"
    );

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected spread copy element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn array_spread_of_unshaped_source_does_not_index_pushes_from_zero() {
    // Anti-vacuity: an ArrayAccumulation containing a spread never claims
    // a static base length for a later push, even when the operand was
    // produced by an actual concat call.
    let program = lower_script(
        "let source = [].concat({ length: 1 }); let copy = [...source]; copy.push(9); copy[0];",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: copy_init,
        ..
    } = &script.body.statements[1]
    else {
        panic!("expected spread copy declaration");
    };
    assert_eq!(name, "copy");
    assert!(
        copy_init.heap_shape.is_none(),
        "an unshaped concat input must not become an empty array shape"
    );

    let StatementIr::Expression(read) = &script.body.statements[3] else {
        panic!(
            "expected spread copy element read, got {:?}",
            script.body.statements[3]
        );
    };
    assert_eq!(
        read.kind,
        ValueKind::Dynamic,
        "push into an unshaped copy must not type index 0 from a base length of 0"
    );
}

#[test]
fn array_spread_of_unknown_iterable_does_not_claim_empty_array_shape() {
    // `x` is an un-inferred parameter and may be any iterable. The typed
    // ArrayAccumulation keeps its result unshaped rather than fabricating
    // a zero-length ArrayShape.
    let program = lower_script("function f(x) { return [...x]; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "f")
        .expect("function should be lowered");
    let StatementIr::Return(expr) = &function.body.statements[0] else {
        panic!(
            "expected return statement, got {:?}",
            function.body.statements
        );
    };
    assert!(
        expr.heap_shape.is_none(),
        "array spread of an unknown iterable must not carry an element vector"
    );
}

#[test]
fn concat_discards_element_shape_after_array_length_write() {
    let program = lower_script(
        "let source = [17, NaN, 'tail']; source.length = 1; let copy = [].concat(source); copy[1];",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: copy_init,
        ..
    } = &script.body.statements[2]
    else {
        panic!("expected concat copy declaration");
    };
    assert_eq!(name, "copy");
    assert!(
        copy_init.heap_shape.is_none(),
        "a length-mutated source must not retain stale concat elements"
    );

    let StatementIr::Expression(read) = &script.body.statements[3] else {
        panic!("expected concat copy element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn concat_discards_element_shape_for_custom_spreadability() {
    let program = lower_script(
            "let source = [17]; source[Symbol.isConcatSpreadable] = false; let copy = [].concat(source); copy[0];",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: copy_init,
        ..
    } = &script.body.statements[2]
    else {
        panic!("expected concat copy declaration");
    };
    assert_eq!(name, "copy");
    assert!(
        copy_init.heap_shape.is_none(),
        "@@isConcatSpreadable must make concat element layout dynamic"
    );

    let StatementIr::Expression(read) = &script.body.statements[3] else {
        panic!("expected concat copy element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn concat_discards_element_shape_for_holey_arrays() {
    let program = lower_script("let source = [, 17]; let copy = [].concat(source); copy[0];");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        init: copy_init, ..
    } = &script.body.statements[1]
    else {
        panic!("expected concat copy declaration");
    };
    assert!(
        copy_init.heap_shape.is_none(),
        "a hole must not be confused with an explicit undefined element"
    );

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected concat copy element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn flat_does_not_reuse_unflattened_receiver_shape() {
    let program = lower_script("let nested = [[17]]; let flattened = nested.flat(); flattened[0];");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: flattened_init,
        ..
    } = &script.body.statements[1]
    else {
        panic!("expected flat result declaration");
    };
    assert_eq!(name, "flattened");
    assert!(
        flattened_init.heap_shape.is_none(),
        "flat must not report the receiver's nested element shape"
    );

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected flat result element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn flat_map_result_elements_remain_dynamic() {
    let program = lower_script(
            "let source = [1, 2]; let mapped = source.flatMap(function (value) { return [value, value]; }); mapped[0];",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical {
        name,
        init: mapped_init,
        ..
    } = &script.body.statements[1]
    else {
        panic!("expected flatMap result declaration");
    };
    assert_eq!(name, "mapped");
    assert!(
        mapped_init.heap_shape.is_none(),
        "flatMap output cardinality and element layout are runtime-dependent"
    );

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected flatMap result element read");
    };
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn species_capable_array_results_preserve_runtime_object_tags() {
    for (source, result_statement) in [
        ("let result = [1].flat();", 0),
        ("let result = [1].slice();", 0),
        (
            "let result = [1].flatMap(function (value) { return value; });",
            0,
        ),
        ("let source = []; let result = source.concat(1);", 1),
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[result_statement] else {
            panic!("expected result declaration for {source}");
        };
        assert_eq!(init.kind, ValueKind::Dynamic, "{source}");
        assert_eq!(init.possible_kinds, KindSet::all_runtime_tags(), "{source}");
        assert!(init.heap_shape.is_none(), "{source}");
    }
}

#[test]
fn concat_with_proven_array_layout_remains_runtime_dynamic() {
    let program = lower_script("let result = [1].concat([2]);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
        panic!("expected concat result declaration");
    };
    assert_eq!(init.kind, ValueKind::Dynamic);
    assert_eq!(init.possible_kinds, KindSet::all_runtime_tags());
    assert!(init.heap_shape.is_none());
}
