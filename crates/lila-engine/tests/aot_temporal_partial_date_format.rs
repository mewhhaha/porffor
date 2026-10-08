use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
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
            .unwrap_or_else(|error| panic!("partial-date formatting failed: {error}\n{source}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert!(observed.output_events.is_empty(), "{source}");
    }
}

#[test]
fn reference_iso_fields_survive_hidden_non_iso_calendar_annotations() {
    assert_modes(
        r#"
function same(actual, expected) {
  if (actual !== expected) throw new Error(actual + ' != ' + expected);
}
for (const calendar of [
  'iso8601', 'gregory', 'buddhist', 'roc', 'japanese', 'indian',
  'persian', 'coptic', 'ethiopic', 'ethioaa', 'islamic-civil',
  'islamic-tbla', 'islamic-umalqura', 'hebrew', 'chinese', 'dangi'
]) {
  const monthDay = new Temporal.PlainMonthDay(5, 2, calendar, 2000);
  const yearMonth = new Temporal.PlainYearMonth(2000, 5, calendar, 17);
  const iso = calendar === 'iso8601';
  same(monthDay.toString({calendarName: 'never'}), iso ? '05-02' : '2000-05-02');
  same(yearMonth.toString({calendarName: 'never'}), iso ? '2000-05' : '2000-05-17');
  const suffix = '[u-ca=' + calendar + ']';
  const critical = '[!u-ca=' + calendar + ']';
  same(monthDay.toString({calendarName: 'always'}), '2000-05-02' + suffix);
  same(yearMonth.toString({calendarName: 'always'}), '2000-05-17' + suffix);
  same(monthDay.toString({calendarName: 'critical'}), '2000-05-02' + critical);
  same(yearMonth.toString({calendarName: 'critical'}), '2000-05-17' + critical);
  const defaultMonthDay = iso ? '05-02' : '2000-05-02' + suffix;
  const defaultYearMonth = iso ? '2000-05' : '2000-05-17' + suffix;
  same(monthDay.toString({calendarName: 'auto'}), defaultMonthDay);
  same(yearMonth.toString({calendarName: 'auto'}), defaultYearMonth);
  same(monthDay.toString(), defaultMonthDay);
  same(yearMonth.toString(), defaultYearMonth);
  const ignored = {get calendarName() { throw new Error('toJSON option read'); }};
  same(monthDay.toJSON(ignored), defaultMonthDay);
  same(yearMonth.toJSON(ignored), defaultYearMonth);
}
same(new Temporal.PlainMonthDay(5, 2, 'gregory', -1).toString({calendarName: 'never'}), '-000001-05-02');
same(new Temporal.PlainYearMonth(10000, 5, 'gregory', 17).toString({calendarName: 'never'}), '+010000-05-17');
true;
"#,
    );
}

#[test]
fn borrowed_formatters_keep_reference_fields_and_observable_option_order() {
    assert_modes(
        r#"
function same(actual, expected) {
  if (actual !== expected) throw new Error(actual + ' != ' + expected);
}
const foreign = __lilaCreateRealm().global;
const families = [
  [Temporal.PlainMonthDay, foreign.Temporal.PlainMonthDay,
    new Temporal.PlainMonthDay(5, 2, 'gregory', 2000),
    new foreign.Temporal.PlainMonthDay(5, 2, 'gregory', 2000), '2000-05-02'],
  [Temporal.PlainYearMonth, foreign.Temporal.PlainYearMonth,
    new Temporal.PlainYearMonth(2000, 5, 'gregory', 17),
    new foreign.Temporal.PlainYearMonth(2000, 5, 'gregory', 17), '2000-05-17']
];
for (const [localType, foreignType, local, other, expected] of families) {
  const calls = [
    [localType.prototype.toString, other, RangeError, TypeError],
    [foreignType.prototype.toString, local, foreign.RangeError, foreign.TypeError]
  ];
  for (const [method, value, expectedRangeError, expectedTypeError] of calls) {
    let log = '';
    const options = { get calendarName() {
      log += 'get;';
      return { toString() { log += 'string;'; return 'never'; } };
    }};
    same(method.call(value, options), expected);
    same(log, 'get;string;');
    log = '';
    let failure;
    try { method.call({}, options); } catch (error) { failure = error; }
    if (!(failure instanceof expectedTypeError)) throw new Error('borrowed brand error Realm');
    same(log, '');
    failure = undefined;
    try { method.call(value, {calendarName: 'invalid'}); } catch (error) { failure = error; }
    if (!(failure instanceof expectedRangeError)) throw new Error('borrowed option error Realm');
    const marker = {};
    failure = undefined;
    try { method.call(value, { get calendarName() { throw marker; } }); }
    catch (error) { failure = error; }
    same(failure, marker);
  }
}
true;
"#,
    );
}
