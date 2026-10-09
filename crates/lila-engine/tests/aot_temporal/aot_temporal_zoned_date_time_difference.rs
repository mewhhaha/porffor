use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_difference(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Temporal differences must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn offset_syntax_precedes_later_fields_and_fractional_offsets_remain_exact() {
    assert_difference(
        r#"
function range(action) { let caught; try { action(); } catch (error) { caught = error; }
  if (!(caught instanceof RangeError)) throw new Error('offset rejection'); }
let yearReads = 0;
for (const offset of ['--00:00', '00:00', '+00:00.0', '+00:00:00.0000000000', '+00:0000', '+24:00', '+00:60', '+00:00:60']) {
  range(() => Temporal.ZonedDateTime.from({offset, day: 1, month: 1, timeZone: 'UTC', get year() { yearReads++; throw {}; }}));
}
if (yearReads !== 0) throw new Error('late offset parsing');
const marker = {};
let caught;
try { Temporal.ZonedDateTime.from({offset: '+04:30', day: 1, month: 1, timeZone: 'UTC', get year() { throw marker; }}); }
catch (error) { caught = error; }
if (caught !== marker) throw new Error('offset matching too early');
for (const [offset, expected] of [['+00:00:00.000000001', -1n], ['-00:00:00.000000001', 1n], ['+000030,123456789', -30123456789n]]) {
  const bag = {year: 1970, month: 1, day: 1, timeZone: 'UTC', offset};
  if (Temporal.ZonedDateTime.from(bag, {offset: 'use'}).epochNanoseconds !== expected) throw new Error('offset precision');
  if (Temporal.ZonedDateTime.from(bag, {offset: 'prefer'}).epochNanoseconds !== 0n) throw new Error('prefer mismatch');
  range(() => Temporal.ZonedDateTime.from(bag));
}
const exact = Temporal.ZonedDateTime.from({year: 1970, month: 1, day: 1, timeZone: '+01:30', offset: '+01:30:00.000000000'});
if (exact.epochNanoseconds !== -5400000000000n) throw new Error('exact matching');
true;
"#,
    );
}

#[test]
fn wide_time_differences_keep_integer_precision_and_numeric_field_normalization() {
    assert_difference(
        r#"
const origin = new Temporal.ZonedDateTime(0n, 'UTC');
for (const sign of [1n, -1n]) {
  const endpoint = new Temporal.ZonedDateTime(sign * 18446744073709551616n, 'UTC');
  const forward = origin.until(endpoint, {largestUnit: 'microseconds'});
  const backward = origin.since(endpoint, {largestUnit: 'microseconds'});
  const expected = sign === 1n ? 18446744073709552 : -18446744073709552;
  if (forward.microseconds !== expected || backward.microseconds !== -expected) throw new Error('wrapped duration');
  if (forward.nanoseconds !== (sign === 1n ? 616 : -616) || backward.nanoseconds !== -forward.nanoseconds) throw new Error('subsecond precision');
  for (const key of ['years', 'months', 'weeks', 'days', 'hours', 'minutes', 'seconds', 'milliseconds']) {
    if (forward[key] !== 0 || backward[key] !== 0) throw new Error('unexpected large unit ' + key);
  }
  if (Temporal.Duration.compare(forward.add({microseconds: sign === 1n ? 1 : -1}), forward) !== 0) throw new Error('field normalization');
  const plain = origin.toPlainDateTime().until(endpoint.toPlainDateTime(), {largestUnit: 'microseconds'});
  if (plain.microseconds !== expected || plain.nanoseconds !== forward.nanoseconds) throw new Error('plain wide difference');
}
if (origin.since(new Temporal.ZonedDateTime(18446744073709551616n, 'UTC'), {largestUnit: 'microseconds'}).toString() !== '-PT18446744073.709552616S') throw new Error('normalized formatting');
true;
"#,
    );
}

#[test]
fn zoned_time_nudge_validates_the_adjacent_day_before_rounding() {
    assert_difference(
        r#"
const limit = 8640000000000000000000n;
const hour = 3600000000000n;
const day = 24n * hour;
function checkHour(value, expected) {
  if (value.hours !== expected) throw new Error('hour difference');
  for (const key of ['years', 'months', 'weeks', 'days', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds']) {
    if (value[key] !== 0) throw new Error('unexpected duration field ' + key);
  }
}
for (const [start, end, sign] of [[limit - hour, limit, 1], [-limit + hour, -limit, -1]]) {
  const origin = new Temporal.ZonedDateTime(start, 'UTC');
  const destination = new Temporal.ZonedDateTime(end, 'UTC');
  for (const method of ['until', 'since']) {
    let received;
    try { origin[method](destination, {largestUnit:'day', smallestUnit:'hour'}); }
    catch (error) { received = error; }
    if (!(received instanceof RangeError)) throw new Error('unvalidated adjacent day ' + method);
    const expected = method === 'until' ? sign : -sign;
    checkHour(origin[method](destination, {largestUnit:'hour', smallestUnit:'hour'}), expected);
    checkHour(origin[method](destination, {largestUnit:'day'}), expected);
    const inwardOrigin = new Temporal.ZonedDateTime(start - BigInt(sign) * day, 'UTC');
    const inwardDestination = new Temporal.ZonedDateTime(end - BigInt(sign) * day, 'UTC');
    checkHour(inwardOrigin[method](inwardDestination, {largestUnit:'day', smallestUnit:'hour'}), expected);
  }
  if (origin.epochNanoseconds !== start || destination.epochNanoseconds !== end) throw new Error('mutated endpoint');
}
true;
"#,
    );
}

#[test]
fn day_and_time_rounding_uses_the_plain_or_zoned_duration_origin() {
    assert_difference(
        r#"
function checkParts(value, days, hours) {
  if (value.days !== days || value.hours !== hours) throw new Error('day and hour rounding');
  for (const key of ['years', 'months', 'weeks', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds']) {
    if (value[key] !== 0) throw new Error('unexpected rounded field ' + key);
  }
}
const options = {largestUnit:'day', smallestUnit:'hour', roundingIncrement:8, roundingMode:'halfEven'};
const plainStart = new Temporal.PlainDateTime(2000, 1, 1);
const plainEnd = new Temporal.PlainDateTime(2000, 1, 2, 4);
for (const [origin, destination, sign] of [[plainStart, plainEnd, 1], [plainEnd, plainStart, -1]]) {
  checkParts(origin.until(destination, options), sign, sign * 8);
  checkParts(origin.since(destination, options), -sign, -sign * 8);
}
for (const zone of ['UTC', '+05:30', '-04:00']) {
  const zonedStart = Temporal.ZonedDateTime.from('2000-01-01T00:00[' + zone + ']');
  const zonedEnd = Temporal.ZonedDateTime.from('2000-01-02T04:00[' + zone + ']');
  for (const [origin, destination, sign] of [[zonedStart, zonedEnd, 1], [zonedEnd, zonedStart, -1]]) {
    checkParts(origin.until(destination, options), sign, 0);
    checkParts(origin.since(destination, options), -sign, 0);
  }
}
true;
"#,
    );
}

#[test]
fn plain_bubbling_validates_the_calendar_candidate_without_bounding_the_nudged_epoch() {
    assert_difference(
        r#"
const options = {largestUnit:'month', smallestUnit:'day', roundingIncrement:365, roundingMode:'expand'};
for (const year of [2020, 275760]) {
  const origin = new Temporal.PlainDateTime(year, 1, 1);
  const destination = new Temporal.PlainDateTime(year, 1, 2);
  const result = origin.until(destination, options);
  if (result.months !== 1) throw new Error('month candidate');
  for (const key of ['years', 'weeks', 'days', 'hours', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds']) {
    if (result[key] !== 0) throw new Error('unbalanced calendar field ' + key);
  }
}
const zonedOrigin = Temporal.ZonedDateTime.from('+275760-01-01T00:00[UTC]');
const zonedDestination = Temporal.ZonedDateTime.from('+275760-01-02T00:00[UTC]');
let received;
try { zonedOrigin.until(zonedDestination, options); } catch (error) { received = error; }
if (!(received instanceof RangeError)) throw new Error('zoned calendar bracket must remain bounded');
true;
"#,
    );
}

#[test]
fn plain_calendar_candidates_use_iso_date_bounds_without_constructing_date_times() {
    assert_difference(
        r#"
const options = {largestUnit:'week', smallestUnit:'week', roundingMode:'trunc'};
const origin = new Temporal.PlainDateTime(-271821, 4, 26);
const destination = new Temporal.PlainDateTime(-271821, 4, 25);
for (const method of ['until', 'since']) {
  const result = origin[method](destination, options);
  for (const key of ['years', 'months', 'weeks', 'days', 'hours', 'minutes', 'seconds', 'milliseconds', 'microseconds', 'nanoseconds']) {
    if (!Object.is(result[key], 0)) throw new Error('plain lower-date candidate ' + method + ' ' + key);
  }
}
const zonedOrigin = Temporal.ZonedDateTime.from('-271821-04-26T00:00[UTC]');
const zonedDestination = Temporal.ZonedDateTime.from('-271821-04-25T00:00[UTC]');
for (const method of ['until', 'since']) {
  let received;
  try { zonedOrigin[method](zonedDestination, options); } catch (error) { received = error; }
  if (!(received instanceof RangeError)) throw new Error('zoned lower-date candidate ' + method);
}
true;
"#,
    );
}

#[test]
fn time_units_compare_epochs_across_zones_without_calendar_conversion_limits() {
    assert_difference(
        r#"
const zero = new Temporal.ZonedDateTime(0n, 'UTC');
if (zero.since('1970-01-01T00:00[+01:00]').hours !== 1) throw new Error('wall clock difference');
if (zero.since('1970-01-01T00:00Z[+01:00]').hours !== 0) throw new Error('Z offset');
const maximum = new Temporal.ZonedDateTime(8640000000000000000000n, '+23:59');
const minimum = new Temporal.ZonedDateTime(-8640000000000000000000n, '-23:59');
if (minimum.until(maximum).hours !== 4800000000 || maximum.until(minimum).hours !== -4800000000) throw new Error('epoch range');
if (maximum.since(minimum).hours !== 4800000000) throw new Error('since epoch range');
let trace = '';
let caught;
try { zero.until(new Temporal.ZonedDateTime(0n, '+01:00'), {
 get largestUnit() { trace += 'L'; return 'day'; },
 get roundingIncrement() { trace += 'I'; return 1; },
 get roundingMode() { trace += 'R'; return 'trunc'; },
 get smallestUnit() { trace += 'S'; return 'nanosecond'; }
}); } catch (error) { caught = error; }
if (trace !== 'LIRS' || !(caught instanceof RangeError)) throw new Error('zone check ordering');
true;
"#,
    );
}

#[test]
fn rounding_bubbles_months_and_carried_days_in_both_directions() {
    assert_difference(
        r#"
for (const make of [source => Temporal.ZonedDateTime.from(source + '[UTC]'), source => Temporal.PlainDateTime.from(source)]) {
  const start = make('2022-01-01T00:00');
  const end = make('2023-12-25T00:00');
  const options = {largestUnit: 'year', smallestUnit: 'month', roundingMode: 'expand'};
  if (start.until(end, options).years !== 2 || start.since(end, options).years !== -2) throw new Error('month bubble');
  const epoch = make('1970-01-01T00:00');
  const last = make('1971-12-31T23:59:59.999999999');
  const rounded = epoch.until(last, {largestUnit: 'year', smallestUnit: 'microsecond', roundingMode: 'expand'});
  if (rounded.years !== 2 || rounded.months !== 0 || rounded.days !== 0 || rounded.hours !== 0) throw new Error('day bubble');
  const roundedSince = epoch.since(last, {largestUnit: 'year', smallestUnit: 'microsecond', roundingMode: 'expand'});
  if (roundedSince.years !== -2 || roundedSince.months !== 0 || roundedSince.days !== 0) throw new Error('since day bubble');
  const multi = make('2019-01-01T00:00').until(make('2021-12-01T00:00'), {largestUnit: 'year', smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'expand'});
  if (multi.years !== 3 || multi.months !== 0) throw new Error('retain years before nudge');
}
true;
"#,
    );
}

#[test]
fn calendar_nudge_validates_unselected_brackets_and_accepts_exact_epoch_bounds() {
    assert_difference(
        r#"
const earlier = new Temporal.ZonedDateTime(0n, 'UTC');
const later = new Temporal.ZonedDateTime(5n, 'UTC');
for (const pair of [[later, earlier], [earlier, later]]) {
  let caught;
  try { pair[0].since(pair[1], {smallestUnit: 'day', roundingIncrement: 100000001}); }
  catch (error) { caught = error; }
  if (!(caught instanceof RangeError)) throw new Error('unchecked bracket');
}
if (later.since(earlier, {smallestUnit: 'day', roundingIncrement: 100000000, roundingMode: 'expand'}).days !== 100000000) throw new Error('valid minimum bracket');
if (earlier.since(later, {smallestUnit: 'day', roundingIncrement: 100000000, roundingMode: 'expand'}).days !== -100000000) throw new Error('valid maximum bracket');
const minimum = new Temporal.ZonedDateTime(-8640000000000000000000n, 'UTC');
if (minimum.until(minimum, {smallestUnit: 'day', roundingIncrement: 100000001}).days !== 0) throw new Error('equal endpoints');
true;
"#,
    );
}

#[test]
fn half_even_uses_whole_duration_parity_and_exact_subsecond_midpoints() {
    assert_difference(
        r#"
const origin = new Temporal.ZonedDateTime(0n, 'UTC');
for (const [ns, expected] of [[2500000000n, 0], [7500000000n, 10], [-2500000000n, 0], [-7500000000n, -10]]) {
  const result = origin.until(new Temporal.ZonedDateTime(ns, 'UTC'), {largestUnit: 'second', smallestUnit: 'second', roundingIncrement: 5, roundingMode: 'halfEven'});
  if (result.seconds !== expected) throw new Error('seconds midpoint');
}
for (const [ns, expected] of [[1100000000n, 1200], [1300000000n, 1200], [-1100000000n, -1200], [-1300000000n, -1200]]) {
  const result = origin.until(new Temporal.ZonedDateTime(ns, 'UTC'), {largestUnit: 'millisecond', smallestUnit: 'millisecond', roundingIncrement: 200, roundingMode: 'halfEven'});
  if (result.milliseconds !== expected) throw new Error('whole duration quotient parity');
}
true;
"#,
    );
}
