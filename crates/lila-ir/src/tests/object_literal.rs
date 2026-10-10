#[test]
fn lowers_objects_arrays_and_properties() {
    let program = lower_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;");
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("objects=1"));
    assert!(summary.contains("arrays=1"));
    assert!(summary.contains("property_reads=1"));
    assert!(summary.contains("property_writes=1"));
}

#[test]
fn lowers_only_colon_proto_properties_as_prototype_setters() {
    fn object_literal(source: &str) -> TypedExpr {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script ir should exist");
        let StatementIr::Expression(expr) = script.body.statements.last().expect("expression")
        else {
            panic!("expected object literal expression");
        };
        expr.clone()
    }

    let prototype_setter = object_literal("({ __proto__: { marker: 1 } });");
    let ExprIr::ObjectLiteral(properties) = &prototype_setter.expr else {
        panic!("expected object literal");
    };
    assert!(matches!(
        properties.as_slice(),
        [ObjectPropertyIr::PrototypeSetter { .. }]
    ));
    let Some(HeapShape::Object(shape)) = prototype_setter.heap_shape.as_deref() else {
        panic!("expected object shape");
    };
    assert!(shape.prototype.is_some());
    assert!(!shape.properties.contains_key("__proto__"));

    let shorthand = object_literal("let __proto__ = 1; ({ __proto__ });");
    let ExprIr::ObjectLiteral(properties) = &shorthand.expr else {
        panic!("expected object literal");
    };
    assert!(matches!(
        properties.as_slice(),
        [ObjectPropertyIr::Data { .. }]
    ));

    let computed = object_literal("({ ['__proto__']: 1 });");
    let ExprIr::ObjectLiteral(properties) = &computed.expr else {
        panic!("expected object literal");
    };
    assert!(matches!(
        properties.as_slice(),
        [ObjectPropertyIr::Data { .. }]
    ));

    let method = object_literal("({ __proto__() {} });");
    let ExprIr::ObjectLiteral(properties) = &method.expr else {
        panic!("expected object literal");
    };
    assert!(matches!(
        properties.as_slice(),
        [ObjectPropertyIr::Method { .. }]
    ));
}

#[test]
fn lowers_object_spreads_in_property_evaluation_order() {
    let program = lower_script("let source = { copied: 2 }; ({ before: 1, ...source, after: 3 });");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("script ir should exist");
    let StatementIr::Expression(object) = script.body.statements.last().expect("object expression")
    else {
        panic!("expected object literal expression");
    };
    let ExprIr::ObjectLiteral(properties) = &object.expr else {
        panic!("expected object literal");
    };
    assert!(matches!(
        properties.as_slice(),
        [
            ObjectPropertyIr::Data { key: before, .. },
            ObjectPropertyIr::Spread { .. },
            ObjectPropertyIr::Data { key: after, .. },
        ] if before == "before" && after == "after"
    ));
    assert!(
        object.heap_shape.is_none(),
        "spread keys make the object shape dynamic"
    );
}

#[test]
fn lowers_fractional_array_keys_as_named_properties() {
    let program = lower_script("const arr = [39, 42]; arr[1.1] = 'other prop'; arr[1.1];");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");

    let StatementIr::Expression(write) = &script.body.statements[1] else {
        panic!("expected array property write");
    };
    let ExprIr::OrdinaryPropertyAssignment(assignment) = &write.expr else {
        panic!(
            "expected ordinary property assignment, got {:?}",
            write.expr
        );
    };
    assert!(matches!(
        assignment.referenced_name(),
        PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Number
    ));

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected array property read");
    };
    let ExprIr::SpecOperation {
        operation,
        operands,
    } = &read.expr
    else {
        panic!("expected GetV operation, got {:?}", read.expr);
    };
    assert_eq!(*operation, SpecOperationIr::GetV);
    assert_eq!(operands.len(), 2);
    assert!(matches!(operands[1].expr, ExprIr::Number(_)));
    assert_eq!(read.kind, ValueKind::Dynamic);
}

#[test]
fn preserves_well_known_symbols_as_computed_property_keys() {
    let program = lower_script(
        r#"let target = { "Symbol.asyncIterator": "string" };
target[Symbol.asyncIterator] = "symbol";
target[Symbol.asyncIterator];"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");

    let StatementIr::Expression(write) = &script.body.statements[1] else {
        panic!("expected property write");
    };
    let ExprIr::OrdinaryPropertyAssignment(assignment) = &write.expr else {
        panic!(
            "expected ordinary property assignment, got {:?}",
            write.expr
        );
    };
    assert!(
        matches!(assignment.referenced_name(), PropertyKeyIr::StringExpr(key)
            if key.kind == ValueKind::Symbol
                && matches!(key.expr, ExprIr::WellKnownSymbol(WellKnownSymbol::AsyncIterator))),
        "well-known Symbol write key must retain its Symbol kind, got {:?}",
        assignment.referenced_name()
    );

    // A property Set may invoke an inherited setter and replace global Symbol.
    // A fresh read has a Symbol proof; the read after Set must reacquire it.
    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected property read");
    };
    let ExprIr::SpecOperation {
        operation: SpecOperationIr::GetV,
        operands,
    } = &read.expr
    else {
        panic!("expected GetV operation, got {:?}", read.expr);
    };
    assert!(
        matches!(operands.as_slice(), [_, TypedExpr {
            expr: ExprIr::SpecOperation { operation: SpecOperationIr::GetV, operands }, ..
        }] if matches!(operands.as_slice(), [TypedExpr {
            expr: ExprIr::GlobalIdentifierRead { name }, ..
        }, TypedExpr { expr: ExprIr::String(key), .. }]
            if name == "Symbol" && key == "asyncIterator")),
        "the later key must reacquire Symbol.asyncIterator, got {operands:?}"
    );
}

#[test]
fn symbol_and_similarly_named_string_properties_do_not_share_shape_facts() {
    let program = lower_script(
        r#"let target = {
                [Symbol.iterator]: 1,
                "Symbol.iterator": "ordinary"
            };
            target[Symbol.iterator];
            target["Symbol.iterator"];"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");

    let StatementIr::Expression(symbol_read) = &script.body.statements[1] else {
        panic!("expected symbol property read");
    };
    assert_eq!(symbol_read.kind, ValueKind::Number);

    let StatementIr::Expression(string_read) = &script.body.statements[2] else {
        panic!("expected string property read");
    };
    assert_eq!(string_read.kind, ValueKind::String);
}

#[test]
fn similarly_named_string_property_cannot_supply_a_symbol_read() {
    let program = lower_script(
            "function stringNamed() {} let target = { 'Symbol.iterator': stringNamed }; target[Symbol.iterator];",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let string_named = script
        .functions
        .iter()
        .find(|function| function.name == "stringNamed")
        .expect("source function should be lowered");
    let StatementIr::Expression(symbol_read) = script.body.statements.last().unwrap() else {
        panic!("expected symbol property read");
    };
    assert!(
        !symbol_read
            .function_targets
            .known_targets()
            .contains(&string_named.id),
        "a string-keyed property must not satisfy a Symbol-keyed read"
    );
}

#[test]
fn distinct_computed_symbols_are_omitted_from_string_key_shapes() {
    let program = lower_script(
        r#"const makeSymbol = Symbol;
            const first = makeSymbol("first");
            const second = makeSymbol("second");
            let target = { [first]: 1, [second]: "second" };"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[3] else {
        panic!("expected object declaration");
    };
    let Some(HeapShape::Object(shape)) = init.heap_shape.as_deref() else {
        panic!("expected object shape");
    };
    assert!(shape.properties.is_empty());
}

#[test]
fn computed_property_keys_respect_a_shadowed_symbol_binding() {
    let program = lower_script(
        r#"const Symbol = { iterator: "actual" };
let target = { actual: 1, "Symbol.iterator": 2 };
target[Symbol.iterator];"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");

    let StatementIr::Expression(read) = &script.body.statements[2] else {
        panic!("expected property read");
    };
    let ExprIr::SpecOperation {
        operation: SpecOperationIr::GetV,
        operands,
    } = &read.expr
    else {
        panic!("expected GetV operation, got {:?}", read.expr);
    };
    assert!(
        !matches!(
            operands.as_slice(),
            [
                _,
                TypedExpr {
                    expr: ExprIr::WellKnownSymbol(_),
                    ..
                }
            ]
        ),
        "shadowed Symbol key must not resolve to the well-known Symbol marker"
    );
}

#[test]
fn specialized_iterator_reads_preserve_symbol_keys() {
    // Each first lookup has a live global Symbol proof. A preceding unknown
    // property Get would correctly require reacquiring the mutable global.
    for source in ["[][Symbol.iterator];", "\"\"[Symbol.iterator];"] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(read) = &script.body.statements[0] else {
            panic!("expected property read");
        };
        match &read.expr {
            ExprIr::PropertyRead { key, .. } => assert!(
                matches!(key, PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Symbol),
                "well-known iterator reads must retain their Symbol key, got {key:?}"
            ),
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } => assert!(
                matches!(operands.as_slice(), [_, key] if key.kind == ValueKind::Symbol),
                "well-known iterator reads must retain their Symbol operand, got {operands:?}"
            ),
            _ => panic!("expected property read, got {:?}", read.expr),
        }
    }

    let program = lower_script("[][\"Symbol.iterator\"];");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(literal_read) = &script.body.statements[0] else {
        panic!("expected literal-string property read");
    };
    match &literal_read.expr {
        ExprIr::PropertyRead {
            key: literal_key, ..
        } => assert_eq!(
            literal_key,
            &PropertyKeyIr::StaticString("Symbol.iterator".to_string())
        ),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } => assert!(
            matches!(
                operands.as_slice(),
                [_, TypedExpr {
                    kind: ValueKind::String,
                    expr: ExprIr::String(key),
                    ..
                }] if key == "Symbol.iterator"
            ),
            "literal iterator key must remain a String operand, got {operands:?}"
        ),
        _ => panic!(
            "expected literal-string property read, got {:?}",
            literal_read.expr
        ),
    }
}

#[test]
fn computed_object_literal_keys_preserve_symbol_identity() {
    let program = lower_script(
        r#"({
  [Symbol.iterator]: 1,
  [Symbol.toPrimitive]() { return 2; },
  ["Symbol.iterator"]: 3
});"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(object) = &script.body.statements[0] else {
        panic!("expected object literal");
    };
    let ExprIr::ObjectLiteral(properties) = &object.expr else {
        panic!("expected object literal, got {:?}", object.expr);
    };
    assert!(
        matches!(
            properties.as_slice(),
            [
                ObjectPropertyIr::ComputedData { key: symbol_data, .. },
                ObjectPropertyIr::ComputedMethod { key: symbol_method, .. },
                ObjectPropertyIr::Data {
                    key: string_key,
                    ..
                }
            ] if symbol_data.kind == ValueKind::Symbol
                && symbol_method.kind == ValueKind::Symbol
                && string_key == "Symbol.iterator"
        ),
        "computed Symbol keys and ordinary string keys must stay distinct: {properties:?}"
    );
}

#[test]
fn lowers_runtime_number_array_keys_through_to_property_key() {
    for key_init in ["1", "1.1", "-1", "NaN", "Infinity"] {
        let source = format!("let key = {key_init}; let array = []; array[key] = 7; array[key];");
        let program = lower_script(&source);
        assert!(program.is_wasm_supported(), "{source}");
        let script = program.script.as_ref().expect("script ir should exist");

        let StatementIr::Expression(write) = &script.body.statements[2] else {
            panic!("expected array property write for {source}");
        };
        let ExprIr::OrdinaryPropertyAssignment(assignment) = &write.expr else {
            panic!(
                "expected ordinary property assignment for {source}, got {:?}",
                write.expr
            );
        };
        assert!(
                matches!(assignment.referenced_name(), PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Number),
                "runtime numeric write key must retain its raw ToPropertyKey input for {source}, got {:?}",
                assignment.referenced_name()
            );

        let StatementIr::Expression(read) = &script.body.statements[3] else {
            panic!("expected array property read for {source}");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &read.expr
        else {
            panic!("expected GetV operation for {source}, got {:?}", read.expr);
        };
        assert!(
            *operation == SpecOperationIr::GetV
                && operands.len() == 2
                && operands[1].kind == ValueKind::Number,
            "runtime numeric read key must use GetV for {source}, got {operands:?}"
        );
    }
}

#[test]
fn literal_property_key_facts_distinguish_lone_units_backslashes_and_encoding_markers() {
    let program = lower_script(r#"({'\uD800': 1, '\\uD800': 2, '\u{F0000}D800': 3});"#);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("script IR");
    let StatementIr::Expression(expression) = &script.body.statements[0] else {
        panic!("object literal expression");
    };
    let ExprIr::ObjectLiteral(properties) = &expression.expr else {
        panic!("object literal");
    };
    let keys: Vec<_> = properties.iter().map(|property| match property {
        ObjectPropertyIr::Data { key, .. } => key.as_str(),
        _ => panic!("literal data property"),
    }).collect();
    let lone = encode_js_string_utf16(&[0xD800]);
    let marker = encode_js_string_utf16(&"\u{F0000}D800".encode_utf16().collect::<Vec<_>>());
    assert_eq!(keys, [lone.as_str(), "\\uD800", marker.as_str()]);
    let Some(HeapShape::Object(shape)) = expression.heap_shape.as_deref() else {
        panic!("literal property shape facts");
    };
    assert_eq!(shape.properties.len(), 3);
    for key in keys { assert!(shape.properties.contains_key(key)); }
}
