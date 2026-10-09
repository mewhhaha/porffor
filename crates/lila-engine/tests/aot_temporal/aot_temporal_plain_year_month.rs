use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_year_month_semantics(source: &str) {
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
        .expect("Temporal year-month semantics must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn partial_strings_require_iso_calendar_before_overflow_options() {
    assert_year_month_semantics(
        r#"
function expectRangeError(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof RangeError)) throw new Error('invalid year-month string');
}
const yearMonth = new Temporal.PlainYearMonth(1976, 11);
for (const invalid of [
  '2020-13', '1976-11[u-ca=gregory]', '197611[u-ca=gregory]',
  '1976-11[u-ca=hebrew]', '1976-11[U-CA=iso8601]',
  '1976-11[u-CA=iso8601]', '1976-11[FOO=bar]', '+999999-01', '-999999-01'
]) {
  expectRangeError(() => Temporal.PlainYearMonth.from(invalid));
  expectRangeError(() => Temporal.PlainYearMonth.compare(yearMonth, invalid));
  expectRangeError(() => Temporal.PlainYearMonth.compare(invalid, yearMonth));
  expectRangeError(() => yearMonth.equals(invalid));
}
for (const valid of ['1976-11', '197611', '1976-11[u-ca=ISO8601]', '1976-11-18[u-ca=gregory]']) {
  const result = Temporal.PlainYearMonth.from(valid);
  if (result.year !== 1976 || result.month !== 11) throw new Error('valid year-month string');
}
let reads = 0;
expectRangeError(() => Temporal.PlainYearMonth.from('1976-11[u-ca=gregory]', {
  get overflow() { reads++; throw {}; }
}));
if (reads !== 0) throw new Error('parse rejection must precede overflow');
true;
"#,
    );
}

#[test]
fn arithmetic_uses_first_day_for_both_signs_and_overflow_modes() {
    assert_year_month_semantics(
        r#"
for (const year of [2023, 2024]) {
  for (let month = 1; month <= 12; month++) {
    const value = new Temporal.PlainYearMonth(year, month);
    for (const duration of [{years: 1}, {years: -1}, {months: 1}, {months: -1}]) {
      const added = value.add(duration, {overflow: 'reject'});
      const constrained = value.add(duration, {overflow: 'constrain'});
      if (!added.equals(constrained)) throw new Error('add overflow depends on reference day');
      const subtracted = value.subtract(duration, {overflow: 'reject'});
      const constrainedSubtract = value.subtract(duration, {overflow: 'constrain'});
      if (!subtracted.equals(constrainedSubtract)) throw new Error('subtract overflow depends on reference day');
    }
  }
}
const last = new Temporal.PlainYearMonth(275760, 9);
for (const overflow of ['constrain', 'reject']) {
  if (last.add({months: -1}, {overflow}).toString() !== '+275760-08' ||
      last.subtract({months: 1}, {overflow}).toString() !== '+275760-08' ||
      last.add({years: -1}, {overflow}).toString() !== '+275759-09' ||
      last.subtract({years: 1}, {overflow}).toString() !== '+275759-09') {
    throw new Error('backwards arithmetic from final month');
  }
}
true;
"#,
    );
}

#[test]
fn differences_validate_first_days_after_the_exact_date_zero_shortcut() {
    assert_year_month_semantics(
        r#"
function expectRangeError(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof RangeError)) throw new Error('first-day range');
}
const first = new Temporal.PlainYearMonth(-271821, 4);
const firstWithDay = new Temporal.PlainYearMonth(-271821, 4, 'iso8601', 30);
const last = new Temporal.PlainYearMonth(275760, 9);
const epoch = new Temporal.PlainYearMonth(1970, 1);
if (first.until(first).sign !== 0 || first.since(first).sign !== 0 ||
    first.until(first, {roundingIncrement: 100000000}).sign !== 0) {
  throw new Error('equal ISO dates must return zero');
}
expectRangeError(() => first.until(firstWithDay));
expectRangeError(() => first.since(firstWithDay));
for (const other of [last, epoch]) {
  expectRangeError(() => first.until(other));
  expectRangeError(() => first.since(other));
  expectRangeError(() => other.until(first));
  expectRangeError(() => other.since(first));
}
for (const invalid of ['-271821-04', '-271821-04-30T23:59:59.999999999', '+275760-10']) {
  expectRangeError(() => epoch.until(invalid));
  expectRangeError(() => epoch.since(invalid));
}
for (const valid of ['-271821-05', '-271821-05-01T00:00', '+275760-09', '+275760-09-30T23:59:59.999999999']) {
  const until = epoch.until(valid);
  const since = epoch.since(valid);
  if (until.years !== -since.years || until.months !== -since.months) {
    throw new Error('valid boundary string difference');
  }
}
true;
"#,
    );
}

#[test]
fn rounding_validates_both_calendar_brackets_even_for_exact_or_truncated_results() {
    assert_year_month_semantics(
        r#"
function expectRangeError(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof RangeError)) throw new Error('rounding bracket range');
}
const earlier = new Temporal.PlainYearMonth(1970, 1);
const later = new Temporal.PlainYearMonth(1971, 1);
for (const roundingMode of ['trunc', 'expand', 'ceil', 'floor', 'halfExpand', 'halfEven']) {
  for (const smallestUnit of ['month', 'year']) {
    const options = {roundingIncrement: 100000000, roundingMode, smallestUnit};
    expectRangeError(() => earlier.until(later, options));
    expectRangeError(() => later.until(earlier, options));
    expectRangeError(() => later.since(earlier, options));
    expectRangeError(() => earlier.since(later, options));
  }
}
if (earlier.until(later).years !== 1 || earlier.until(earlier).sign !== 0) {
  throw new Error('ordinary exact differences');
}
true;
"#,
    );
}

#[test]
fn rounding_retains_years_and_balances_only_after_rounding_months() {
    assert_year_month_semantics(
        r#"
function check(duration, years, months) {
  if (duration.years !== years || duration.months !== months || duration.days !== 0) {
    throw new Error('rounded year-month duration');
  }
}
const earlier = new Temporal.PlainYearMonth(2019, 1);
const later = new Temporal.PlainYearMonth(2021, 9);
const mixed = {smallestUnit: 'month', roundingIncrement: 5};
check(earlier.until(later, mixed), 2, 5);
check(later.since(earlier, mixed), 2, 5);
check(later.until(earlier, mixed), -2, -5);
check(earlier.since(later, mixed), -2, -5);
check(earlier.until(later, {smallestUnit: 'year', roundingIncrement: 4, roundingMode: 'halfExpand'}), 4, 0);
check(earlier.until(later, {largestUnit: 'month', smallestUnit: 'month', roundingIncrement: 10}), 0, 30);
check(earlier.until(later, {smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'expand'}), 2, 10);
const almostThree = new Temporal.PlainYearMonth(2021, 12);
check(earlier.until(almostThree, {smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'expand'}), 3, 0);
check(almostThree.until(earlier, {smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'expand'}), -3, 0);
check(almostThree.since(earlier, {smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'expand'}), 3, 0);
check(later.since(earlier, {smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'ceil'}), 2, 10);
check(earlier.since(later, {smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'floor'}), -2, -10);
true;
"#,
    );
}

#[test]
fn half_even_uses_increment_quotients_and_calendar_day_distances() {
    assert_year_month_semantics(
        r#"
function check(start, end, options, years, months) {
  const duration = start.until(end, options);
  if (duration.years !== years || duration.months !== months) throw new Error('half-even quotient');
}
const months = {largestUnit: 'month', smallestUnit: 'month', roundingIncrement: 2, roundingMode: 'halfEven'};
// July and August both have 31 days: August 1 is exactly between July 1 and September 1.
check(new Temporal.PlainYearMonth(2021, 5), new Temporal.PlainYearMonth(2021, 8), months, 0, 4);
check(new Temporal.PlainYearMonth(2021, 3), new Temporal.PlainYearMonth(2021, 8), months, 0, 4);
check(new Temporal.PlainYearMonth(2021, 11), new Temporal.PlainYearMonth(2021, 8), months, 0, -4);
// Unequal month lengths must not be treated as a numerical half tie.
check(new Temporal.PlainYearMonth(2021, 1), new Temporal.PlainYearMonth(2021, 2), months, 0, 2);
check(new Temporal.PlainYearMonth(2020, 1), new Temporal.PlainYearMonth(2023, 1),
  {smallestUnit: 'year', roundingIncrement: 2, roundingMode: 'halfEven'}, 4, 0);
true;
"#,
    );
}

#[test]
fn option_order_and_abrupt_identity_precede_date_and_duration_validation() {
    assert_year_month_semantics(
        r#"
const minimum = new Temporal.PlainYearMonth(-271821, 4);
const epoch = new Temporal.PlainYearMonth(1970, 1);
const trace = [];
const options = {
  get largestUnit() { trace.push('largestUnit'); return 'year'; },
  get roundingIncrement() { trace.push('roundingIncrement'); return 1; },
  get roundingMode() { trace.push('roundingMode'); return 'trunc'; },
  get smallestUnit() { trace.push('smallestUnit'); return 'month'; }
};
if (minimum.until(minimum, options).sign !== 0 ||
    trace.join(',') !== 'largestUnit,roundingIncrement,roundingMode,smallestUnit') {
  throw new Error('options before equal-date shortcut');
}
trace.length = 0;
let received;
try { minimum.until(epoch, options); } catch (error) { received = error; }
if (!(received instanceof RangeError) ||
    trace.join(',') !== 'largestUnit,roundingIncrement,roundingMode,smallestUnit') {
  throw new Error('options before first-day validation');
}
const marker = {};
for (const value of [minimum, epoch]) {
  for (const duration of [{months: 1}, {weeks: 1}]) {
    let thrown;
    try { value.add(duration, {get overflow() { throw marker; }}); }
    catch (error) { thrown = error; }
    if (thrown !== marker) throw new Error('overflow abrupt identity');
  }
}
let thrown;
try { minimum.since(epoch, {get roundingIncrement() { throw marker; }}); }
catch (error) { thrown = error; }
if (thrown !== marker) throw new Error('difference option abrupt identity');
true;
"#,
    );
}

#[test]
fn expanded_month_rounding_validates_and_selects_the_next_year_boundary() {
    assert_year_month_semantics(
        r#"
function expectRangeError(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof RangeError)) throw new Error('bubble year range');
}
// Both month brackets are valid. Only the next whole-year candidate is outside
// the date range, and BubbleRelativeDuration must validate it before comparison.
for (const [start, end, sign] of [
  [new Temporal.PlainYearMonth(275760, 1), new Temporal.PlainYearMonth(275760, 2), 1],
  [new Temporal.PlainYearMonth(-271821, 8), new Temporal.PlainYearMonth(-271821, 7), -1]
]) {
  const expand = {smallestUnit: 'month', roundingIncrement: 2, roundingMode: 'expand'};
  expectRangeError(() => start.until(end, expand));
  expectRangeError(() => start.since(end, expand));
  const months = start.until(end, {
    largestUnit: 'month', smallestUnit: 'month', roundingIncrement: 2, roundingMode: 'expand'
  });
  if (months.years !== 0 || months.months !== 2 * sign) throw new Error('no larger unit');
  const truncated = start.until(end, {smallestUnit: 'month', roundingIncrement: 2});
  if (truncated.sign !== 0) throw new Error('no expansion');
}
const start = new Temporal.PlainYearMonth(2019, 1);
const exactBoundary = new Temporal.PlainYearMonth(2021, 12);
const rounded = start.until(exactBoundary, {
  smallestUnit: 'month', roundingIncrement: 3, roundingMode: 'expand'
});
if (rounded.years !== 3 || rounded.months !== 0) throw new Error('reaches next year');
const beforeBoundary = start.until(new Temporal.PlainYearMonth(2021, 9), {
  smallestUnit: 'month', roundingIncrement: 5, roundingMode: 'expand'
});
if (beforeBoundary.years !== 2 || beforeBoundary.months !== 10) throw new Error('before next year');
true;
"#,
    );
}
