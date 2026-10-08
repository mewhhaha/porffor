#[test]
fn map_iterable_construction_preserves_the_map_instance_shape() {
    for source in [
        "new Map([]);",
        "function makeMap(iterable) { return new Map(iterable); } makeMap([]);",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected constructed Map instance for {source}");
        };
        let Some(HeapShape::Object(instance_shape)) = instance.heap_shape.as_deref() else {
            panic!("expected Map instance shape for {source}");
        };
        let Some(HeapShape::Object(prototype_shape)) = instance_shape.prototype.as_deref() else {
            panic!("expected Map prototype shape for {source}");
        };
        assert!(prototype_shape.properties.contains_key("set"), "{source}");
        assert!(
            prototype_shape.properties.contains_key("forEach"),
            "{source}"
        );
    }
}

#[test]
fn set_iterable_construction_preserves_the_set_instance_shape() {
    for source in [
        "new Set([]);",
        "function makeSet(iterable) { return new Set(iterable); } makeSet([]);",
        "new Set('ab');",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected constructed Set instance for {source}");
        };
        let Some(HeapShape::Object(instance_shape)) = instance.heap_shape.as_deref() else {
            panic!("expected Set instance shape for {source}");
        };
        let Some(HeapShape::Object(prototype_shape)) = instance_shape.prototype.as_deref() else {
            panic!("expected Set prototype shape for {source}");
        };
        assert!(prototype_shape.properties.contains_key("add"), "{source}");
        assert!(
            prototype_shape.properties.contains_key("forEach"),
            "{source}"
        );
    }
}

#[test]
fn set_algebra_preserves_the_set_instance_shape() {
    for method in ["difference", "intersection", "symmetricDifference", "union"] {
        let source = format!("new Set().{method}(new Set());");
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected Set algebra result for {source}");
        };
        let Some(HeapShape::Object(instance_shape)) = instance.heap_shape.as_deref() else {
            panic!("expected Set instance shape for {source}");
        };
        let Some(HeapShape::Object(prototype_shape)) = instance_shape.prototype.as_deref() else {
            panic!("expected Set prototype shape for {source}");
        };
        assert!(prototype_shape.properties.contains_key("add"), "{source}");
        assert!(prototype_shape.properties.contains_key("union"), "{source}");
    }
}

#[test]
fn set_predicates_have_boolean_result_kind() {
    for method in ["isDisjointFrom", "isSubsetOf", "isSupersetOf"] {
        let source = format!("new Set().{method}(new Set());");
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected Set predicate result for {source}");
        };
        assert_eq!(result.kind, ValueKind::Boolean, "{source}");
        assert_eq!(
            result.possible_kinds,
            KindSet::from_kind(ValueKind::Boolean),
            "{source}"
        );
    }
}

#[test]
fn optional_super_method_call_uses_current_this_receiver() {
    let source = "class Base { method() { return this; } } class Derived extends Base { call() { return super.method?.(); } }";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "Derived.call")
        .expect("derived call method should be lowered");
    let StatementIr::Return(expr) = &function.body.statements[0] else {
        panic!(
            "expected return statement, got {:?}",
            function.body.statements
        );
    };
    let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
        panic!("expected optional super call IR, got {:?}", expr.expr);
    };
    assert!(matches!(
        &target.expr,
        ExprIr::SuperPropertyRead {
            key: PropertyKeyIr::StaticString(key),
            ..
        } if key == "method"
    ));
    assert!(matches!(
        chain.as_slice(),
        [OptionalChainOperationIr::Call {
            args,
            receiver: OptionalChainCallReceiverIr::CurrentThis,
            shorted: true,
            boundary_before: false,
        }] if args.is_empty()
    ));
}

#[test]
fn optional_private_call_preserves_mandatory_get_and_the_original_receiver() {
    let source = "class C { #method() {} call() { return this.#method?.(); } }";
    let program = lower_script(source);
    assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    let function = program.script.as_ref().unwrap().functions.iter().find(|function| function.name == "C.call").unwrap();
    let StatementIr::Return(TypedExpr { expr: ExprIr::OptionalPropertyChain { chain, .. }, .. }) = &function.body.statements[0] else { panic!("ordered private Reference chain"); };
    assert!(matches!(chain.as_slice(), [
        OptionalChainOperationIr::PrivateProperty { private_name_id, shorted: false },
        OptionalChainOperationIr::Call { args, receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined, shorted: true, boundary_before: false },
    ] if *private_name_id == function.private_name_ids["method"] && args.is_empty()));
}
