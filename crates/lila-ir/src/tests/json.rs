#[test]
fn lowers_json_parse_reviver_with_static_string_binding_as_an_ordinary_call() {
    let program =
        lower_script("var json = \"[1, 2]\"; JSON.parse(json, function(k, v) { return v; });");
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = &script.body.statements[1] else {
        panic!("expected JSON.parse expression");
    };
    let call = indirect_call_body(expr).expect("JSON.parse must retain runtime Call");
    let ExprIr::CallIndirect { callee, args, .. } = &call.expr else {
        unreachable!()
    };
    assert_eq!(
        callee.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::JsonParse.function_id())
    );
    assert_eq!(args.len(), 2);
    assert!(matches!(&args[0].expr, ExprIr::GlobalIdentifierRead { name } if name == "json"));
    assert!(matches!(&args[1].expr, ExprIr::FunctionValue(_)));
}

#[test]
fn json_parse_spread_arguments_do_not_specialize() {
    let program = lower_script(
        "function firstReviver(key, value) { return value; } \
             function secondReviver(key, value) { return value; } \
             JSON.parse(...['[1]', firstReviver]); \
             JSON.parse('[2]', ...[secondReviver]);",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let calls = script
        .body
        .statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::Expression(expression) => Some(expression),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 2, "expected both spread calls: {calls:?}");

    for call in calls {
        let call = indirect_call_body(call).expect("spread JSON.parse should remain a call");
        let ExprIr::CallIndirect { args, .. } = &call.expr else {
            unreachable!("indirect_call_body only returns indirect calls");
        };
        assert!(
            args.iter()
                .any(|argument| matches!(argument.expr, ExprIr::SpreadArgument(_))),
            "the spread operand must remain explicit: {args:?}"
        );
    }
}

#[test]
fn mutable_static_json_binding_in_a_repeated_for_test_does_not_specialize() {
    let program = lower_script(
        "let json = '[1]'; \
             function reviver(key, value) { return value; } \
             for (; JSON.parse(json, reviver); json = '[2]') { \
                 if (json === '[2]') break; \
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let test = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::For {
                test: Some(test), ..
            } => Some(test),
            _ => None,
        })
        .expect("expected the repeated for-loop test");

    assert!(
        indirect_call_body(test).is_some(),
        "the repeated JSON.parse call must be retained: {test:?}"
    );
}

#[test]
fn tdz_shadowed_json_input_retains_its_reference_error() {
    let program = lower_script(
        "function run() { \
                 const json = '[1]'; \
                 { \
                     JSON.parse(json, function reviver(key, value) { return value; }); \
                     let json = '[2]'; \
                 } \
             } \
             run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let run = script
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function should be lowered");
    let block = run
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Block(block) => Some(block),
            _ => None,
        })
        .expect("run should contain the shadowing block");
    let StatementIr::Expression(call) = &block.statements[0] else {
        panic!("expected JSON.parse before the inner declaration");
    };

    let call = indirect_call_body(call).expect("TDZ JSON.parse should remain a call");
    let ExprIr::CallIndirect { args, .. } = &call.expr else {
        unreachable!("indirect_call_body only returns indirect calls");
    };
    assert!(matches!(
        args.first().map(|argument| &argument.expr),
        Some(ExprIr::RuntimeThrow {
            name: NativeErrorKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn json_parse_runtime_call_retains_non_property_callee_evaluation() {
    let program = lower_script(
        "function run() { \
                 let hits = 0; \
                 (hits++, JSON.parse)('[1]', function firstReviver(key, value) { return value; }); \
             } \
             run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let run = script
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function should be lowered");
    let non_property_call = run
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => Some(expression),
            _ => None,
        })
        .expect("run should contain its JSON.parse call");

    let call = indirect_call_body(non_property_call).expect("JSON.parse must retain ordinary Call");
    let ExprIr::CallIndirect {
        callee,
        this_arg,
        args,
        ..
    } = &call.expr
    else {
        unreachable!()
    };
    assert!(this_arg.is_none());
    let [input, reviver] = args.as_slice() else {
        panic!("expected input and reviver operands")
    };
    let ExprIr::Comma {
        lhs: callee_effect,
        rhs: acquired_callee,
    } = &callee.expr
    else {
        panic!("non-property call must retain its comma callee: {callee:?}");
    };
    assert!(matches!(
        callee_effect.expr,
        ExprIr::UpdateIdentifier { .. }
    ));
    assert!(matches!(
        &acquired_callee.expr,
        ExprIr::PropertyRead {
            target,
            key: PropertyKeyIr::StaticString(key),
        } if matches!(&target.expr, ExprIr::GlobalPropertyRead { name } if name == JSON_NAME)
            && key == "parse"
    ));
    assert!(matches!(&input.expr, ExprIr::String(source) if source == "[1]"));
    assert!(matches!(reviver.expr, ExprIr::FunctionValue(_)));
}

#[test]
fn json_parse_runtime_call_retains_property_callee_evaluation() {
    let program = lower_script(
            "function run() { \
                 let hits = 0; \
                 (hits++, JSON).parse('[2]', function secondReviver(key, value) { return value; }); \
             } \
             run();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let run = script
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function should be lowered");
    let property_call = run
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => Some(expression),
            _ => None,
        })
        .expect("run should contain its JSON.parse call");

    let ExprIr::MaterializeBinding {
        name,
        value: receiver,
        body: call,
    } = &property_call.expr
    else {
        panic!("property call must evaluate its receiver once: {property_call:?}");
    };
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(this_arg),
        args,
        ..
    } = &call.expr
    else {
        panic!("property call must retain acquired callee and receiver: {call:?}");
    };
    let ExprIr::PropertyRead {
        target,
        key: PropertyKeyIr::StaticString(key),
    } = &callee.expr
    else {
        panic!("property call must retain explicit parse acquisition: {callee:?}");
    };
    assert_eq!(key, "parse");
    assert!(matches!(&target.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(matches!(&this_arg.expr, ExprIr::Identifier(storage) if storage == name));
    let [input, reviver] = args.as_slice() else {
        panic!("expected input and reviver operands")
    };
    let ExprIr::Comma {
        lhs: receiver_effect,
        rhs: json_receiver,
    } = &receiver.expr
    else {
        panic!("property call must retain its comma receiver: {receiver:?}");
    };
    assert!(matches!(
        receiver_effect.expr,
        ExprIr::UpdateIdentifier { .. }
    ));
    assert!(matches!(
        &json_receiver.expr,
        ExprIr::GlobalPropertyRead { name } if name == JSON_NAME
    ));
    assert!(matches!(&input.expr, ExprIr::String(source) if source == "[2]"));
    assert!(matches!(reviver.expr, ExprIr::FunctionValue(_)));
}

#[test]
fn leaving_a_shadowing_scope_keeps_json_parse_reading_the_outer_input() {
    let program = lower_script(
        "function run() { \
                 var json = '[1]'; \
                 { let json = '[2]'; } \
                 return JSON.parse(json, function reviver(key, value) { return value; }); \
             } \
             run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let run = script
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function should be lowered");
    let returned = function_return(run).expect("run should return JSON.parse");

    let call = indirect_call_body(returned).expect("post-scope JSON.parse stays an ordinary call");
    let ExprIr::CallIndirect { args, .. } = &call.expr else {
        unreachable!()
    };
    assert!(matches!(&args[0].expr, ExprIr::Identifier(name) if name == "json"));
}

#[test]
fn json_parse_snapshots_its_input_before_reviver_effects() {
    let program = lower_script(
        "let json = '[1]'; \
             let first = JSON.parse(json, function(k, v) { json = '[2]'; return v; }); \
             let second = JSON.parse(json, function(k, v) { return v; });",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let [StatementIr::Lexical { name: json, .. }, StatementIr::Lexical {
        name: first,
        init: first_init,
        ..
    }, StatementIr::Lexical {
        name: second,
        init: second_init,
        ..
    }] = script.body.statements.as_slice()
    else {
        panic!(
            "expected three lexical declarations: {:?}",
            script.body.statements
        );
    };
    assert_eq!(
        (json.as_str(), first.as_str(), second.as_str()),
        ("json", "first", "second")
    );
    for init in [first_init, second_init] {
        let call =
            indirect_call_body(init).expect("each parse must read its live input once at Call");
        let ExprIr::CallIndirect { callee, this_arg: Some(receiver), args, .. } = &call.expr else {
            unreachable!()
        };
        let acquired_receiver = match &callee.expr {
            ExprIr::PropertyRead { target, key: PropertyKeyIr::StaticString(key) } if key == "parse" => {
                assert_eq!(callee.function_targets.exact_single_target(), Some(&StandardBuiltinId::JsonParse.function_id()));
                target.as_ref()
            }
            ExprIr::SpecOperation { operation: SpecOperationIr::GetV, operands } => {
                let [target, key] = operands.as_slice() else { panic!("GetV operands"); };
                assert!(matches!(&key.expr, ExprIr::String(key) if key == "parse"));
                target
            }
            _ => panic!("the live JSON.parse property must be acquired before arguments: {callee:?}"),
        };
        assert_eq!(acquired_receiver.expr, receiver.expr);
        let [input, reviver] = args.as_slice() else {
            panic!("expected input and reviver operands")
        };
        assert!(matches!(&input.expr, ExprIr::Identifier(name) if name == json));
        assert!(matches!(reviver.expr, ExprIr::FunctionValue(_)));
    }
}

#[test]
fn dynamic_json_parse_observes_reviver_holder_kinds() {
    let program = lower_script(
            "function parse(text) { return JSON.parse(text, function reviver(key, value) { this[1] = value; return value; }); } parse('[1, 2]');",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let parse = script
        .functions
        .iter()
        .find(|function| function.name == "parse")
        .expect("parse should be lowered");
    let returned = function_return(parse).expect("parse should return JSON.parse");
    let call = indirect_call_body(returned).expect("parse should call JSON.parse");
    let ExprIr::CallIndirect { callee, args, .. } = &call.expr else {
        unreachable!("indirect_call_body only returns indirect calls");
    };
    assert_eq!(
        callee.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::JsonParse.function_id())
    );
    let reviver_id = args
        .get(1)
        .and_then(|reviver| reviver.function_targets.exact_single_target())
        .expect("JSON.parse should receive the lowered reviver");
    let reviver = script
        .functions
        .iter()
        .find(|function| &function.id == reviver_id)
        .expect("the JSON.parse reviver should be lowered");
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::OrdinaryPropertyAssignment(assignment),
        ..
    }) = &reviver.body.statements[0]
    else {
        panic!("expected reviver holder write");
    };
    assert_eq!(assignment.base_and_receiver().kind, ValueKind::Dynamic);
    assert_eq!(
        assignment.base_and_receiver().possible_kinds,
        KindSet::HEAP_COERCIBLE_ONLY.union(KindSet::from_kind(ValueKind::Function))
    );
    assert_eq!(
        assignment.base_and_receiver().function_targets,
        FunctionTargetKnowledge::unknown()
    );
    assert!(matches!(
        assignment.referenced_name(),
        PropertyKeyIr::StringExpr(key) if key.kind == ValueKind::Number
    ));
}

#[test]
fn captured_json_parse_reviver_remains_conservative_without_specialization() {
    let program = lower_script(
            "function parse(text) { let captured; return JSON.parse(text, function reviver(key, value) { captured = value; this[1] = value; return value; }); } parse('[1, 2]');",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let reviver = script
        .functions
        .iter()
        .find(|function| function.name == "reviver")
        .expect("captured reviver should be lowered");
    assert_eq!(
        script
            .functions
            .iter()
            .filter(|function| function.name == "reviver")
            .count(),
        1
    );
    let assignment = reviver
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::OrdinaryPropertyAssignment(assignment),
                ..
            }) => Some(assignment),
            _ => None,
        })
        .expect("captured reviver should write to its holder");

    assert_eq!(assignment.base_and_receiver().kind, ValueKind::Dynamic);
    assert_eq!(
        assignment.base_and_receiver().possible_kinds,
        KindSet::all_runtime_tags()
    );
}
