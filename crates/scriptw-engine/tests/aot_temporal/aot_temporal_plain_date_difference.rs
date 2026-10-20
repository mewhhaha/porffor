use scriptw_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_plain_date_difference(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("PlainDate differences must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn half_modes_round_exact_calendar_midpoints_with_both_bucket_parities_and_signs() {
    assert_plain_date_difference(
        r#"
function check(duration, unit, expected) {
  const years = unit === 'year' ? expected : 0;
  const months = unit === 'month' ? expected : 0;
  if (duration.years !== years || duration.months !== months || duration.weeks !== 0 ||
      duration.days !== 0 || duration.hours !== 0 || duration.minutes !== 0 ||
      duration.seconds !== 0 || duration.milliseconds !== 0 ||
      duration.microseconds !== 0 || duration.nanoseconds !== 0) {
    throw new Error('calendar midpoint duration');
  }
}
// Each negative-direction window is itself exactly halfway. Reversing an
// arbitrary date pair need not preserve its calendar-unit fraction.
for (const row of [
  ['2019-01-01', '2020-07-02', 'year', 1, 1],
  ['2018-01-01', '2020-07-02', 'year', 2, 1],
  ['2022-01-01', '2020-07-02', 'year', 1, -1],
  ['2023-01-01', '2020-07-02', 'year', 2, -1],
  ['2019-01-01', '2019-02-15', 'month', 1, 1],
  ['2018-12-01', '2019-02-15', 'month', 2, 1],
  ['2019-04-01', '2019-02-15', 'month', 1, -1],
  ['2019-05-01', '2019-02-15', 'month', 2, -1]
]) {
  const start = Temporal.PlainDate.from(row[0]);
  const end = Temporal.PlainDate.from(row[1]);
  const unit = row[2];
  const lower = row[3];
  const sign = row[4];
  for (const mode of ['trunc', 'ceil', 'floor', 'expand',
                      'halfCeil', 'halfFloor', 'halfExpand', 'halfTrunc', 'halfEven']) {
    let up = mode === 'expand' || mode === 'halfExpand' ||
      ((mode === 'ceil' || mode === 'halfCeil') && sign > 0) ||
      ((mode === 'floor' || mode === 'halfFloor') && sign < 0) ||
      (mode === 'halfEven' && lower % 2 === 1);
    check(start.until(end, {smallestUnit: unit, roundingMode: mode}),
      unit, sign * (lower + (up ? 1 : 0)));
    up = mode === 'expand' || mode === 'halfExpand' ||
      ((mode === 'ceil' || mode === 'halfCeil') && sign < 0) ||
      ((mode === 'floor' || mode === 'halfFloor') && sign > 0) ||
      (mode === 'halfEven' && lower % 2 === 1);
    check(start.since(end, {smallestUnit: unit, roundingMode: mode}),
      unit, -sign * (lower + (up ? 1 : 0)));
  }
}
true;
"#,
    );
}

#[test]
fn half_even_uses_increment_quotients_and_excludes_retained_larger_fields() {
    assert_plain_date_difference(
        r#"
function check(duration, years, months, weeks, days) {
  if (duration.years !== years || duration.months !== months || duration.weeks !== weeks ||
      duration.days !== days || duration.hours !== 0 || duration.minutes !== 0 ||
      duration.seconds !== 0 || duration.milliseconds !== 0 ||
      duration.microseconds !== 0 || duration.nanoseconds !== 0) {
    throw new Error('calendar increment quotient');
  }
}
// An even unit coefficient can still have an odd increment quotient. All
// rows are exact calendar-day midpoints; week/day dispatch stays distinct.
for (const row of [
  ['2015-01-01', '2018-01-01', 'year', 2, 4, 0, 0, 0],
  ['2013-01-01', '2018-01-01', 'year', 2, 4, 0, 0, 0],
  ['2021-01-01', '2018-01-01', 'year', 2, -4, 0, 0, 0],
  ['2023-01-01', '2018-01-01', 'year', 2, -4, 0, 0, 0],
  ['2019-05-01', '2019-08-01', 'month', 2, 0, 4, 0, 0],
  ['2019-03-01', '2019-08-01', 'month', 2, 0, 4, 0, 0],
  ['2019-11-01', '2019-08-01', 'month', 2, 0, -4, 0, 0],
  ['2020-01-01', '2019-08-01', 'month', 2, 0, -4, 0, 0],
  ['2019-11-01', '2020-03-17', 'month', 3, 0, 6, 0, 0],
  ['2019-08-01', '2020-03-17', 'month', 3, 0, 6, 0, 0],
  ['2020-08-01', '2020-03-17', 'month', 3, 0, -6, 0, 0],
  ['2020-11-01', '2020-03-17', 'month', 3, 0, -6, 0, 0],
  ['2020-01-01', '2020-01-22', 'week', 2, 0, 0, 4, 0],
  ['2020-01-01', '2020-02-05', 'week', 2, 0, 0, 4, 0],
  ['2020-02-12', '2020-01-22', 'week', 2, 0, 0, -4, 0],
  ['2020-02-26', '2020-01-22', 'week', 2, 0, 0, -4, 0],
  ['2020-01-01', '2020-01-04', 'day', 2, 0, 0, 0, 4],
  ['2020-01-01', '2020-01-06', 'day', 2, 0, 0, 0, 4],
  ['2020-01-04', '2020-01-01', 'day', 2, 0, 0, 0, -4]
]) {
  const start = Temporal.PlainDate.from(row[0]);
  const end = Temporal.PlainDate.from(row[1]);
  const options = {smallestUnit: row[2], roundingIncrement: row[3], roundingMode: 'halfEven'};
  check(start.until(end, options), row[4], row[5], row[6], row[7]);
  check(start.since(end, options), -row[4], -row[5], -row[6], -row[7]);
}
// For a four-month increment the retained year contributes an odd number of
// increments. Its parity must not leak into the selected month coefficient.
const options = {largestUnit: 'year', smallestUnit: 'month',
  roundingIncrement: 4, roundingMode: 'halfEven'};
for (const row of [
  ['2019-10-01', '2021-04-02', 1, 8],
  ['2019-06-01', '2021-04-02', 1, 8],
  ['2022-10-01', '2021-04-02', -1, -8],
  ['2023-02-01', '2021-04-02', -1, -8]
]) {
  const start = Temporal.PlainDate.from(row[0]);
  const end = Temporal.PlainDate.from(row[1]);
  check(start.until(end, options), row[2], row[3], 0, 0);
  check(start.since(end, options), -row[2], -row[3], 0, 0);
}
true;
"#,
    );
}

#[test]
fn calendar_rounding_preserves_non_ties_exact_dates_and_unselected_endpoint_checks() {
    assert_plain_date_difference(
        r#"
for (const row of [
  ['2019-01-01', '2020-07-01', 'year', 1],
  ['2019-01-01', '2020-07-03', 'year', 2],
  ['2018-01-01', '2020-07-01', 'year', 2],
  ['2018-01-01', '2020-07-03', 'year', 3],
  ['2022-01-01', '2020-07-01', 'year', -2],
  ['2022-01-01', '2020-07-03', 'year', -1],
  ['2019-01-01', '2019-02-14', 'month', 1],
  ['2019-01-01', '2019-02-16', 'month', 2],
  ['2018-12-01', '2019-02-14', 'month', 2],
  ['2018-12-01', '2019-02-16', 'month', 3],
  ['2019-04-01', '2019-02-14', 'month', -2],
  ['2019-04-01', '2019-02-16', 'month', -1]
]) {
  const start = Temporal.PlainDate.from(row[0]);
  const end = Temporal.PlainDate.from(row[1]);
  const unit = row[2];
  const options = {smallestUnit: unit, roundingMode: 'halfEven'};
  const until = start.until(end, options);
  const since = start.since(end, options);
  if ((unit === 'year' ? until.years : until.months) !== row[3] ||
      (unit === 'year' ? since.years : since.months) !== -row[3]) {
    throw new Error('calendar non-tie distance');
  }
}
const start = new Temporal.PlainDate(2020, 1, 1);
const exact = new Temporal.PlainDate(2022, 1, 1);
for (const roundingMode of ['trunc', 'ceil', 'floor', 'expand',
                            'halfCeil', 'halfFloor', 'halfExpand', 'halfTrunc', 'halfEven']) {
  const options = {smallestUnit: 'year', roundingIncrement: 2, roundingMode};
  if (start.until(exact, options).years !== 2 || start.since(exact, options).years !== -2 ||
      exact.until(start, options).years !== -2 || exact.since(start, options).years !== 2 ||
      start.until(start, options).sign !== 0) throw new Error('exact calendar endpoints');
  let caught;
  try {
    start.until(new Temporal.PlainDate(275760, 1, 1), {
      smallestUnit: 'year', roundingIncrement: 1000000, roundingMode
    });
  } catch (error) { caught = error; }
  if (!(caught instanceof RangeError)) throw new Error('unselected calendar endpoint range');
}
true;
"#,
    );
}
