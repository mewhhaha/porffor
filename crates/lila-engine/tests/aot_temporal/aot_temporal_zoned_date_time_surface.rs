use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_zoned_semantics(source: &str) {
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
        .expect("ZonedDateTime semantics must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn compares_exact_epoch_bigints_and_publishes_static_method_contract() {
    assert_zoned_semantics(
        r#"
const compare = Temporal.ZonedDateTime.compare;
const descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime, 'compare');
if (compare.name !== 'compare' || compare.length !== 2 || !descriptor.writable || descriptor.enumerable || !descriptor.configurable) throw new Error('method descriptor');
if (Object.getPrototypeOf(compare) !== Function.prototype || compare.hasOwnProperty('prototype')) throw new Error('builtin function shape');
const cases = [
  [-8640000000000000000000n, -8639999999999999999999n],
  [-1n, 0n], [0n, 1n], [9007199254740992n, 9007199254740993n],
  [8639999999999999999999n, 8640000000000000000000n]
];
for (const [left, right] of cases) {
  const one = new Temporal.ZonedDateTime(left, '+23:59');
  const two = new Temporal.ZonedDateTime(right, '-23:59', 'gregory');
  if (compare.call(null, one, two) !== -1 || compare(two, one) !== 1 || compare(one, one) !== 0) throw new Error('exact comparison');
}
if (compare('2000-01-01T01:00+01:00[+01:00]', '2000-01-01T00:00Z[UTC]') !== 0) throw new Error('offset equality');
let error;
try { new compare(0, 0); } catch (caught) { error = caught; }
if (!(error instanceof TypeError)) throw new Error('compare must not construct');
true;
"#,
    );
}

#[test]
fn converts_arguments_sequentially_and_preserves_abrupt_completions() {
    assert_zoned_semantics(
        r#"
const log = [];
function bag(label) {
  const fields = {calendar:'iso8601',day:2,hour:3,microsecond:4,millisecond:5,minute:6,month:7,monthCode:'M07',nanosecond:8,offset:'+00:00',second:9,timeZone:'UTC',year:2000};
  return new Proxy(fields, {get(target, key) {
    log.push(label + '.' + key);
    const value = target[key];
    if (typeof value === 'number') return {valueOf() { log.push(label + '.' + key + '.valueOf'); return value; }};
    if (key === 'monthCode' || key === 'offset') return {toString() { log.push(label + '.' + key + '.toString'); return value; }};
    return value;
  }});
}
if (Temporal.ZonedDateTime.compare(bag('one'), bag('two')) !== 0) throw new Error('bag comparison');
const fields = 'calendar,day,day.valueOf,hour,hour.valueOf,microsecond,microsecond.valueOf,millisecond,millisecond.valueOf,minute,minute.valueOf,month,month.valueOf,monthCode,monthCode.toString,nanosecond,nanosecond.valueOf,offset,offset.toString,second,second.valueOf,timeZone,year,year.valueOf'.split(',');
const expected = fields.map(field => 'one.' + field).concat(fields.map(field => 'two.' + field));
if (log.join('|') !== expected.join('|')) throw new Error(log.join('|'));
let secondReads = 0;
const marker = {};
let received;
try { Temporal.ZonedDateTime.compare({get calendar() { throw marker; }}, {get calendar() { secondReads++; }}); }
catch (error) { received = error; }
if (received !== marker || secondReads !== 0) throw new Error('first conversion abrupt');
const branded = new Temporal.ZonedDateTime(0n, 'UTC');
Object.defineProperty(branded, 'calendar', {get() { throw marker; }});
Object.defineProperty(branded, 'epochNanoseconds', {get() { throw marker; }});
if (Temporal.ZonedDateTime.compare(branded, '1970-01-01T00:00Z[UTC]') !== 0) throw new Error('internal slot conversion');
true;
"#,
    );
}

#[test]
fn validates_month_code_primitive_and_syntax_before_later_fields() {
    assert_zoned_semantics(
        r#"
function check(convert) {
  for (const monthCode of [5, 5n, false, Symbol(), null, {toString() { return 5; }}]) {
    let error;
    try { convert({year:2026, monthCode, day:1, timeZone:'UTC'}); } catch (caught) { error = caught; }
    if (!(error instanceof TypeError)) throw new Error('month-code primitive type');
  }
  let reads = 0;
  let error;
  try { convert({day:1, monthCode:'L99M', timeZone:'UTC', get year() { reads++; throw {}; }}); }
  catch (caught) { error = caught; }
  if (!(error instanceof RangeError) || reads !== 0) throw new Error('syntax before year');
  const marker = {};
  try { convert({day:1, monthCode:'M99L', timeZone:'UTC', get year() { reads++; throw marker; }}); }
  catch (caught) { error = caught; }
  if (error !== marker || reads !== 1) throw new Error('suitability after year');
  const boxed = {year:2026, monthCode:new String('M05'), day:1, timeZone:'UTC'};
  convert(boxed);
}
check(value => Temporal.ZonedDateTime.from(value));
check(value => Temporal.ZonedDateTime.compare(value, '2026-05-01T00:00Z[UTC]'));
true;
"#,
    );
}

#[test]
fn validates_original_iso_date_only_for_explicit_offset_prefer_and_reject() {
    assert_zoned_semantics(
        r#"
const minimum = -8640000000000000000000n;
const belowWall = '-271821-04-19T23:00-01:00[-01:00]';
const fields = {year:-271821, month:4, day:19, hour:23, timeZone:'-01:00', offset:'-01:00'};
for (const offset of ['use', 'ignore']) {
  if (Temporal.ZonedDateTime.from(belowWall, {offset}).epochNanoseconds !== minimum) throw new Error('balanced string minimum');
  if (Temporal.ZonedDateTime.from(fields, {offset}).epochNanoseconds !== minimum) throw new Error('balanced bag minimum');
}
for (const offset of ['prefer', 'reject']) {
  for (const input of [belowWall, fields]) {
    let error;
    try { Temporal.ZonedDateTime.from(input, {offset}); } catch (caught) { error = caught; }
    if (!(error instanceof RangeError)) throw new Error('original date range');
  }
}
for (const offset of ['use', 'ignore', 'prefer', 'reject']) {
  if (Temporal.ZonedDateTime.from('-271821-04-19T23:00[-01:00]', {offset}).epochNanoseconds !== minimum) throw new Error('wall behavior');
  if (Temporal.ZonedDateTime.from('+275760-09-13T23:59+23:59[+23:59]', {offset}).epochNanoseconds !== -minimum) throw new Error('last ISO day');
  let error;
  try { Temporal.ZonedDateTime.from('-271821-04-19T22:59-01:00[-01:00]', {offset}); } catch (caught) { error = caught; }
  if (!(error instanceof RangeError)) throw new Error('epoch range');
}
let error;
try { Temporal.ZonedDateTime.compare(belowWall, new Temporal.ZonedDateTime(minimum, 'UTC')); } catch (caught) { error = caught; }
if (!(error instanceof RangeError)) throw new Error('compare default offset policy');
true;
"#,
    );
}

#[test]
fn calendar_getters_use_local_date_and_preserve_result_kinds_and_brands() {
    assert_zoned_semantics(
        r#"
const value = new Temporal.ZonedDateTime(217178610123456789n, 'UTC');
const expected = {dayOfWeek:4, dayOfYear:323, weekOfYear:47, yearOfWeek:1976, daysInWeek:7, daysInMonth:30, daysInYear:366, monthsInYear:12, inLeapYear:true};
for (const key of Object.keys(expected)) {
  if (value[key] !== expected[key]) throw new Error(key);
  const descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, key);
  if (descriptor.set !== undefined || descriptor.enumerable || !descriptor.configurable || descriptor.get.length !== 0 || descriptor.get.name !== 'get ' + key) throw new Error('getter descriptor');
  for (const receiver of [{}, new Proxy(value, {}), 1, null]) {
    let error;
    try { descriptor.get.call(receiver); } catch (caught) { error = caught; }
    if (!(error instanceof TypeError)) throw new Error('getter brand ' + key);
  }
}
const january = Temporal.ZonedDateTime.from('2021-01-01T00:30Z[UTC]');
const december = january.withTimeZone('-01:00');
if (january.dayOfYear !== 1 || january.weekOfYear !== 53 || january.yearOfWeek !== 2020 || january.inLeapYear !== false) throw new Error('January ISO week');
if (december.dayOfYear !== 366 || december.dayOfWeek !== 4 || december.daysInYear !== 366 || december.inLeapYear !== true) throw new Error('local previous year');
const gregory = december.withCalendar('gregory');
if (gregory.weekOfYear !== undefined || gregory.yearOfWeek !== undefined || gregory.inLeapYear !== true) throw new Error('calendar week convention');
true;
"#,
    );
}

#[test]
fn calendar_string_probes_preserve_partial_date_goal_restrictions() {
    assert_zoned_semantics(
        r#"
for (const calendar of ['2000-05[u-ca=gregory]', '05-02[u-ca=gregory]']) {
  let error;
  try { Temporal.ZonedDateTime.from({year:2026, month:5, day:1, timeZone:'UTC', calendar}); }
  catch (caught) { error = caught; }
  if (!(error instanceof RangeError)) throw new Error('non-ISO partial calendar string');
}
for (const calendar of ['2000-05[u-ca=iso8601]', '05-02[u-ca=iso8601]', '2000-05-02[u-ca=gregory]']) {
  const value = Temporal.ZonedDateTime.from({year:2026, month:5, day:1, timeZone:'UTC', calendar});
  if (value.month !== 5 || value.day !== 1) throw new Error('valid calendar string');
}
true;
"#,
    );
}
