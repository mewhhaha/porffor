use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_character_constructors(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!(
            "{directive}function require(value, label) {{ if (!value) throw label; }}\n{source}"
        );
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| {
                panic!("String character constructors execute through Wasm: {error}")
            });
        assert!(
            matches!(&observed.completion, ObservedCompletion::Normal(_)),
            "{observed:?}"
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine("string-characters:ok".into())]
        );
    }
}

#[test]
fn boundaries_and_empty_calls_publish_exact_utf16_lengths() {
    assert_character_constructors(
        r#"
require(String.fromCodePoint() === '' && String.fromCharCode() === '', 'empty calls');
var text = String.fromCodePoint(-0, 0xffff, 0x10000, 0x10ffff, 0xd800, 0xdc00);
require(text === '\u0000\uffff\ud800\udc00\udbff\udfff\ud800\udc00', 'exact point units');
require(text.length === 8, 'point length excludes spare capacity');
var residues = String.fromCharCode(-0, 0xffff, 0x10000, -1.9, Infinity, NaN, 0xd800, 0xdc00);
require(residues === '\u0000\uffff\u0000\uffff\u0000\u0000\ud800\udc00', 'ToUint16 units');
require(residues.length === 8, 'one unit per char argument');
require(String.fromCodePoint.call(null, 65) === 'A', 'point receiver unused');
require(String.fromCharCode.call(Symbol(), 66) === 'B', 'char receiver unused');
print('string-characters:ok');
"#,
    );
}

#[test]
fn conversions_observe_live_left_to_right_changes_once() {
    assert_character_constructors(
        r#"
for (var make of [String.fromCharCode, String.fromCodePoint]) {
  var events = [], later = { valueOf: function() { throw 'old hook'; } };
  var first = { get [Symbol.toPrimitive]() {
    events.push('get first');
    return function(hint) {
      require(hint === 'number', 'number hint');
      events.push('call first');
      later.valueOf = function() { events.push('later'); return 66; };
      return 65;
    };
  } };
  var third = { valueOf: function() { events.push('third valueOf'); return {}; },
                toString: function() { events.push('third toString'); return '67'; } };
  require(make(first, later, third) === 'ABC', 'live conversions');
  require(events.join('|') === 'get first|call first|later|third valueOf|third toString', 'single ordered conversions');
}
print('string-characters:ok');
"#,
    );
}

#[test]
fn first_abrupt_conversion_or_range_error_stops_later_conversions() {
    assert_character_constructors(
        r#"
for (var make of [String.fromCharCode, String.fromCodePoint]) {
  for (var marker of [undefined, {}]) {
    var calls = 0, caught = false;
    try {
      make(65, { valueOf: function() { calls++; throw marker; } },
           { valueOf: function() { throw 'later conversion'; } });
    } catch (error) { require(error === marker, 'exact thrown value'); caught = true; }
    require(caught && calls === 1, 'abrupt conversion stops');
  }
  for (var value of [1n, Symbol()]) {
    var calls = 0, caught;
    try { make(65, value, { valueOf: function() { calls++; return 66; } }); }
    catch (error) { caught = error; }
    require(caught instanceof TypeError && calls === 0, 'ToNumber TypeError stops');
  }
}
for (var invalid of [NaN, Infinity, -Infinity, -1, 0x110000, 1.5]) {
  var calls = 0, caught;
  try { String.fromCodePoint(65, invalid, { valueOf: function() { calls++; return 66; } }); }
  catch (error) { caught = error; }
  require(caught instanceof RangeError && calls === 0, 'point range stops');
}
print('string-characters:ok');
"#,
    );
}

#[test]
fn unpublished_units_survive_reentrant_constructors_and_gc() {
    assert_character_constructors(
        r#"
for (var make of [String.fromCharCode, String.fromCodePoint]) {
  var events = [];
  var first = { valueOf: function() {
    events.push('first');
    require(String.fromCodePoint(0x1f600, 65) === '\ud83d\ude00A', 'inner point result');
    gc();
    return 65;
  } };
  var last = { valueOf: function() {
    events.push('last');
    require(String.fromCharCode(66, 0xd800) === 'B\ud800', 'inner char result');
    gc();
    return 67;
  } };
  require(make(first, 66, last) === 'ABC', 'outer unpublished prefix survives');
  require(events.join('|') === 'first|last', 'reentrant conversion order');
}
var grown = String.fromCodePoint(0x1f600, { valueOf: function() { gc(); return 0x10ffff; } });
require(grown === '\ud83d\ude00\udbff\udfff' && grown.length === 4, 'grown backing survives GC');
print('string-characters:ok');
"#,
    );
}

#[test]
fn borrowed_character_constructor_errors_use_the_defining_realm() {
    assert_character_constructors(
        r#"
var other = __scriptwCreateRealm().global;
for (var receiver of [null, String, other.String, {}]) {
  var caught;
  try { other.String.fromCodePoint.call(receiver, 0x110000); } catch (error) { caught = error; }
  require(caught instanceof other.RangeError && !(caught instanceof RangeError), 'foreign RangeError');
  for (var make of [other.String.fromCharCode, other.String.fromCodePoint]) {
    caught = undefined;
    try { make.call(receiver, 1n); } catch (error) { caught = error; }
    require(caught instanceof other.TypeError && !(caught instanceof TypeError), 'foreign TypeError');
  }
}
require(other.String.fromCodePoint(65, 0x1f600) === 'A\ud83d\ude00', 'borrowed normal result');
print('string-characters:ok');
"#,
    );
}

#[test]
fn ten_thousand_arguments_keep_all_bmp_and_supplementary_units() {
    assert_character_constructors(
        r#"
var points = [], chars = [];
for (var i = 0; i < 10000; i++) {
  points.push(i % 2 === 0 ? 65 : 0x1f600);
  chars.push(i % 2 === 0 ? 66 : 0xd800);
}
var pointText = String.fromCodePoint.apply(null, points);
var charText = String.fromCharCode.apply(null, chars);
require(pointText.length === 15000 && charText.length === 10000, 'full exact extents');
for (var i = 0; i < 5000; i++) {
  require(pointText.charCodeAt(3 * i) === 65 &&
          pointText.charCodeAt(3 * i + 1) === 0xd83d &&
          pointText.charCodeAt(3 * i + 2) === 0xde00, 'every supplementary group');
  require(charText.charCodeAt(2 * i) === 66 && charText.charCodeAt(2 * i + 1) === 0xd800, 'every char group');
}
var bmpText = String.fromCodePoint.apply(null, chars);
require(bmpText === charText && bmpText.length === 10000, 'all BMP exact capacity');
print('string-characters:ok');
"#,
    );
}
