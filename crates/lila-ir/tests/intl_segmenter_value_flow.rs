use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, ScriptIr, StandardBuiltinId, StatementIr, ValueKind};

fn lower_script(source: &str) -> ScriptIr {
    let program = lower(&parse(source, ParseOptions::script()).expect("value-flow source parses"));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

#[test]
fn containing_keeps_object_or_undefined_through_storage_and_conditional_flow() {
    for index in ["0", "99"] {
        for tail in [
            "value;",
            "let retained; retained = value; retained;",
            "let holder = { value }; holder.value;",
            "value === undefined ? undefined : value;",
        ] {
            let source = format!(
                "let segmenter = new Intl.Segmenter('en', {{granularity:'word'}}); \
                 let segments = segmenter.segment('Hi!'); \
                 let value = segments.containing({index}); {tail}"
            );
            let script = lower_script(&source);
            let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
                panic!("Segmenter constructor must initialize the first binding");
            };
            let ExprIr::Construct { callee, .. } = &init.expr else {
                panic!("the source must exercise the real constructor lowering");
            };
            assert_eq!(
                callee.function_targets.exact_single_target(),
                Some(&StandardBuiltinId::IntlSegmenterConstructor.function_id()),
                "the static Intl.Segmenter reference must resolve to its genuine builtin: {source}"
            );
            let StatementIr::Lexical { init, .. } = &script.body.statements[2] else {
                panic!("containing's optional result must initialize a retained binding");
            };
            assert!(init.possible_kinds.contains(ValueKind::Object), "{source}");
            assert!(
                init.possible_kinds.contains(ValueKind::Undefined),
                "{source}"
            );
            let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
                panic!("the final expression must read the retained optional result");
            };
            assert!(value.possible_kinds.contains(ValueKind::Object), "{source}");
            assert!(
                value.possible_kinds.contains(ValueKind::Undefined),
                "{source}"
            );
            assert_eq!(script.result_kind(), ValueKind::Dynamic, "{source}");
        }
    }
}

#[test]
fn containing_optional_value_keeps_boolean_comparison_and_captured_string_coercion() {
    // Capture the intrinsic before any constructor or method can observe user
    // code and replace the mutable global String binding.
    for index in ["0", "99"] {
        let prefix = format!(
            "const stringify = String; \
             let segmenter = new Intl.Segmenter('en', {{granularity:'word'}}); \
             let segments = segmenter.segment('Hi!'); \
             let value = segments.containing({index}); "
        );
        for (tail, expected) in [
            ("value === undefined;", ValueKind::Boolean),
            ("stringify(value);", ValueKind::String),
            ("value + '';", ValueKind::String),
        ] {
            let source = format!("{prefix}{tail}");
            let script = lower_script(&source);
            let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
                panic!("String intrinsic must be captured before observable service calls");
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
