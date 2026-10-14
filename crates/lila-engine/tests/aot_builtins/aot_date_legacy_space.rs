use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, RealmBuilder, RunOptions,
};
use lila_intl::ConfiguredSystemTimeZone;

fn assert_legacy_date(source: &str, zone: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!(
            "{directive}function check(value, label) {{ if (!value) throw label; }}\n\
             {source}\nprint('legacy-date:ok');\ntrue;"
        );
        let realm = RealmBuilder::new()
            .with_system_time_zone(
                ConfiguredSystemTimeZone::resolve(zone).expect("control zone is admitted"),
            )
            .build();
        let observed = Engine::new(realm)
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(120_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("legacy Date parsing executes through Wasm: {error}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}\n{observed:?}",
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine("legacy-date:ok".into())],
        );
    }
}

#[test]
fn legacy_space_fields_and_offsets_produce_finite_epochs() {
    assert_legacy_date(
        r#"
var cases = [
  ['1997-03-08 11:19:20', 857819960000],
  ['1997-3-08 11:19:20', 857819960000],
  ['1997-3-8 11:19:20', 857819960000],
  ['+001997-3-8 11:19:20', 857819960000],
  ['+001997-03-8 11:19:20', 857819960000],
  ['1997-03-08 11:19', Date.UTC(1997, 2, 8, 11, 19)],
  ['1997-03-08 1:19', Date.UTC(1997, 2, 8, 1, 19)],
  ['1997-03-08 1:1', Date.UTC(1997, 2, 8, 1, 1)],
  ['1997-03-08 1:1:01', Date.UTC(1997, 2, 8, 1, 1, 1)],
  ['1997-03-08 1:1:1', Date.UTC(1997, 2, 8, 1, 1, 1)],
  ['1997-3-8 1:1:1.123', Date.UTC(1997, 2, 8, 1, 1, 1, 123)],
  ['1997-03-08 11:19:10-07', 857845150000],
  ['1997-03-08 11:19:10-0700', 857845150000],
  ['1997-03-08 11:19:10-07:00', 857845150000],
  ['1997-03-08 11:19:20+01:30', 857814560000],
  ['1997-03-08 11:19:20+0130', 857814560000],
  ['1997-03-08 11:19:20+01', 857816360000],
  ['1970-1-1 0:0Z', 0],
  ['-000001-3-8 1:2:3Z', Date.parse('-000001-03-08T01:02:03Z')],
  ['2000-2-29 24:0:0.000Z', Date.UTC(2000, 2, 1)],
  ['+275760-9-13 0:0Z', 8640000000000000],
  ['-271821-4-20 0:0Z', -8640000000000000],
  ['+275760-9-13 1:0+01', 8640000000000000],
  ['-271821-4-19 23:0-0100', -8640000000000000]
];
var detached = Date.parse;
for (var i = 0; i < cases.length; i++) {
  var source = cases[i][0].slice(0, 7) + cases[i][0].slice(7);
  var expected = cases[i][1];
  check(Number.isFinite(expected), 'finite expected ' + source);
  check(detached(source) === expected, 'detached parse ' + source);
  check(new Date(source).getTime() === expected, 'constructor parse ' + source);
}
check(1 / Date.parse('1970-1-1 0:0Z') === Infinity, 'positive clipped zero');
"#,
        "UTC",
    );
}

#[test]
fn legacy_space_fallback_rejects_malformed_fields_and_keeps_t_syntax_strict() {
    assert_legacy_date(
        r#"
var strings = [
  '1997-03-08T11:19:10-07', '1997-03-08T',
  '1997-3-8T11:19:20', '1997-03-8T11:19:20',
  '+001997-3-8T11:19:20', '+001997-3-08T11:19:20',
  '1997-03-08T1:19', '1997-03-08T1:1',
  '1997-03-08T1:1:01', '1997-03-08T1:1:1',
  '1997-03-08 11', '1997-03-08T11',
  // Both original staging .01 sides remain NaN under the existing, separately
  // controlled three-digit policy; their equality is not positive acceptance.
  '1997-03-08 1:1:1.01', '1997-03-08T01:01:01.01',
  '1970-1-1 0:0:0.1Z', '1970-1-1 0:0:0.00Z',
  '1970-1-1 0:0:0.0000Z', '1970-1-1 0:0.123Z',
  '1970-1-1 0:0:', '1970-1-1 0:', '1970-1-1 :0',
  '1970-1-1 ', '1970-1-1', '1970-1 0:0Z', '1970 0:0Z',
  '1970-1-1 000:0Z', '1970-001-1 0:0Z', '1970-1-001 0:0Z',
  '1970-1-1 0:000Z', '1970-1-1 0:0:000Z',
  '1970-0-1 0:0Z', '1970-13-1 0:0Z', '1970-1-0 0:0Z',
  '1970-1-32 0:0Z', '2001-2-29 0:0Z', '2000-2-30 24:0Z',
  '1970-1-1 25:0Z', '1970-1-1 24:1Z', '1970-1-1 24:0:1Z',
  '1970-1-1 24:0:0.001Z', '1970-1-1 0:60Z', '1970-1-1 0:0:60Z',
  '-000000-1-1 0:0Z', '+01970-1-1 0:0Z', '019700-1-1 0:0Z',
  '1970-1-1 0:0+0', '1970-1-1 0:0+001', '1970-1-1 0:0+00:',
  '1970-1-1 0:0+00:0', '1970-1-1 0:0+0060', '1970-1-1 0:0-24',
  '1970-1-1 0:0+00000', '1970-1-1 0:0Zjunk', '1970-1-1 0:0Z\u0000',
  '1970-1-1  0:0Z', '1970-1-1\t0:0Z', '1970-1-1\u00a00:0Z',
  '1970-\uff11-1 0:0Z', '1970\u20101-1 0:0Z',
  '+275760-9-13 0:0:0.001Z', '-271821-4-19 23:59:59.999Z',
  '+275760-9-13 24:0Z'
];
for (var i = 0; i < strings.length; i++) {
  check(Number.isNaN(Date.parse(strings[i])), 'invalid parse ' + strings[i]);
  check(Number.isNaN(new Date(strings[i]).getTime()), 'invalid constructor ' + strings[i]);
  check(Date.parse('1970-1-1 0:0Z') === 0, 'invalidity stays within one parse');
}
check(Date.parse('Thu, 01 Jan 1970 00:00:00 GMT') === 0, 'owned display fallback');
check(Date.parse('1970-01-01T00:00:00.123Z') === 123, 'strict ISO fraction control');
check(Date.parse('1970-01-01') === 0, 'date-only UTC control');
"#,
        "UTC",
    );
}

#[test]
fn legacy_space_parse_uses_realm_zone_once_after_coercion_and_propagates_errors() {
    let source = r#"
var expected = new Date(1970, 0, 1, 0, 0, 0).getTime();
check(Number.isFinite(expected) && expected !== 0, 'configured local epoch');
var foreign = __lilaCreateRealm().global;
var calls = [];
var value = { [Symbol.toPrimitive]: function(hint) {
  calls.push(hint);
  return '1970-1-1 0:0';
} };
check(Date.parse.call(null, value) === expected, 'root parse local zone');
check(foreign.Date.parse(value) === expected, 'created Realm parse local zone');
check(new foreign.Date(value).getTime() === expected, 'foreign constructor zone');
check(calls.join(',') === 'string,string,default', 'one conversion per operation');
check(Date.parse('1970-1-1 0:0Z') === 0, 'Z independent of zone');
check(foreign.Date.parse('1970-1-1 0:0+01') === -3600000, 'explicit offset independent of zone');
check(Date.parse('1970-01-01') === 0, 'strict date-only remains UTC');
var marker = {};
var caught;
try { Date.parse({ toString: function() { throw marker; } }); }
catch (error) { caught = error; }
check(caught === marker, 'abrupt conversion identity');
caught = undefined;
try { foreign.Date.parse(Symbol()); } catch (error) { caught = error; }
check(caught !== undefined && Object.getPrototypeOf(caught) === foreign.TypeError.prototype,
      'called Realm intrinsic ToString error');
if (expected === 18000000) {
  check(Date.parse('2021-3-14 2:30') === 1615707000000, 'named gap compatible inverse');
  check(Date.parse('2021-11-7 1:30') === 1636263000000, 'named fold earlier inverse');
  check(foreign.Date.parse('2021-11-7 1:30-05') === 1636266600000, 'explicit later fold');
} else {
  check(expected === -3600000, 'positive fixed zone control');
}
"#;
    for zone in ["+01:00", "America/New_York"] {
        assert_legacy_date(source, zone);
    }
}
