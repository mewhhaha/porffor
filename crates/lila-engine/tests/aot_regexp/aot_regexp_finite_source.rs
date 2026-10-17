use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

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
    let mut rows = vec![
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
        (vec![91, 47, 93], vec![91, 47, 93], vec![47]),
        (vec![91, 92, 47, 93], vec![91, 92, 47, 93], vec![47]),
        (vec![91, 92, 92, 47, 93], vec![91, 92, 92, 47, 93], vec![92]),
        (
            vec![92, 91, 47, 92, 93],
            vec![92, 91, 92, 47, 92, 93],
            vec![91, 47, 93],
        ),
    ];
    for (unit, suffix) in [(10, "n"), (13, "r"), (0x2028, "u2028"), (0x2029, "u2029")] {
        for prefix_length in 1..=4 {
            let mut pattern = vec![92; prefix_length];
            pattern.push(unit);
            let mut expected = vec![92; prefix_length + 1 - prefix_length % 2];
            expected.extend(suffix.encode_utf16());
            let mut input = vec![92; prefix_length / 2];
            input.push(unit);
            rows.push((pattern.clone(), expected.clone(), input));
            pattern.insert(0, 91);
            pattern.push(93);
            expected.insert(0, 91);
            expected.push(93);
            rows.push((pattern, expected, vec![unit]));
        }
    }
    let rows = rows
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
  if (regexp.toString() !== '/' + row[1] + '/') throw 'toString units';
  if (!regexp.test(row[2])) throw 'original matcher';
  var repeated = eval('/' + regexp.source + '/');
  if (repeated.source !== row[1]) throw 'roundtrip source units';
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
    // Historical selector retained; valid native misses no longer report eval debt.
    assert_modes(
        r#"
var pattern = 'z' + Math.random(), trace = '';
var patternArgument = { toString() { trace += 'P'; return pattern; } };
var flagsArgument = { toString() { trace += 'F'; return ''; } };
function caller() {
    var local = {};
    var result = eval('({regexp: /' + new RegExp(patternArgument, flagsArgument).source + '/, caller: local})');
    return result.caller === local && result.regexp.source === pattern && result.regexp.test(pattern);
}
caller() && trace === 'PF';
"#,
    );
    assert_modes(
        r#"
var regexp = /a/, pattern, trace = '';
Object.defineProperty(regexp, 'source', { get() {
    trace += 'G'; pattern = 'z' + Math.random(); return pattern;
} });
function caller() {
    var local = {};
    var result = eval('({regexp: /' + regexp.source + '/, caller: local})');
    return result.caller === local && result.regexp.source === pattern && result.regexp.test(pattern);
}
caller() && trace === 'G';
"#,
    );
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

#[test]
fn literal_source_keeps_character_class_slashes_and_backslash_spellings() {
    assert_modes(
        r#"
var rows = [
  [/\\n/, '\\\\n', '\\n'],
  [/\\\n/, '\\\\\\n', '\\\n'],
  [/\\u2028/, '\\\\u2028', '\\u2028'],
  [/[/]/, '[/]', '/'],
  [/[\/]/, '[\\/]', '/'],
  [/[\\/]/, '[\\\\/]', '\\'],
  [/\[\/\]/, '\\[\\/\\]', '[/]']
];
for (var row of rows) {
  if (row[0].source !== row[1]) throw 'literal source units';
  if (row[0].toString() !== '/' + row[1] + '/') throw 'literal toString units';
  if (!row[0].test(row[2])) throw 'literal matcher';
  var repeated = eval('/' + row[0].source + '/');
  if (repeated.source !== row[1] || !repeated.test(row[2])) throw 'literal roundtrip';
}
true;
"#,
    );
}

#[test]
fn source_getter_preserves_unicode_set_slashes_after_nested_class_closes() {
    assert_modes(
        r#"
var nested = /[[a]--[b]]\//v;
if (nested.source !== '[[a]--[b]]\\/') throw 'nested class source';
if (!nested.test('a/') || nested.test('b/')) throw 'nested class matcher';
var afterInnerClose = /[[a]\/]x/v;
if (afterInnerClose.source !== '[[a]\\/]x') throw 'slash after nested close';
if (!afterInnerClose.test('/x') || !afterInnerClose.test('ax')) throw 'union matcher';
var stringSet = /[\q{a\/b}]/v;
if (stringSet.source !== '[\\q{a\\/b}]') throw 'class string source';
if (!stringSet.test('a/b') || stringSet.test('axb')) throw 'class string matcher';
true;
"#,
    );
}
