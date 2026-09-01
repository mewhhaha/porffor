use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, DynamicFunctionKind, DynamicSourceGap, DynamicSourceKind, ProgramIr, UnsupportedFeature,
    ValueKind,
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

    assert_eq!(
        dynamic_source_gaps(&program),
        vec![DynamicSourceGap::aot_known_source(
            DynamicSourceKind::IndirectEval,
        )]
    );
}

#[test]
fn intrinsic_call_forwards_runtime_eval_source_as_indirect_eval() {
    let program = lower_script("eval.call(undefined, String('source'));");

    assert_eq!(
        dynamic_source_gaps(&program),
        vec![DynamicSourceGap::runtime_source(
            DynamicSourceKind::IndirectEval,
        )]
    );
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
        assert_eq!(
            dynamic_source_gaps(&program),
            vec![DynamicSourceGap::aot_known_source(
                DynamicSourceKind::Function(kind),
            )],
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn intrinsic_call_preserves_no_source_eval_result_precision() {
    for (source, expected_kind) in [
        ("eval.call(undefined);", ValueKind::Undefined),
        ("eval.call('ignored this');", ValueKind::Undefined),
        ("eval.call(undefined, 7);", ValueKind::Number),
        ("eval.call(undefined, true, 'ignored');", ValueKind::Boolean),
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        assert_eq!(
            program.script.as_ref().expect("script IR").result_kind(),
            expected_kind,
            "{source}"
        );
    }
}

#[test]
fn intrinsic_call_keeps_the_receiver_identity_captured_before_arguments() {
    let program = lower_script("eval.call(undefined, 'source', (eval = Math.abs, 0));");

    assert_eq!(
        dynamic_source_gaps(&program),
        vec![DynamicSourceGap::aot_known_source(
            DynamicSourceKind::IndirectEval,
        )]
    );
}

#[test]
fn intrinsic_call_preflights_every_retained_exact_receiver_candidate() {
    let source = "let target = unknown ? eval : Math.abs; target.call(undefined, 'source');";
    let program = lower_script(source);

    assert_eq!(
        dynamic_source_gaps(&program),
        vec![DynamicSourceGap::aot_known_source(
            DynamicSourceKind::IndirectEval,
        )],
        "{source}: {:?}",
        program.diagnostics
    );
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
