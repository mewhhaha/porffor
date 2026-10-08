use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, DynamicFunctionKind, DynamicSourceGap, ExprIr, PreparedScriptKind, ProgramIr,
    PropertyKeyIr, StandardBuiltinId, StatementIr, TypedExpr, UnsupportedFeature, ValueKind,
};

fn lower_script(source: &str) -> ProgramIr {
    let parsed = parse(source, ParseOptions::script()).expect("script should parse");
    lower(&parsed)
}

fn dynamic_source_gaps(program: &ProgramIr) -> Vec<DynamicSourceGap> {
    program
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic.unsupported_feature() {
            Some(UnsupportedFeature::DynamicSource(gap)) => Some(gap),
            None => None,
        })
        .collect()
}

#[test]
fn intrinsic_call_forwards_aot_known_eval_source_as_indirect_eval() {
    let program = lower_script("eval.call(undefined, 'source');");

    assert!(
        dynamic_source_gaps(&program).is_empty(),
        "{:?}",
        program.diagnostics
    );
    assert!(program
        .script
        .as_ref()
        .expect("script IR")
        .prepared_scripts
        .iter()
        .any(|prepared| prepared.kind == PreparedScriptKind::IndirectEval));
}

#[test]
fn intrinsic_call_forwards_runtime_eval_source_as_indirect_eval() {
    let program = lower_script("eval.call(undefined, globalThis.unknownSource);");

    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("forwarded runtime invocation");
    assert!(script.prepared_scripts.is_empty());
    assert_eq!(script.result_kind(), ValueKind::Dynamic);
}

#[test]
fn intrinsic_call_forwards_every_function_family_identity() {
    for (source, kind) in [
        (
            "Function.call(undefined, 'return 1');",
            DynamicFunctionKind::Ordinary,
        ),
        (
            "Object.getPrototypeOf(function*() {}).constructor.call(undefined, 'yield 1');",
            DynamicFunctionKind::Generator,
        ),
        (
            "Object.getPrototypeOf(async function() {}).constructor.call(undefined, 'return 1');",
            DynamicFunctionKind::Async,
        ),
        (
            "Object.getPrototypeOf(async function*() {}).constructor.call(undefined, 'yield 1');",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        let program = lower_script(source);
        assert!(
            program.diagnostics.is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
        let prepared = &program
            .script
            .expect("script IR")
            .prepared_dynamic_functions;
        assert_eq!(prepared.len(), 1, "{source}");
        assert_eq!(prepared[0].kind, kind, "{source}");
        assert!(matches!(
            prepared[0].outcome,
            lila_ir::PreparedDynamicFunctionOutcome::Compiled { .. }
        ));
    }
}

#[test]
fn non_string_eval_forwarding_retains_the_unproven_method_get_and_original_arguments() {
    for (source, expected_arguments) in [
        (
            "eval.call(undefined);",
            vec![ExprIr::GlobalPropertyRead {
                name: "undefined".into(),
            }],
        ),
        (
            "eval.call('ignored this');",
            vec![ExprIr::String("ignored this".into())],
        ),
        (
            "eval.call(undefined, 7);",
            vec![
                ExprIr::GlobalPropertyRead {
                    name: "undefined".into(),
                },
                ExprIr::Number(7.0f64.to_bits()),
            ],
        ),
        (
            "eval.call(undefined, true, 'ignored');",
            vec![
                ExprIr::GlobalPropertyRead {
                    name: "undefined".into(),
                },
                ExprIr::Boolean(true),
                ExprIr::String("ignored".into()),
            ],
        ),
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script IR");
        // A function's incomplete shape does not prove its current inherited
        // `call`. Preserve the Get and call operands; the native admission
        // cohort checks eval's exact non-String pass-through results.
        assert_eq!(script.result_kind(), ValueKind::Dynamic, "{source}");
        // Finite candidate discovery may retain source spellings for these
        // operands. Only the acquired runtime callee decides how to use them.
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::MaterializeBinding { name, value, body },
            ..
        }) = script.body.statements.last().expect("forwarded invocation")
        else {
            panic!("the original eval receiver must be acquired once: {source}");
        };
        assert!(matches!(
            &value.expr,
            ExprIr::GlobalPropertyRead { name } | ExprIr::GlobalIdentifierRead { name }
                if name == "eval"
        ));
        assert!(value
            .function_targets
            .known_targets()
            .contains(&StandardBuiltinId::EvalFunction.function_id()));
        let ExprIr::CallIndirect {
            direct_eval: None,
            callee,
            this_arg: Some(this_arg),
            args,
            ..
        } = &body.expr
        else {
            panic!("forwarding must retain the actual indirect Call: {source}");
        };
        assert!(callee.function_targets.exact_targets().is_none());
        assert!(callee
            .function_targets
            .known_targets()
            .contains(&StandardBuiltinId::FunctionPrototypeCall.function_id()));
        let ExprIr::PropertyRead { target, key } = &callee.expr else {
            panic!("the live call property must be acquired: {source}");
        };
        assert_eq!(key, &PropertyKeyIr::StaticString("call".into()));
        assert!(matches!(&target.expr, ExprIr::Identifier(base) if base == name));
        assert!(matches!(&this_arg.expr, ExprIr::Identifier(receiver) if receiver == name));
        assert_eq!(
            args.iter()
                .map(|argument| &argument.expr)
                .collect::<Vec<_>>(),
            expected_arguments.iter().collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn intrinsic_call_keeps_the_receiver_identity_captured_before_arguments() {
    let program = lower_script("eval.call(undefined, 'source', (eval = Math.abs, 0));");

    assert!(
        dynamic_source_gaps(&program).is_empty(),
        "{:?}",
        program.diagnostics
    );
    assert!(program
        .script
        .as_ref()
        .expect("script IR")
        .prepared_scripts
        .iter()
        .any(|prepared| prepared.kind == PreparedScriptKind::IndirectEval));
}

#[test]
fn intrinsic_call_preflights_every_retained_exact_receiver_candidate() {
    let source = "let target = unknown ? eval : Math.abs; target.call(undefined, 'source');";
    let program = lower_script(source);

    assert!(
        dynamic_source_gaps(&program).is_empty(),
        "{:?}",
        program.diagnostics
    );
    assert!(program
        .script
        .as_ref()
        .expect("script IR")
        .prepared_scripts
        .iter()
        .any(|prepared| prepared.kind == PreparedScriptKind::IndirectEval));
}

#[test]
fn open_receiver_without_call_acquisition_authority_is_not_forwarded() {
    let source =
        "let target = unknown ? eval : globalThis.unknownFunction; target.call(undefined, 'source');";
    let program = lower_script(source);

    assert!(
        dynamic_source_gaps(&program).is_empty(),
        "{source}: {:?}",
        program.diagnostics
    );
}

#[test]
fn replaced_call_property_does_not_gain_forwarding_authority_from_spelling() {
    for source in [
        "eval.call = Math.abs; eval.call(undefined, 'source');",
        "Function.prototype.call = Math.abs; eval.call(undefined, 'source');",
        "Object.defineProperty(eval, 'call', { value: Math.abs }); eval.call(undefined, 'source');",
        "Object.defineProperty(Function.prototype, 'call', { value: Math.abs }); eval.call(undefined, 'source');",
        "delete Function.prototype.call; eval.call(undefined, 'source');",
    ] {
        let program = lower_script(source);

        assert!(
            dynamic_source_gaps(&program).is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn unknown_hook_erases_intrinsic_call_forwarding_authority() {
    let program = lower_script(
        "var target = eval; globalThis.unknownHook(); target.call(undefined, 'source');",
    );

    assert!(
        dynamic_source_gaps(&program).is_empty(),
        "{:?}",
        program.diagnostics
    );
}

#[test]
fn replaced_receiver_prototype_does_not_gain_intrinsic_call_authority() {
    for source in [
        "Object.setPrototypeOf(eval, { call: Math.abs }); eval.call(undefined, 'source');",
        "eval.__proto__ = { call: Math.abs }; eval.call(undefined, 'source');",
    ] {
        let program = lower_script(source);

        assert!(
            dynamic_source_gaps(&program).is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn comma_eval_candidates_survive_runtime_named_binding_resolution() {
    for call in [
        "(0, eval)('23')",
        "(0, (1, eval))('23')",
        "(0, eval).call(undefined, '23')",
        "(0, eval).apply(undefined, ['23'])",
        "Reflect.apply((0, eval), undefined, ['23'])",
    ] {
        let source = format!("var completion = {call}; eval(''); completion === 23;");
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let prepared = &program.script.expect("Script IR").prepared_scripts;
        assert!(
            prepared.iter().any(|entry| entry.source == "23"
                && entry.kind == PreparedScriptKind::IndirectEval
                && matches!(entry.outcome, lila_ir::PreparedScriptOutcome::Executable(_))),
            "comma call lacks its indirect source: {source}"
        );
        assert!(
            !prepared.iter().any(|entry| entry.source == "23"
                && matches!(entry.kind, PreparedScriptKind::DirectEval(_))),
            "comma call acquired direct-eval context: {source}"
        );
    }
}
