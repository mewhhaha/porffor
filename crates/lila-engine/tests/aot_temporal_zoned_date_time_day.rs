use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_zoned_day_semantics(source: &str) {
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
        .expect("ZonedDateTime day semantics must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn transition_queries_validate_direction_before_returning_no_transition() {
    assert_zoned_day_semantics(
        r#"
const transition = Temporal.ZonedDateTime.prototype.getTimeZoneTransition;
const descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, 'getTimeZoneTransition');
if (transition.length !== 1 || transition.name !== 'getTimeZoneTransition' || !descriptor.writable || descriptor.enumerable || !descriptor.configurable) throw new Error('transition descriptor');
if (Object.getPrototypeOf(transition) !== Function.prototype || transition.hasOwnProperty('prototype')) throw new Error('transition builtin shape');
for (const zone of ['UTC', '+00', '-01:30', '+23:59']) {
  for (const epoch of [0n, -1n, -8640000000000000000000n, 8640000000000000000000n]) {
    const value = new Temporal.ZonedDateTime(epoch, zone);
    for (const direction of ['next', 'previous']) {
      if (transition.call(value, direction) !== null || value.getTimeZoneTransition({direction}) !== null) throw new Error('no transition');
    }
  }
}
const value = new Temporal.ZonedDateTime(0n, 'UTC');
function expect(errorType, operation) {
  let error;
  try { operation(); } catch (caught) { error = caught; }
  if (!(error instanceof errorType)) throw new Error('direction error type');
}
for (const direction of [undefined, null, 1, 1n, true, Symbol()]) {
  expect(TypeError, () => value.getTimeZoneTransition(direction));
}
for (const direction of [undefined, null, 1, 1n, true, 'NEXT', 'following', 'next\0', 'prevıous']) {
  expect(RangeError, () => value.getTimeZoneTransition({direction}));
}
expect(RangeError, () => value.getTimeZoneTransition({}));
expect(RangeError, () => value.getTimeZoneTransition(() => {}));
expect(RangeError, () => value.getTimeZoneTransition(new String('next')));
expect(TypeError, () => value.getTimeZoneTransition({direction:Symbol()}));
expect(TypeError, () => new transition('next'));
true;
"#,
    );
}

#[test]
fn transition_options_preserve_get_and_string_coercion_order_and_abrupt_identity() {
    assert_zoned_day_semantics(
        r#"
const value = new Temporal.ZonedDateTime(0n, '-01:00');
const log = [];
const options = {
  get direction() {
    log.push('get direction');
    return {
      get toString() { log.push('get toString'); return () => { log.push('call toString'); return 'previous'; }; },
      get valueOf() { throw new Error('unwanted valueOf'); }
    };
  }
};
if (value.getTimeZoneTransition(options) !== null || log.join(',') !== 'get direction,get toString,call toString') throw new Error(log.join(','));
const marker = {};
for (const options of [
  {get direction() { throw marker; }},
  {direction:{toString() { throw marker; }}}
]) {
  let received;
  try { value.getTimeZoneTransition(options); } catch (error) { received = error; }
  if (received !== marker) throw new Error('abrupt identity');
}
let reads = 0;
let received;
try { Temporal.ZonedDateTime.prototype.getTimeZoneTransition.call({}, {get direction() { reads++; throw marker; }}); }
catch (error) { received = error; }
if (!(received instanceof TypeError) || reads !== 0) throw new Error('brand before direction');
const callable = () => {};
callable.direction = 'next';
const array = [];
array.direction = 'previous';
if (value.getTimeZoneTransition(callable) !== null || value.getTimeZoneTransition(array) !== null) throw new Error('object option domain');
true;
"#,
    );
}

#[test]
fn start_of_day_uses_exact_negative_and_heap_epoch_values_with_local_offsets() {
    assert_zoned_day_semantics(
        r#"
const day = 86400000000000n;
const limit = 8640000000000000000000n;
const cases = [
  [0n, 'UTC', 0n], [-1n, 'UTC', -day],
  [0n, '+01:00', -3600000000000n], [0n, '-01:00', -82800000000000n],
  [0n, '+23:59', -86340000000000n], [0n, '-23:59', -60000000000n],
  [10000n * day + 7272123456789n, 'UTC', 10000n * day],
  [-10000n * day - 1n, 'UTC', -10001n * day],
  [limit, 'UTC', limit], [limit - 1n, 'UTC', limit - day], [-limit, 'UTC', -limit]
];
for (const [epoch, zone, expected] of cases) {
  const value = new Temporal.ZonedDateTime(epoch, zone, 'gregory');
  const result = value.startOfDay();
  if (result === value || result.epochNanoseconds !== expected || result.timeZoneId !== value.timeZoneId || result.calendarId !== 'gregory') throw new Error('start of day exact value');
  if (result.hour !== 0 || result.minute !== 0 || result.second !== 0 || result.millisecond !== 0 || result.microsecond !== 0 || result.nanosecond !== 0) throw new Error('not midnight');
  if (value.epochNanoseconds !== epoch) throw new Error('mutated receiver');
}
true;
"#,
    );
}

#[test]
fn day_methods_validate_both_midnight_endpoints_at_instant_limits() {
    assert_zoned_day_semantics(
        r#"
const limit = 8640000000000000000000n;
for (const zone of ['UTC', '+00', '+01:00', '-01:00']) {
  for (const epoch of [0n, 86400000000000n, -86400000000000n, -1n]) {
    if (new Temporal.ZonedDateTime(epoch, zone).hoursInDay !== 24) throw new Error('ordinary day length');
  }
}
function rangeError(operation) {
  let error;
  try { operation(); } catch (caught) { error = caught; }
  if (!(error instanceof RangeError)) throw new Error('day endpoint range');
}
for (const zone of ['-01:00', '+01:00', '-23:59', '+23:59']) {
  const earliest = new Temporal.ZonedDateTime(-limit, zone);
  rangeError(() => earliest.startOfDay());
  rangeError(() => earliest.hoursInDay);
}
for (const zone of ['UTC', '-01:00', '+01:00', '-23:59', '+23:59']) {
  const latest = new Temporal.ZonedDateTime(limit, zone);
  latest.startOfDay();
  rangeError(() => latest.hoursInDay);
}
if (new Temporal.ZonedDateTime(-limit, 'UTC').hoursInDay !== 24) throw new Error('first complete UTC day');
if (new Temporal.ZonedDateTime(limit - 1n, 'UTC').hoursInDay !== 24) throw new Error('last complete UTC day');
true;
"#,
    );
}

#[test]
fn day_methods_use_internal_slots_and_intrinsic_results_with_brand_checks() {
    assert_zoned_day_semantics(
        r#"
const value = new Temporal.ZonedDateTime(-1n, '+01:00', 'gregory');
for (const key of ['constructor', 'timeZone', 'calendar', 'epochNanoseconds', 'year', 'hour']) {
  Object.defineProperty(value, key, {get() { throw new Error('public property read: ' + key); }});
}
const result = value.startOfDay();
if (Object.getPrototypeOf(result) !== Temporal.ZonedDateTime.prototype || result.calendarId !== 'gregory' || result.timeZoneId !== '+01:00' || value.hoursInDay !== 24) throw new Error('internal slot result');
const getter = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, 'hoursInDay');
if (getter.set !== undefined || getter.enumerable || !getter.configurable || getter.get.length !== 0 || getter.get.name !== 'get hoursInDay') throw new Error('hours getter descriptor');
const start = Temporal.ZonedDateTime.prototype.startOfDay;
if (start.length !== 0 || start.name !== 'startOfDay' || start.hasOwnProperty('prototype')) throw new Error('start method metadata');
for (const receiver of [undefined, null, true, '', Symbol(), 1, 1n, {}, Temporal.ZonedDateTime.prototype, new Proxy(value, {})]) {
  for (const operation of [() => getter.get.call(receiver), () => start.call(receiver)]) {
    let error;
    try { operation(); } catch (caught) { error = caught; }
    if (!(error instanceof TypeError)) throw new Error('day receiver brand');
  }
}
true;
"#,
    );
}
