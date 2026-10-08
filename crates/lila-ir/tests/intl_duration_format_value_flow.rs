use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, ScriptIr, StandardBuiltinId, StatementIr, ValueKind};

fn lower_script(source: &str) -> ScriptIr {
    let program = lower(&parse(source, ParseOptions::script()).expect("duration source parses"));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

#[test]
fn duration_constructor_keeps_its_real_target_through_intrinsic_capture() {
    for source in [
        "let formatter = new Intl.DurationFormat('en'); formatter;",
        "const DurationFormat = Intl.DurationFormat; let formatter = new DurationFormat('de', {style:'digital'}); formatter;",
        "const DurationFormat = Intl.DurationFormat; let formatter = new DurationFormat(['fr'], {get hours(){return 'numeric';}}); formatter;",
    ] {
        let script = lower_script(source);
        let binding = script.body.statements.iter().find_map(|statement| match statement {
            StatementIr::Lexical { init, .. } if matches!(&init.expr, ExprIr::Construct { .. }) => Some(init),
            _ => None,
        }).expect("the formatter must be initialized through real constructor lowering");
        let ExprIr::Construct { callee, .. } = &binding.expr else { unreachable!() };
        assert_eq!(callee.function_targets.exact_single_target(),
            Some(&StandardBuiltinId::IntlDurationFormatConstructor.function_id()), "{source}");
        assert!(binding.possible_kinds.contains(ValueKind::Object), "{source}");
        assert_eq!(script.result_kind(), ValueKind::Object, "{source}");
    }
}

#[test]
fn duration_observations_preserve_captured_string_and_boolean_flow() {
    for observed in [
        "formatter.format({seconds:1})",
        "formatter.formatToParts({seconds:1})",
        "formatter.resolvedOptions()",
    ] {
        for (tail, expected) in [
            ("value === undefined;", ValueKind::Boolean),
            ("stringify(value);", ValueKind::String),
            ("value + '';", ValueKind::String),
        ] {
            let source = format!("const stringify = String; let formatter = new Intl.DurationFormat('en'); let value = {observed}; {tail}");
            let script = lower_script(&source);
            let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
                panic!(
                    "String is captured before observable constructor and duration property reads"
                );
            };
            assert_eq!(
                init.function_targets.exact_single_target(),
                Some(&StandardBuiltinId::StringConstructor.function_id()),
                "{source}"
            );
            assert_eq!(script.result_kind(), expected, "{source}");
        }
    }
}
