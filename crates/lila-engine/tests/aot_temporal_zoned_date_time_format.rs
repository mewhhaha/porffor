use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_zoned_format(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("ZonedDateTime formatting must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn formats_exact_epoch_remainders_and_fixed_offsets() {
    assert_zoned_format(
        r#"
const cases = [
  [0n, 'UTC', '1970-01-01T00:00:00+00:00[UTC]'],
  [-1n, 'UTC', '1969-12-31T23:59:59.999999999+00:00[UTC]'],
  [1000000000123987500n, '+05:30', '2001-09-09T07:16:40.1239875+05:30[+05:30]'],
  [8640000000000000000000n, 'UTC', '+275760-09-13T00:00:00+00:00[UTC]'],
  [-8640000000000000000000n, 'UTC', '-271821-04-20T00:00:00+00:00[UTC]']
];
for (const [epoch, zone, expected] of cases) {
  const actual = new Temporal.ZonedDateTime(epoch, zone).toString();
  if (actual !== expected) throw new Error(actual + ' != ' + expected);
}
true;
"#,
    );
}

#[test]
fn rounds_epoch_before_formatting_and_preserves_receiver() {
    assert_zoned_format(
        r#"
const cases = [
  [-1n, 'trunc', '1969-12-31T23:59:59'],
  [-1n, 'ceil', '1970-01-01T00:00:00'],
  [-500000000n, 'halfEven', '1970-01-01T00:00:00'],
  [-1500000000n, 'halfEven', '1969-12-31T23:59:58'],
  [500000000n, 'halfEven', '1970-01-01T00:00:00'],
  [1500000000n, 'halfEven', '1970-01-01T00:00:02'],
  [86399999999999n, 'ceil', '1970-01-02T00:00:00']
];
for (const [epoch, roundingMode, expected] of cases) {
  const value = new Temporal.ZonedDateTime(epoch, 'UTC');
  const actual = value.toString({smallestUnit: 'second', roundingMode});
  if (actual !== expected + '+00:00[UTC]') throw new Error(actual);
  if (value.epochNanoseconds !== epoch) throw new Error('mutated epoch');
}
const value = new Temporal.ZonedDateTime(1000000000123987500n, 'UTC');
if (value.toString({smallestUnit: 'minute'}) !== '2001-09-09T01:46+00:00[UTC]') throw new Error('minute');
if (value.toString({fractionalSecondDigits: 6}) !== '2001-09-09T01:46:40.123987+00:00[UTC]') throw new Error('microsecond');
true;
"#,
    );
}

#[test]
fn formats_calendar_offset_and_time_zone_annotations() {
    assert_zoned_format(
        r#"
const value = new Temporal.ZonedDateTime(0n, '+01:00', 'gregory');
if (value.toString() !== '1970-01-01T01:00:00+01:00[+01:00][u-ca=gregory]') throw new Error('default');
if (value.toString({calendarName:'never', offset:'never', timeZoneName:'never'}) !== '1970-01-01T01:00:00') throw new Error('never');
if (value.toString({calendarName:'critical', timeZoneName:'critical'}) !== '1970-01-01T01:00:00+01:00[!+01:00][!u-ca=gregory]') throw new Error('critical');
if (new Temporal.ZonedDateTime(0n, 'UTC').toString({calendarName:'always'}) !== '1970-01-01T00:00:00+00:00[UTC][u-ca=iso8601]') throw new Error('always');
true;
"#,
    );
}

#[test]
fn reads_options_in_spec_order_before_rejecting_hour_precision() {
    assert_zoned_format(
        r#"
const log = [];
const options = {
  get calendarName() { log.push('calendarName'); return 'auto'; },
  get fractionalSecondDigits() { log.push('fractionalSecondDigits'); return 'auto'; },
  get offset() { log.push('offset'); return 'auto'; },
  get roundingMode() { log.push('roundingMode'); return 'trunc'; },
  get smallestUnit() { log.push('smallestUnit'); return 'hour'; },
  get timeZoneName() { log.push('timeZoneName'); return 'auto'; }
};
let received;
try { new Temporal.ZonedDateTime(0n, 'UTC').toString(options); } catch (error) { received = error; }
if (!(received instanceof RangeError)) throw new Error('hour precision');
if (log.join(',') !== 'calendarName,fractionalSecondDigits,offset,roundingMode,smallestUnit,timeZoneName') throw new Error(log.join(','));
log.length = 0;
try { Temporal.ZonedDateTime.prototype.toString.call({}, options); } catch (error) { received = error; }
if (!(received instanceof TypeError) || log.length !== 0) throw new Error('branding precedes options');
true;
"#,
    );
}

#[test]
fn shared_precision_preserves_plain_time_and_plain_date_time_formatting() {
    assert_zoned_format(
        r#"
const time = new Temporal.PlainTime(1, 2, 3, 456, 789, 123);
const datetime = new Temporal.PlainDateTime(2024, 2, 29, 1, 2, 3, 456, 789, 123);
for (const [options, expected] of [
  [{}, '01:02:03.456789123'],
  [{smallestUnit: 'minute'}, '01:02'],
  [{fractionalSecondDigits: 4}, '01:02:03.4567'],
  [{smallestUnit: 'millisecond', roundingMode: 'ceil'}, '01:02:03.457']
]) {
  if (time.toString(options) !== expected) throw new Error('PlainTime precision');
  if (datetime.toString(options) !== '2024-02-29T' + expected) throw new Error('PlainDateTime precision');
}
true;
"#,
    );
}

#[test]
fn unit_spelling_errors_precede_later_getters_but_unit_suitability_follows_them() {
    assert_zoned_format(
        r#"
const value = new Temporal.ZonedDateTime(0n, 'UTC');
for (const smallestUnit of ['bogus', 'HOUR', 'millis']) {
  let reads = 0;
  let received;
  try { value.toString({smallestUnit, get timeZoneName() { reads++; throw {}; }}); }
  catch (error) { received = error; }
  if (!(received instanceof RangeError) || reads !== 0) throw new Error('invalid spelling');
}
for (const smallestUnit of ['hour', 'day', 'auto']) {
  const marker = {};
  let received;
  try { value.toString({smallestUnit, get timeZoneName() { throw marker; }}); }
  catch (error) { received = error; }
  if (received !== marker) throw new Error('unit suitability');
}
true;
"#,
    );
}
