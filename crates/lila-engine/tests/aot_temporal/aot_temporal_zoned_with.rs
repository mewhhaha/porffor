use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_zoned_with(source: &str) {
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
        .unwrap_or_else(|error| panic!("zoned field replacement failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn zoned_with_has_intrinsic_metadata_branding_and_result_prototype() {
    assert_zoned_with(
        r#"
var method = Temporal.ZonedDateTime.prototype.with;
var descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, 'with');
if (method.name !== 'with' || method.length !== 1 || !descriptor.writable ||
    descriptor.enumerable || !descriptor.configurable || Object.hasOwn(method, 'prototype')) throw 'metadata';
var reads = 0, fields = {get day() { reads++; return 2; }};
for (var receiver of [undefined, null, 1, {}, new Temporal.PlainDate(2000, 1, 1)]) {
  var caught = undefined;
  try { method.call(receiver, fields); } catch (e) { caught = e; }
  if (!(caught instanceof TypeError) || reads !== 0) throw 'receiver branding';
}
var constructorError;
try { new method({day: 2}); } catch (e) { constructorError = e; }
if (!(constructorError instanceof TypeError)) throw 'constructor';
class Derived extends Temporal.ZonedDateTime {}
var original = new Derived(0n, '+01:00', 'buddhist');
Object.defineProperty(original, 'constructor', {get() { throw 'constructor read'; }});
var result = original.with({day: 2});
if (Object.getPrototypeOf(result) !== Temporal.ZonedDateTime.prototype || result === original ||
    original.epochNanoseconds !== 0n || result.epochNanoseconds !== 86400000000000n ||
    result.calendarId !== 'buddhist' || result.timeZoneId !== '+01:00') throw 'result intrinsic';
true;
"#,
    );
}

#[test]
fn zoned_with_replaces_all_fields_and_preserves_unsupplied_subseconds() {
    assert_zoned_with(
        r#"
var original = new Temporal.ZonedDateTime(-1n, 'UTC');
var result = original.with({year: 2000, monthCode: 'M02', day: 29, hour: 12,
  minute: 34, second: 56, millisecond: 123, microsecond: 456, nanosecond: 789});
if (result.toPlainDateTime().toString() !== '2000-02-29T12:34:56.123456789') throw 'all fields';
if (original.with({second: 0}).epochNanoseconds !== -59000000001n ||
    original.with({nanosecond: 1}).epochNanoseconds !== -999n) throw 'negative subseconds';
var inherited = Object.create({day: 2});
inherited.year = undefined;
inherited.calendar = undefined;
inherited.timeZone = undefined;
var epoch = new Temporal.ZonedDateTime(123456789n, 'UTC');
var changed = epoch.with(inherited);
if (changed.epochNanoseconds !== 86400123456789n || changed.year !== 1970) throw 'inherited fields';
if (epoch.with({offset: '+00'}).epochNanoseconds !== epoch.epochNanoseconds) throw 'offset-only';
for (var empty of [{}, {year: undefined}, {Day: 2}, {era: 'ce'}, {calendar: undefined}]) {
  var caught = undefined;
  try { epoch.with(empty); } catch (e) { caught = e; }
  if (!(caught instanceof TypeError)) throw 'empty fields';
}
true;
"#,
    );
}

#[test]
fn zoned_with_reads_and_coerces_fields_before_options_in_calendar_order() {
    assert_zoned_with(
        r#"
var log = [];
function number(name, value) { return {valueOf() { log.push('number ' + name); return value; }}; }
function string(name, value) { return {toString() { log.push('string ' + name); return value; }}; }
var fields = new Proxy({day: number('day', 2), monthCode: string('monthCode', 'M01'),
  offset: string('offset', '+00'), year: number('year', 1970)}, {
  get(target, key) { log.push('field ' + key); return target[key]; }
});
var options = new Proxy({disambiguation: string('disambiguation', 'compatible'),
  offset: string('option offset', 'prefer'), overflow: string('overflow', 'constrain')}, {
  get(target, key) { log.push('option ' + key); return target[key]; }
});
var epoch = new Temporal.ZonedDateTime(0n, 'UTC');
epoch.with(fields, options);
var expected = 'field calendar|field timeZone|field day|number day|field hour|field microsecond|field millisecond|field minute|field month|field monthCode|string monthCode|field nanosecond|field offset|string offset|field second|field year|number year|option disambiguation|string disambiguation|option offset|string option offset|option overflow|string overflow';
if (log.join('|') !== expected) throw log.join('|');
log = [];
var caught = undefined;
try { epoch.with(fields, null); } catch (e) { caught = e; }
if (!(caught instanceof TypeError) || log.join('|') !== expected.split('|option disambiguation')[0]) throw 'primitive options order';
log = [];
var buddhistFields = new Proxy({era: 'be', eraYear: 2514}, {
  get(target, key) { log.push(key); return target[key]; }
});
var buddhist = new Temporal.ZonedDateTime(0n, 'UTC', 'buddhist').with(buddhistFields);
if (buddhist.year !== 2514 || log.join('|') !== 'calendar|timeZone|day|era|eraYear|hour|microsecond|millisecond|minute|month|monthCode|nanosecond|offset|second|year') throw 'era order';
true;
"#,
    );
}

#[test]
fn offset_conversion_requires_a_string_primitive_and_preserves_abrupt_values() {
    assert_zoned_with(
        r#"
var epoch = new Temporal.ZonedDateTime(0n, 'UTC');
function from(offset) { return Temporal.ZonedDateTime.from({year: 1970, month: 1, day: 1, timeZone: 'UTC', offset}); }
for (var value of [0, null, true, 1n, Symbol('offset')]) {
  for (var operation of [offset => epoch.with({offset}), from]) {
    var primitiveError = undefined, boxedError = undefined;
    try { operation(value); } catch (e) { primitiveError = e; }
    try { operation({toString() { return value; }}); } catch (e) { boxedError = e; }
    if (!(primitiveError instanceof TypeError) || !(boxedError instanceof TypeError)) throw 'offset primitive type';
  }
}
var hints = [], reads = 0;
var good = {[Symbol.toPrimitive](hint) { hints.push(hint); return '+00:00'; }};
if (epoch.with({offset: good}).epochNanoseconds !== 0n || from(good).epochNanoseconds !== 0n ||
    hints.join('|') !== 'string|string') throw 'offset hint';
var sentinel = {};
for (var operation of [offset => epoch.with({offset}), from]) {
  var caught = undefined;
  try { operation({toString() { throw sentinel; }}); } catch (e) { caught = e; }
  if (caught !== sentinel) throw 'offset abrupt value';
}
var invalid;
try { epoch.with({offset: 'invalid', get second() { reads++; return 1; }},
  {get overflow() { reads++; return 'constrain'; }}); } catch (e) { invalid = e; }
if (!(invalid instanceof RangeError) || reads !== 0) throw 'offset grammar read order';
true;
"#,
    );
}

#[test]
fn zoned_with_offset_modes_compare_exact_nanoseconds_and_keep_the_time_zone() {
    assert_zoned_with(
        r#"
var utc = new Temporal.ZonedDateTime(0n, 'UTC');
for (var preference of [undefined, 'prefer', 'ignore']) {
  var options = preference === undefined ? undefined : {offset: preference};
  if (utc.with({offset: '+00:00:00.000000001'}, options).epochNanoseconds !== 0n) throw 'prefer fallback';
}
if (utc.with({offset: '+00:00:00.000000001'}, {offset: 'use'}).epochNanoseconds !== -1n ||
    utc.with({offset: '-00:00:00.000000001'}, {offset: 'use'}).epochNanoseconds !== 1n ||
    utc.with({offset: '+01:02:03.456789123'}, {offset: 'use'}).epochNanoseconds !== -3723456789123n) throw 'exact offset arithmetic';
var rejected;
try { utc.with({offset: '+00:00:00.000000001'}, {offset: 'reject'}); } catch (e) { rejected = e; }
if (!(rejected instanceof RangeError)) throw 'exact offset match';
var fixed = new Temporal.ZonedDateTime(0n, '+01:30');
if (fixed.with({offset: '+01:30'}, {offset: 'reject'}).epochNanoseconds !== 0n ||
    fixed.with({offset: '+00'}, {offset: 'use'}).epochNanoseconds !== 5400000000000n ||
    fixed.with({day: 2}).timeZoneId !== '+01:30') throw 'fixed time zone';
for (var disambiguation of ['compatible', 'earlier', 'later', 'reject']) {
  if (fixed.with({day: 2}, {disambiguation}).epochNanoseconds !== 86400000000000n) throw 'unambiguous fixed zone';
}
true;
"#,
    );
}

#[test]
fn zoned_with_calendar_merge_uses_iso_storage_and_calendar_years() {
    assert_zoned_with(
        r#"
for (var calendar of ['iso8601', 'gregory', 'buddhist']) {
  var year = calendar === 'buddhist' ? 2563 : 2020;
  var leap = Temporal.ZonedDateTime.from({calendar, year, month: 2, day: 29, timeZone: 'UTC'});
  var next = leap.with({year: year + 1});
  if (next.year !== year + 1 || next.day !== 28 || next.calendarId !== calendar ||
      leap.with({month: 3}).day !== 29 || leap.with({monthCode: 'M01'}).month !== 1 ||
      leap.with({hour: 1}).year !== year) throw 'calendar merge';
  var conflict = undefined, overflow = undefined;
  try { leap.with({month: 1, monthCode: 'M02'}); } catch (e) { conflict = e; }
  try { leap.with({year: year + 1}, {overflow: 'reject'}); } catch (e) { overflow = e; }
  if (!(conflict instanceof RangeError) || !(overflow instanceof RangeError)) throw 'calendar validation';
}
var gregory = new Temporal.ZonedDateTime(0n, 'UTC', 'gregory');
if (gregory.with({era: 'bce', eraYear: 1}).year !== 0) throw 'Gregorian era merge';
var buddhist = new Temporal.ZonedDateTime(0n, 'UTC', 'buddhist');
if (buddhist.with({era: 'be', eraYear: 0}).year !== 0 ||
    buddhist.with({era: 'be', eraYear: -1}).withCalendar('iso8601').year !== -544) throw 'Buddhist era merge';
for (var partial of [{era: 'be'}, {eraYear: 2513}]) {
  var caught = undefined;
  try { buddhist.with(partial); } catch (e) { caught = e; }
  if (!(caught instanceof TypeError)) throw 'incomplete era';
}
var log = [], caught;
try { gregory.with({era: 'ce', month: 2, monthCode: 'M01'}, {
  get disambiguation() { log.push('disambiguation'); },
  get offset() { log.push('offset'); }, get overflow() { log.push('overflow'); }
}); } catch (e) { caught = e; }
if (!(caught instanceof TypeError) || log.join('|') !== 'disambiguation|offset|overflow') throw 'era resolution order';
true;
"#,
    );
}

#[test]
fn zoned_with_overflow_and_abrupt_partial_objects_observe_required_boundaries() {
    assert_zoned_with(
        r#"
var epoch = new Temporal.ZonedDateTime(0n, 'UTC'), sentinel = {}, reads = 0;
for (var partial of [null, undefined, 1, '1970-01-01', new Temporal.PlainTime(),
    new Temporal.PlainDate(1970, 1, 1), epoch]) {
  var caught = undefined;
  try { epoch.with(partial, {get overflow() { reads++; }}); } catch (e) { caught = e; }
  if (!(caught instanceof TypeError) || reads !== 0) throw 'partial object';
}
var calendarError;
try { epoch.with({get calendar() { throw sentinel; }, get timeZone() { reads++; }}); }
catch (e) { calendarError = e; }
if (calendarError !== sentinel || reads !== 0) throw 'calendar abrupt';
var constrained = epoch.with({month: 15, day: 40, hour: 30, minute: 80, second: 70,
  millisecond: 1000, microsecond: 1000, nanosecond: 1000});
if (constrained.toPlainDateTime().toString() !== '1970-12-31T23:59:59.999999999') throw 'constrain';
for (var value of [Infinity, -Infinity, NaN]) {
  var caught = undefined;
  try { epoch.with({hour: value}); } catch (e) { caught = e; }
  if (!(caught instanceof RangeError)) throw 'finite fields';
}
for (var options of [{overflow: 'invalid'}, {offset: 'invalid'}, {disambiguation: 'invalid'}, {overflow: 'reject'}]) {
  var caught = undefined;
  try { epoch.with({hour: 30}, options); } catch (e) { caught = e; }
  if (!(caught instanceof RangeError)) throw 'option validation';
}
true;
"#,
    );
}

#[test]
fn zoned_with_enforces_epoch_limits_after_the_selected_offset() {
    assert_zoned_with(
        r#"
var minimum = new Temporal.ZonedDateTime(-8640000000000000000000n, 'UTC');
var maximum = new Temporal.ZonedDateTime(8640000000000000000000n, 'UTC');
if (minimum.with({offset: '+00'}).epochNanoseconds !== minimum.epochNanoseconds ||
    maximum.with({offset: '+00'}).epochNanoseconds !== maximum.epochNanoseconds) throw 'endpoint identity';
for (var pair of [[minimum, '+00:00:00.000000001'], [maximum, '-00:00:00.000000001']]) {
  var caught = undefined;
  try { pair[0].with({offset: pair[1]}, {offset: 'use'}); } catch (e) { caught = e; }
  if (!(caught instanceof RangeError)) throw 'endpoint overflow';
}
var previousLocalDay = new Temporal.ZonedDateTime(-8640000000000000000000n, '-01:00');
for (var preference of ['prefer', 'reject']) {
  var caught = undefined;
  try { previousLocalDay.with({offset: '-01:00'}, {offset: preference}); } catch (e) { caught = e; }
  if (!(caught instanceof RangeError)) throw 'original ISO day range';
}
for (var preference of ['use', 'ignore']) {
  if (previousLocalDay.with({offset: '-01:00'}, {offset: preference}).epochNanoseconds !== minimum.epochNanoseconds) throw 'balanced ISO day range';
}
true;
"#,
    );
}

#[test]
fn zoned_to_plain_date_uses_the_local_iso_date_and_preserves_the_calendar() {
    assert_zoned_with(
        r#"
var method = Temporal.ZonedDateTime.prototype.toPlainDate;
var descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, 'toPlainDate');
if (method.name !== 'toPlainDate' || method.length !== 0 || !descriptor.writable ||
    descriptor.enumerable || !descriptor.configurable || Object.hasOwn(method, 'prototype')) throw 'date metadata';
class Derived extends Temporal.ZonedDateTime {}
for (var calendar of ['iso8601', 'gregory', 'buddhist']) {
  var zdt = new Derived(-1n, '-01:00', calendar);
  Object.defineProperty(zdt, 'constructor', {get() { throw 'constructor read'; }});
  var date = zdt.toPlainDate();
  if (Object.getPrototypeOf(date) !== Temporal.PlainDate.prototype || date.calendarId !== calendar ||
      date.year !== (calendar === 'buddhist' ? 2512 : 1969) || date.month !== 12 || date.day !== 31 ||
      !date.equals(zdt.toPlainDateTime().toPlainDate())) throw 'local calendar date';
}
var forward = new Temporal.ZonedDateTime(-1n, '+01:00').toPlainDate();
if (forward.year !== 1970 || forward.month !== 1 || forward.day !== 1) throw 'offset date rollover';
var earliest = new Temporal.ZonedDateTime(-8640000000000000000000n, '-01:00').toPlainDate();
if (earliest.year !== -271821 || earliest.month !== 4 || earliest.day !== 19) throw 'minimum local date';
var caught = undefined;
try { method.call({}); } catch (e) { caught = e; }
if (!(caught instanceof TypeError)) throw 'date branding';
true;
"#,
    );
}
