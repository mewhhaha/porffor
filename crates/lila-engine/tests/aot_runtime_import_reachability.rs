use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, ObservedNumber,
    RealmBuilder, RunOptions,
};

fn observe(source: &str) -> ObservedCompletion {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("runtime-planned script compiles and executes through Wasm AOT");
    assert!(observed.output_events.is_empty());
    observed.completion
}

fn number(value: f64) -> ObservedJsValue {
    ObservedJsValue::Number(ObservedNumber::from_f64(value))
}

fn text(value: &str) -> ObservedJsValue {
    ObservedJsValue::String(value.encode_utf16().collect())
}

#[test]
fn scalar_expressions_preserve_values_and_statement_list_completion() {
    for (source, value) in [
        ("", ObservedJsValue::Undefined),
        (";;", ObservedJsValue::Undefined),
        ("null;", ObservedJsValue::Null),
        ("true;", ObservedJsValue::Boolean(true)),
        ("'A😃B';", text("A😃B")),
        ("262;", number(262.0)),
        ("1 + 1;", number(2.0)),
        ("(7 - 2) * 3 / 2;", number(7.5)),
        ("5 % 2;", number(1.0)),
        ("-0;", number(-0.0)),
        ("+7;", number(7.0)),
        ("~7;", number(-8.0)),
        ("1 / 0;", number(f64::INFINITY)),
        ("0 / 0;", number(f64::NAN)),
        ("void (1 + 2);", ObservedJsValue::Undefined),
        ("delete 1;", ObservedJsValue::Boolean(true)),
        ("!0;", ObservedJsValue::Boolean(true)),
        ("1 < 2;", ObservedJsValue::Boolean(true)),
        ("false || 'text';", text("text")),
        ("null ?? 7;", number(7.0)),
        ("+(null ?? 7);", number(7.0)),
        ("!(null ?? 7);", ObservedJsValue::Boolean(false)),
        ("+(0 ?? 7);", number(0.0)),
        ("+(3 ?? 7);", number(3.0)),
        ("true ? 'yes' : 3;", text("yes")),
        ("(1, 2, 3);", number(3.0)),
        ("{ 7; { 8; } }", number(8.0)),
        ("7; ; {}", number(7.0)),
    ] {
        assert_eq!(
            observe(source),
            ObservedCompletion::Normal(value),
            "{source}"
        );
    }
}

#[test]
fn primitive_prototype_formatters_remain_observable_and_callable() {
    for source in [
        "(123).toLocaleString('en-US', {minimumFractionDigits:2});",
        "(123n)['toLocaleString']('en-US', {minimumFractionDigits:2});",
        "Object.getPrototypeOf(123).toLocaleString.call(123, 'en-US', {minimumFractionDigits:2});",
    ] {
        assert_eq!(
            observe(source),
            ObservedCompletion::Normal(text("123.00")),
            "{source}"
        );
    }
}

#[test]
fn runtime_side_effects_and_abrupt_completions_remain_in_the_product_path() {
    for (source, value) in [
        (
            "let count=0; const value={valueOf(){count++; return 7;}}; value + 1; count;",
            number(1.0),
        ),
        ("var answer=7; answer;", number(7.0)),
        ("function answer(){return 7;} answer();", number(7.0)),
    ] {
        assert_eq!(
            observe(source),
            ObservedCompletion::Normal(value),
            "{source}"
        );
    }
    assert_eq!(observe("throw 7;"), ObservedCompletion::Throw(number(7.0)));
}
