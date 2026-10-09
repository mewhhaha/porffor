use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_zoned_round(source: &str) {
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
        .expect("ZonedDateTime rounding must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn rounds_local_clock_with_exact_negative_epoch_remainders() {
    assert_zoned_round(
        r#"
const cases = [
  [-1n, 'UTC', 'second', 'trunc', -1000000000n],
  [-1n, 'UTC', 'second', 'ceil', 0n],
  [-500000000n, 'UTC', 'second', 'halfEven', 0n],
  [-1500000000n, 'UTC', 'second', 'halfEven', -2000000000n],
  [-13849764999999999n, 'UTC', 'millisecond', 'halfExpand', -13849765000000000n],
  [-65261246399500000000n, 'UTC', 'second', 'trunc', -65261246400000000000n],
  [-65261246399500000000n, 'UTC', 'second', 'halfExpand', -65261246399000000000n],
  [0n, '+00:30', 'hour', 'halfExpand', 1800000000000n],
  [0n, '+00:30', 'hour', 'halfEven', -1800000000000n],
  [0n, '-00:30', 'hour', 'halfEven', 1800000000000n],
  [-1n, '+01:00', 'nanosecond', 'ceil', -1n]
];
for (const [epoch, zone, smallestUnit, roundingMode, expected] of cases) {
  const value = new Temporal.ZonedDateTime(epoch, zone);
  const result = value.round({smallestUnit, roundingMode});
  if (result.epochNanoseconds !== expected) throw new Error('rounded epoch');
  if (result === value || value.epochNanoseconds !== epoch) throw new Error('receiver changed');
  if (result.timeZoneId !== zone || result.calendarId !== 'iso8601') throw new Error('metadata');
}
true;
"#,
    );
}

#[test]
fn all_rounding_modes_use_local_midnight_as_the_origin() {
    assert_zoned_round(
        r#"
const modes = [
  ['ceil', 1], ['floor', 0], ['expand', 1], ['trunc', 0],
  ['halfCeil', 1], ['halfFloor', 0], ['halfExpand', 1], ['halfTrunc', 0], ['halfEven', 0]
];
for (const [roundingMode, up] of modes) {
  const epoch = -86399500000000n;
  const second = new Temporal.ZonedDateTime(epoch, 'UTC').round({smallestUnit:'second', roundingMode});
  const expectedSecond = up ? -86399000000000n : -86400000000000n;
  if (second.epochNanoseconds !== expectedSecond) throw new Error('second mode ' + roundingMode);
  const day = new Temporal.ZonedDateTime(-43200000000000n, 'UTC').round({smallestUnit:'day', roundingMode});
  const expectedDay = up ? 0n : -86400000000000n;
  if (day.epochNanoseconds !== expectedDay) throw new Error('day mode ' + roundingMode);
}
const fixed = new Temporal.ZonedDateTime(39600000000000n, '+01:00').round('day');
if (fixed.epochNanoseconds !== 82800000000000n) throw new Error('fixed-offset day');
true;
"#,
    );
}

#[test]
fn half_even_ties_exclude_higher_fields_for_odd_divisor_increments() {
    assert_zoned_round(
        r#"
const cases = [
  [1, 10, 0, 0, 0, 0, 'minute', 20, 3600000000000n],
  [1, 30, 0, 0, 0, 0, 'minute', 20, 6000000000000n],
  [0, 1, 10, 0, 0, 0, 'second', 20, 60000000000n],
  [0, 1, 30, 0, 0, 0, 'second', 20, 100000000000n],
  [0, 0, 1, 100, 0, 0, 'millisecond', 200, 1000000000n],
  [0, 0, 1, 300, 0, 0, 'millisecond', 200, 1400000000n],
  [0, 0, 0, 1, 100, 0, 'microsecond', 200, 1000000n],
  [0, 0, 0, 1, 300, 0, 'microsecond', 200, 1400000n],
  [0, 0, 0, 0, 1, 100, 'nanosecond', 200, 1000n],
  [0, 0, 0, 0, 1, 300, 'nanosecond', 200, 1400n],
  [0, 0, 0, 0, 1, 4, 'nanosecond', 8, 1000n],
  [0, 0, 0, 0, 1, 12, 'nanosecond', 8, 1016n],
  [23, 59, 59, 999, 999, 996, 'nanosecond', 8, 86399999999992n],
  [23, 59, 51, 0, 0, 0, 'second', 20, 86400000000000n]
];
for (const [hour, minute, second, millisecond, microsecond, nanosecond, smallestUnit, roundingIncrement, expected] of cases) {
  const options = {smallestUnit, roundingIncrement, roundingMode:'halfEven'};
  const time = new Temporal.PlainTime(hour, minute, second, millisecond, microsecond, nanosecond).round(options);
  const timeNs = BigInt(time.hour) * 3600000000000n + BigInt(time.minute) * 60000000000n +
    BigInt(time.second) * 1000000000n + BigInt(time.millisecond) * 1000000n +
    BigInt(time.microsecond) * 1000n + BigInt(time.nanosecond);
  if (timeNs !== expected % 86400000000000n) throw new Error('PlainTime parity ' + smallestUnit);
  const dateTime = new Temporal.PlainDateTime(1970, 1, 1, hour, minute, second, millisecond, microsecond, nanosecond).round(options);
  const dateTimeNs = BigInt(dateTime.day - 1) * 86400000000000n + BigInt(dateTime.hour) * 3600000000000n +
    BigInt(dateTime.minute) * 60000000000n + BigInt(dateTime.second) * 1000000000n +
    BigInt(dateTime.millisecond) * 1000000n + BigInt(dateTime.microsecond) * 1000n + BigInt(dateTime.nanosecond);
  if (dateTime.year !== 1970 || dateTime.month !== 1 || dateTimeNs !== expected)
    throw new Error('PlainDateTime parity ' + smallestUnit);
  const epoch = BigInt(hour) * 3600000000000n + BigInt(minute) * 60000000000n +
    BigInt(second) * 1000000000n + BigInt(millisecond) * 1000000n + BigInt(microsecond) * 1000n + BigInt(nanosecond);
  const zoned = new Temporal.ZonedDateTime(epoch, 'UTC').round(options);
  if (zoned.epochNanoseconds !== expected) throw new Error('ZonedDateTime parity ' + smallestUnit);
}
true;
"#,
    );
}

#[test]
fn nanosecond_noop_clones_without_converting_outlying_local_dates() {
    assert_zoned_round(
        r#"
const limit = 8640000000000000000000n;
for (const epoch of [-limit, -1n, 0n, limit]) {
  for (const zone of ['UTC', '-23:59', '+23:59']) {
    const value = new Temporal.ZonedDateTime(epoch, zone, 'gregory');
    Object.defineProperty(value, 'constructor', {get() { throw new Error('constructor read'); }});
    for (const roundTo of ['nanoseconds', {smallestUnit:'nanosecond', roundingIncrement:1}]) {
      const result = value.round(roundTo);
      if (result === value || result.epochNanoseconds !== epoch) throw new Error('clone epoch');
      if (Object.getPrototypeOf(result) !== Temporal.ZonedDateTime.prototype) throw new Error('prototype');
      if (result.timeZoneId !== zone || result.calendarId !== 'gregory') throw new Error('clone metadata');
    }
  }
}
class Subclass extends Temporal.ZonedDateTime {}
const subclass = new Subclass(500000000n, 'UTC');
const result = subclass.round('second');
if (Object.getPrototypeOf(result) !== Temporal.ZonedDateTime.prototype) throw new Error('subclass retained');
true;
"#,
    );
}

#[test]
fn reads_and_converts_options_before_unit_specific_validation() {
    assert_zoned_round(
        r#"
const value = new Temporal.ZonedDateTime(1n, 'UTC');
let log = '';
const options = {
  get roundingIncrement() { log += 'I'; return { valueOf() { log += 'i'; return 25; }}; },
  get roundingMode() { log += 'M'; return { toString() { log += 'm'; return 'expand'; }}; },
  get smallestUnit() { log += 'U'; return { toString() { log += 'u'; return 'hour'; }}; }
};
let caught = false;
try { value.round(options); } catch (error) { caught = error instanceof RangeError; }
if (!caught || log !== 'IiMmUu') throw new Error('options order ' + log);
const marker = {};
log = '';
try {
  value.round({
    get roundingIncrement() { log += 'I'; return 1; },
    get roundingMode() { log += 'M'; throw marker; },
    get smallestUnit() { log += 'U'; return 'second'; }
  });
  throw new Error('missing throw');
} catch (error) { if (error !== marker || log !== 'IM') throw new Error('abrupt order'); }
log = '';
caught = false;
try { Temporal.ZonedDateTime.prototype.round.call({}, options); }
catch (error) { caught = error instanceof TypeError; }
if (!caught || log !== '') throw new Error('options before brand');
const shorthand = new Temporal.ZonedDateTime(1n, 'UTC');
Object.defineProperty(Object.prototype, 'roundingIncrement', {get() { throw marker; }, configurable:true});
Object.defineProperty(Object.prototype, 'roundingMode', {get() { throw marker; }, configurable:true});
const rounded = shorthand.round('second');
delete Object.prototype.roundingIncrement;
delete Object.prototype.roundingMode;
if (rounded.epochNanoseconds !== 0n) throw new Error('shorthand');
true;
"#,
    );
}

#[test]
fn validates_units_and_increments_and_truncates_fractional_increments() {
    assert_zoned_round(
        r#"
const value = new Temporal.ZonedDateTime(123456789n, 'UTC');
const invalid = [
  {}, {smallestUnit:'auto'}, {smallestUnit:'year'}, {smallestUnit:'week'},
  {smallestUnit:'day', roundingIncrement:2},
  {smallestUnit:'hour', roundingIncrement:24}, {smallestUnit:'hour', roundingIncrement:7},
  {smallestUnit:'minute', roundingIncrement:60}, {smallestUnit:'second', roundingIncrement:60},
  {smallestUnit:'millisecond', roundingIncrement:1000},
  {smallestUnit:'microsecond', roundingIncrement:1000},
  {smallestUnit:'nanosecond', roundingIncrement:1000},
  {smallestUnit:'nanosecond', roundingIncrement:0},
  {smallestUnit:'nanosecond', roundingIncrement:NaN},
  {smallestUnit:'nanosecond', roundingIncrement:Infinity}
];
for (const options of invalid) {
  let caught = false;
  try { value.round(options); } catch (error) { caught = error instanceof RangeError; }
  if (!caught) throw new Error('invalid unit or increment');
}
for (const options of [undefined, null, true, 1, 1n, Symbol()]) {
  let caught = false;
  try { value.round(options); } catch (error) { caught = error instanceof TypeError; }
  if (!caught) throw new Error('invalid roundTo type');
}
for (const smallestUnit of ['day', 'hour', 'minute', 'second', 'millisecond', 'microsecond', 'nanosecond']) {
  const singular = value.round(smallestUnit);
  const plural = value.round(smallestUnit + 's');
  const fractional = value.round({smallestUnit, roundingIncrement:1.9});
  if (singular.epochNanoseconds !== plural.epochNanoseconds || singular.epochNanoseconds !== fractional.epochNanoseconds)
    throw new Error('unit normalization');
}
const rounded = value.round({smallestUnit:'nanosecond', roundingIncrement:2.9});
if (rounded.epochNanoseconds !== 123456790n) throw new Error('increment truncation');
true;
"#,
    );
}

#[test]
fn day_rounding_validates_both_midnights_even_when_rounding_down() {
    assert_zoned_round(
        r#"
const limit = 8640000000000000000000n;
const cases = [[-limit, '-01:00'], [-limit, '+01:00'], [limit, '-01:00'], [limit, 'UTC'], [limit, '+01:00']];
for (const [epoch, zone] of cases) {
  for (const roundingMode of ['floor', 'ceil']) {
    let caught = false;
    try { new Temporal.ZonedDateTime(epoch, zone).round({smallestUnit:'day', roundingMode}); }
    catch (error) { caught = error instanceof RangeError; }
    if (!caught) throw new Error('day endpoint range');
  }
}
const minimum = new Temporal.ZonedDateTime(-limit, 'UTC').round('day');
if (minimum.epochNanoseconds !== -limit) throw new Error('minimum UTC day');
const maximum = new Temporal.ZonedDateTime(limit - 1n, 'UTC').round('day');
if (maximum.epochNanoseconds !== limit) throw new Error('last complete day');
true;
"#,
    );
}

#[test]
fn subday_rounding_checks_original_iso_day_before_offset_and_epoch_range() {
    assert_zoned_round(
        r#"
const limit = 8640000000000000000000n;
const cases = [
  [limit, '+23:59', 'minute', 10, 'ceil'],
  [-limit, '-01:00', 'second', 1, 'trunc'],
  [limit, '+00:30', 'hour', 1, 'halfExpand'],
  [-limit, '+00:30', 'hour', 1, 'floor']
];
for (const [epoch, zone, smallestUnit, roundingIncrement, roundingMode] of cases) {
  let caught = false;
  try { new Temporal.ZonedDateTime(epoch, zone).round({smallestUnit, roundingIncrement, roundingMode}); }
  catch (error) { caught = error instanceof RangeError; }
  if (!caught) throw new Error('rounded date or epoch range');
}
const maximum = new Temporal.ZonedDateTime(limit, '+23:59').round({smallestUnit:'minute', roundingMode:'floor'});
if (maximum.epochNanoseconds !== limit) throw new Error('valid maximum minute');
const minimum = new Temporal.ZonedDateTime(-limit, '+01:00').round('hour');
if (minimum.epochNanoseconds !== -limit) throw new Error('valid minimum hour');
true;
"#,
    );
}
