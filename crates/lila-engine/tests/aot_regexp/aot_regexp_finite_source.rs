use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};
use lila_ir::DynamicSourceRuntimeOperation;

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("RegExp source hints failed: {error}\n{source}"));
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert!(observation.output_events.is_empty(), "{source}");
    }
}

fn utf16_literal(units: &[u16]) -> String {
    format!(
        "'{}'",
        units
            .iter()
            .map(|unit| format!("\\u{unit:04x}"))
            .collect::<String>()
    )
}

#[test]
fn source_roundtrips_keep_slash_parity_line_terminators_and_raw_surrogates() {
    let rows = [
        (vec![], vec![40, 63, 58, 41], vec![]),
        (vec![47], vec![92, 47], vec![47]),
        (vec![92, 47], vec![92, 47], vec![47]),
        (vec![92, 92, 47], vec![92, 92, 92, 47], vec![92, 47]),
        (vec![92, 92, 92, 47], vec![92, 92, 92, 47], vec![92, 47]),
        (vec![10], vec![92, 110], vec![10]),
        (vec![13], vec![92, 114], vec![13]),
        (
            vec![0x2028],
            "\\u2028".encode_utf16().collect(),
            vec![0x2028],
        ),
        (
            vec![0x2029],
            "\\u2029".encode_utf16().collect(),
            vec![0x2029],
        ),
        (vec![0xd800], vec![0xd800], vec![0xd800]),
        (vec![0xdc00], vec![0xdc00], vec![0xdc00]),
    ]
    .into_iter()
    .map(|(pattern, expected, input)| {
        format!(
            "[{}, {}, {}]",
            utf16_literal(&pattern),
            utf16_literal(&expected),
            utf16_literal(&input)
        )
    })
    .collect::<Vec<_>>()
    .join(",");
    assert_modes(&format!(
        r#"
var Constructor = RegExp;
var rows = [{rows}];
for (var row of rows) {{
  var regexp = new Constructor(row[0]);
  if (regexp.source !== row[1]) throw 'source units';
  var repeated = eval('/' + regexp.source + '/');
  if (!repeated.test(row[2])) throw 'roundtrip matcher';
}}
if (!eval('/' + /\//.source + '/').test('/')) throw 'literal slash';
true;
"#
    ));
}

#[test]
fn source_hints_preserve_live_getters_constructor_replacement_and_eval_replacement() {
    assert_modes(
        r#"
var trace = '';
var regexp = /a/;
Object.defineProperty(regexp, 'source', { get() { trace += 'G'; return 'a'; } });
if (!eval('/' + regexp.source + '/').test('a')) throw 'live getter result';
function replacement(pattern) { trace += 'C'; return { source: pattern }; }
var Constructor = globalThis.chooseConstructor ? RegExp : replacement;
if (!eval('/' + new Constructor('a').source + '/').test('a')) throw 'replacement constructor';
globalThis.eval = function(source) { trace += 'E'; return source; };
if (eval('/' + /a/.source + '/') !== '/a/') throw 'replacement eval';
trace === 'GCE';
"#,
    );
}

#[test]
fn source_hints_preserve_abrupt_getters_and_pattern_conversion() {
    assert_modes(
        r#"
var marker = {}, trace = '';
var regexp = /a/;
Object.defineProperty(regexp, 'source', { get() { trace += 'G'; throw marker; } });
try { eval('/' + regexp.source + '/'); throw 'missing getter throw'; }
catch (error) { if (error !== marker) throw error; }
var pattern = { toString() { trace += 'P'; throw marker; } };
var flags = { toString() { trace += 'wrong'; return ''; } };
try { eval('/' + new RegExp(pattern, flags).source + '/'); throw 'missing conversion throw'; }
catch (error) { if (error !== marker) throw error; }
trace === 'GP';
"#,
    );
}

#[test]
fn unprepared_patterns_and_changed_getter_results_remain_typed_eval_limitations() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        for source in [
            "var pattern = 'z' + Math.random(); eval('/' + new RegExp(pattern).source + '/');",
            "var regexp = /a/; Object.defineProperty(regexp, 'source', { get() { return 'z' + Math.random(); } }); eval('/' + regexp.source + '/');",
        ] {
            let error = Engine::new(RealmBuilder::new().build()).observe_script(
                &format!("{directive}{source}"),
                CompileOptions { host_surface_policy: HostSurfacePolicy::Test262, ..CompileOptions::default() },
                RunOptions { backend: ExecutionBackend::WasmAot, timeout_ms: Some(60_000), ..RunOptions::default() },
            ).expect_err("a finite source hint cannot admit an unmatched runtime string");
            assert_eq!(error.runtime_dynamic_source_operations(), vec![DynamicSourceRuntimeOperation::Eval], "{error}");
            assert!(error.parse_diagnostic().is_none(), "{error}");
            assert!(error.ir_diagnostic().is_none(), "{error}");
        }
    }
}

#[test]
fn full_primary_source_capacity_keeps_unselected_regexp_getter_effects() {
    let texts = (0..256)
        .map(|index| format!("'{index};'"))
        .collect::<Vec<_>>()
        .join(",");
    assert_modes(&format!(
        r#"
var texts = [{texts}], calls = 0;
var regexp = /hint/;
Object.defineProperty(regexp, 'source', {{ get() {{ calls++; return 'not-prepared'; }} }});
var result = eval(...texts, '/' + regexp.source + '/');
result === 0 && calls === 1;
"#
    ));
}
