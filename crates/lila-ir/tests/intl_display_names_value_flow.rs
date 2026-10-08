use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ScriptIr, StatementIr, ValueKind};

fn lower_script(source: &str) -> ScriptIr {
    let program = lower(&parse(source, ParseOptions::script()).expect("value-flow source parses"));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

#[test]
fn optional_display_name_survives_a_local_binding_before_undefined_comparison() {
    for fallback in ["none", "code"] {
        let source = format!(
            "let names = new Intl.DisplayNames('en', {{type:'currency', fallback:'{fallback}'}}); \
             let value = names.of('ZZZ'); value;"
        );
        let script = lower_script(&source);
        let StatementIr::Lexical { init, .. } = &script.body.statements[1] else {
            panic!("the optional value must be retained by a lexical binding");
        };
        assert!(
            init.possible_kinds.contains(ValueKind::Undefined),
            "{source}"
        );
        assert!(init.possible_kinds.contains(ValueKind::String), "{source}");
        let StatementIr::Expression(value) = script.body.statements.last().unwrap() else {
            panic!("the final expression reads the retained optional value");
        };
        assert!(
            value.possible_kinds.contains(ValueKind::Undefined),
            "{source}"
        );
        assert!(value.possible_kinds.contains(ValueKind::String), "{source}");
        assert_eq!(script.result_kind(), ValueKind::Dynamic, "{source}");
    }
}

#[test]
fn optional_display_name_keeps_runtime_coercion_and_comparison_paths() {
    // Capture the intrinsic before the constructor/method may invoke user code.
    // Those observations correctly invalidate a later mutable global lookup.
    let prefix = "const stringify = String; let names = new Intl.DisplayNames('en', {type:'currency',fallback:'none'}); \
                  let value = names.of('ZZZ'); ";
    for (tail, expected) in [
        ("stringify(value);", ValueKind::String),
        ("value === undefined;", ValueKind::Boolean),
        ("value + '';", ValueKind::String),
    ] {
        let source = format!("{prefix}{tail}");
        let script = lower_script(&source);
        assert_eq!(script.result_kind(), expected, "{source}");
    }
}
