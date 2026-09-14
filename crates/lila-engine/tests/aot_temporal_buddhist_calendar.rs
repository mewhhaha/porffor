use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_calendar_semantics(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("calendar semantics failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn non_iso_week_fields_are_undefined_on_all_date_carriers() {
    assert_calendar_semantics(
        r#"
var iso = new Temporal.PlainDate(2016, 1, 1);
if (iso.weekOfYear !== 53 || iso.yearOfWeek !== 2015) throw 'ISO week boundary';
var isoTime = new Temporal.PlainDateTime(2016, 1, 1);
if (isoTime.weekOfYear !== 53 || isoTime.yearOfWeek !== 2015) throw 'ISO datetime week';
for (var calendar of ['gregory', 'buddhist']) {
  var date = new Temporal.PlainDate(2016, 1, 1, calendar);
  var time = isoTime.withCalendar(calendar);
  var zoned = new Temporal.ZonedDateTime(1451606400000000000n, 'UTC', calendar);
  if (date.weekOfYear !== undefined || date.yearOfWeek !== undefined ||
      time.weekOfYear !== undefined || time.yearOfWeek !== undefined ||
      zoned.weekOfYear !== undefined || zoned.yearOfWeek !== undefined) throw 'non-ISO week';
}
true;
"#,
    );
}

#[test]
fn buddhist_constructors_and_strings_keep_iso_coordinates_while_bags_use_calendar_years() {
    assert_calendar_semantics(
        r#"
var date = new Temporal.PlainDate(2020, 2, 29, 'BUDDHIST');
var bag = Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, month: 2, day: 29});
var annotated = Temporal.PlainDate.from('2020-02-29[u-ca=buddhist]');
if (date.year !== 2563 || date.era !== 'be' || date.eraYear !== 2563 ||
    date.calendarId !== 'buddhist' || !date.equals(bag) || !date.equals(annotated) ||
    date.toString() !== '2020-02-29[u-ca=buddhist]') throw 'date coordinates';
var time = new Temporal.PlainDateTime(2020, 2, 29, 12, 0, 0, 0, 0, 0, 'buddhist');
var timeBag = Temporal.PlainDateTime.from({calendar: 'buddhist', year: 2563, monthCode: 'M02', day: 29, hour: 12});
if (!time.equals(timeBag) || time.year !== 2563 || time.eraYear !== 2563 ||
    !time.toPlainDate().equals(date)) throw 'datetime coordinates';
var yearMonth = new Temporal.PlainYearMonth(2020, 2, 'buddhist');
var yearMonthBag = Temporal.PlainYearMonth.from({calendar: 'buddhist', year: 2563, monthCode: 'M02'});
if (!yearMonth.equals(yearMonthBag) || yearMonth.year !== 2563 || yearMonth.eraYear !== 2563 ||
    !yearMonth.toPlainDate({day: 29}).equals(date)) throw 'year-month coordinates';
var monthDay = new Temporal.PlainMonthDay(2, 29, 'buddhist');
if (!monthDay.toPlainDate({year: 2563}).equals(date) ||
    !monthDay.toPlainDate({era: 'be', eraYear: 2563}).equals(date)) throw 'month-day coordinates';
var zoned = new Temporal.ZonedDateTime(0n, 'UTC', 'buddhist');
var zonedBag = Temporal.ZonedDateTime.from({calendar: 'buddhist', year: 2513, month: 1, day: 1, timeZone: 'UTC'});
if (zoned.year !== 2513 || zoned.era !== 'be' || zoned.eraYear !== 2513 ||
    !zoned.equals(zonedBag) || zoned.toPlainDate().year !== 2513 ||
    zoned.toPlainDateTime().year !== 2513) throw 'zoned coordinates';
date.withCalendar('iso8601').year === 2020 &&
  new Temporal.PlainDate(2020, 2, 29).withCalendar('2020-02-29[u-ca=buddhist]').equals(date);
"#,
    );
}

#[test]
fn buddhist_fields_use_proleptic_iso_leap_rules_and_preserve_non_positive_era_years() {
    assert_calendar_semantics(
        r#"
for (var year = 2513; year < 2593; year++) {
  var date = Temporal.PlainDate.from({calendar: 'buddhist', year, month: 2, day: 1});
  var leap = (year - 543) % 4 === 0;
  if (date.year !== year || date.era !== 'be' || date.eraYear !== year ||
      date.inLeapYear !== leap || date.daysInYear !== (leap ? 366 : 365) ||
      date.daysInMonth !== (leap ? 29 : 28) || date.monthsInYear !== 12 ||
      date.month !== 2 || date.monthCode !== 'M02' || date.dayOfYear !== 32) throw 'calendar fields';
}
for (var eraYear of [-1, 0, 1]) {
  var ancient = Temporal.PlainDate.from({calendar: 'buddhist', era: 'be', eraYear, monthCode: 'M01', day: 1});
  if (ancient.year !== eraYear || ancient.eraYear !== eraYear || ancient.era !== 'be' ||
      ancient.withCalendar('iso8601').year !== eraYear - 543) throw 'single era';
}
var century = Temporal.PlainDate.from({calendar: 'buddhist', year: 2443, month: 2, day: 1});
var quadricentury = Temporal.PlainDate.from({calendar: 'buddhist', year: 2543, month: 2, day: 1});
var minimum = Temporal.PlainDate.from({calendar: 'buddhist', year: -271278, month: 4, day: 19});
var maximum = Temporal.PlainDate.from({calendar: 'buddhist', year: 276303, month: 9, day: 13});
!century.inLeapYear && quadricentury.inLeapYear && minimum.year === -271278 && maximum.year === 276303;
"#,
    );
}

#[test]
fn buddhist_with_merges_iso_receivers_without_converting_a_missing_year_twice() {
    assert_calendar_semantics(
        r#"
var date = Temporal.PlainDate.from({calendar: 'buddhist', year: 2566, monthCode: 'M12', day: 15});
var time = date.toPlainDateTime({hour: 12});
var ym = date.toPlainYearMonth();
for (var receiver of [date, time, ym]) {
  if (receiver.with({month: 5}).year !== 2566 || receiver.with({monthCode: 'M05'}).year !== 2566 ||
      receiver.with({year: 2559}).year !== 2559 ||
      receiver.with({era: 'be', eraYear: 2560}).year !== 2560) throw 'with year merge';
}
if (time.with({hour: 2}).year !== 2566 || date.with({day: 1}).year !== 2566) throw 'absent year';
var leap = Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, month: 2, day: 29});
leap.with({year: 2564}).day === 28 && leap.with({year: 2564}).year === 2564;
"#,
    );
}

#[test]
fn buddhist_add_until_and_round_use_the_stored_iso_date_across_calendar_year_boundaries() {
    assert_calendar_semantics(
        r#"
var leap = Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, month: 2, day: 29});
var next = leap.add({years: 1});
if (next.year !== 2564 || next.month !== 2 || next.day !== 28 ||
    next.subtract({years: 1}).year !== 2563) throw 'calendar addition';
var january = Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, month: 1, day: 31});
if (!january.add({months: 1}).equals(leap)) throw 'month constrain';
var start = Temporal.PlainDate.from({calendar: 'buddhist', year: 2125, month: 10, day: 4});
if (start.add({days: 3}).day !== 7) throw 'proleptic calendar';
var end = leap.add({years: 2, months: 7, days: 13});
for (var options of [
  {largestUnit: 'year'}, {largestUnit: 'month'}, {largestUnit: 'week'},
  {largestUnit: 'year', smallestUnit: 'month', roundingIncrement: 3, roundingMode: 'halfExpand'}
]) {
  var actual = leap.until(end, options);
  var expected = leap.withCalendar('iso8601').until(end.withCalendar('iso8601'), options);
  if (actual.toString() !== expected.toString() ||
      end.since(leap, options).toString() !== expected.toString()) throw 'calendar difference';
}
var time = leap.toPlainDateTime({hour: 18});
if (time.round('day').year !== 2563 || time.round('day').month !== 3 ||
    time.round('day').day !== 1) throw 'datetime rounding';
var ym = leap.toPlainYearMonth(), ymEnd = ym.add({years: 2, months: 8});
if (ymEnd.year !== 2565 || ymEnd.month !== 10 || ym.until(ymEnd, {largestUnit: 'month'}).months !== 32) throw 'year-month arithmetic';
var zoned = Temporal.ZonedDateTime.from({calendar: 'buddhist', year: 2563, month: 2, day: 29, hour: 18, timeZone: 'UTC'});
var rounded = zoned.round('day');
rounded.year === 2563 && rounded.month === 3 && rounded.day === 1 &&
  zoned.add({years: 1}).year === 2564 && zoned.until(zoned.add({months: 1}), {largestUnit: 'month'}).months === 1;
"#,
    );
}

#[test]
fn buddhist_month_day_uses_an_iso_reference_year_after_calendar_year_overflow() {
    assert_calendar_semantics(
        r#"
var leap = Temporal.PlainMonthDay.from({calendar: 'buddhist', monthCode: 'M02', day: 29});
var constrained = Temporal.PlainMonthDay.from({calendar: 'buddhist', year: 2564, month: 2, day: 29});
if (leap.toString() !== '1972-02-29[u-ca=buddhist]' ||
    constrained.toString() !== '1972-02-28[u-ca=buddhist]' ||
    leap.with({year: 2564}).day !== 28 || leap.with({day: 29}).day !== 29 ||
    leap.toPlainDate({year: 2563}).day !== 29 || leap.toPlainDate({year: 2564}).day !== 28) throw 'reference year';
var parsed = Temporal.PlainMonthDay.from('2020-02-29[u-ca=buddhist]');
var fromDate = Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, month: 2, day: 29}).toPlainMonthDay();
parsed.equals(leap) && fromDate.equals(leap);
"#,
    );
}

#[test]
fn buddhist_field_errors_preserve_option_read_order_and_calendar_specific_eras() {
    assert_calendar_semantics(
        r#"
function expectError(kind, action) {
  var received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof kind)) throw 'wrong calendar error';
}
var log = [];
var bag = {
  get calendar() { log.push('calendar'); return 'buddhist'; },
  get day() { log.push('day'); return 1; },
  get era() { log.push('era'); return 'be'; },
  get eraYear() { log.push('eraYear'); return 2563; },
  get month() { log.push('month'); return 1; },
  get monthCode() { log.push('monthCode'); return 'M01'; },
  get year() { log.push('year'); return 2020; }
};
expectError(RangeError, () => Temporal.PlainDate.from(bag, {
  get overflow() { log.push('overflow'); return 'reject'; }
}));
if (log.join('|') !== 'calendar|day|era|eraYear|month|monthCode|year|overflow') throw 'read order';
expectError(RangeError, () => Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, era: 'ce', eraYear: 2563, month: 1, day: 1}));
expectError(TypeError, () => Temporal.PlainDate.from({calendar: 'buddhist', era: 'be', month: 1, day: 1}));
expectError(RangeError, () => Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, month: 3, monthCode: 'M02', day: 1}));
expectError(RangeError, () => Temporal.PlainDate.from({calendar: 'buddhist', year: 2563, monthCode: 'M13', day: 1}));
expectError(RangeError, () => Temporal.PlainMonthDay.from({calendar: 'buddhist', year: 2564, monthCode: 'M02', day: 29}, {overflow: 'reject'}));
expectError(RangeError, () => Temporal.PlainMonthDay.from({calendar: 'buddhist', year: 1e300, monthCode: 'M02', day: 1}));
expectError(RangeError, () => Temporal.PlainMonthDay.from({calendar: 'buddhist', year: -1e300, monthCode: 'M02', day: 1}));
new Intl.DateTimeFormat('en', {calendar: 'buddhist'}).resolvedOptions().calendar === 'gregory';
"#,
    );
}
