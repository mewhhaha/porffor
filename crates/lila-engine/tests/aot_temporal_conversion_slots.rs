use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
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
        .expect("Temporal conversion must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn plain_date_and_time_conversion_read_temporal_slots() {
    assert_wasm_true(
        r#"
const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
for (const name of ['year', 'month', 'day', 'hour', 'minute', 'second', 'millisecond', 'microsecond', 'nanosecond', 'calendarId']) {
  Object.defineProperty(Temporal.PlainDateTime.prototype, name, {
    get() { throw new Error('PlainDateTime getter: ' + name); }
  });
}
const date = new Temporal.PlainDate(2000, 5, 2);
if (Temporal.PlainDate.from(datetime).day !== 2) throw new Error('date from datetime');
if (Temporal.PlainDate.compare(date, datetime) !== 0) throw new Error('date compare datetime');
if (!date.equals(datetime)) throw new Error('date equals datetime');
if (date.until(datetime).days !== 0 || date.since(datetime).days !== 0) throw new Error('date difference datetime');
const time = Temporal.PlainTime.from(datetime);
if (time.hour !== 12 || time.nanosecond !== 321) throw new Error('time from datetime');
let optionReads = 0;
const options = { get overflow() { optionReads++; return 'constrain'; } };
Temporal.PlainDate.from(datetime, options);
Temporal.PlainTime.from(datetime, options);
if (optionReads !== 2) throw new Error('datetime overflow option');

const zoned = new Temporal.ZonedDateTime(0n, '-01:00');
for (const name of ['year', 'month', 'day', 'hour', 'minute', 'second', 'millisecond', 'microsecond', 'nanosecond', 'calendarId', 'timeZoneId']) {
  Object.defineProperty(Temporal.ZonedDateTime.prototype, name, {
    get() { throw new Error('ZonedDateTime getter: ' + name); }
  });
}
const localDate = new Temporal.PlainDate(1969, 12, 31);
if (Temporal.PlainDate.from(zoned).day !== 31) throw new Error('date from zoned');
if (Temporal.PlainDate.compare(localDate, zoned) !== 0) throw new Error('date compare zoned');
if (!localDate.equals(zoned)) throw new Error('date equals zoned');
if (localDate.until(zoned).days !== 0 || localDate.since(zoned).days !== 0) throw new Error('date difference zoned');
Temporal.PlainDate.from(zoned, options);
if (optionReads !== 3) throw new Error('zoned overflow option');
true;
"#,
    );
}

#[test]
fn plain_date_with_observes_inherited_calendar_and_time_zone_getters() {
    assert_wasm_true(
        r#"
let order = '';
const prototype = {
  get calendar() { order += 'C'; return undefined; },
  get timeZone() { order += 'T'; return undefined; }
};
const fields = Object.create(prototype);
Object.defineProperty(fields, 'day', { get() { order += 'D'; return 3; } });
const date = new Temporal.PlainDate(2000, 5, 2).with(fields);
if (order !== 'CTD' || date.day !== 3) throw new Error('calendar/timeZone read order');
true;
"#,
    );
}

#[test]
fn instant_accepts_short_offset_time_zone_annotations() {
    assert_wasm_true(
        r#"
for (const annotation of ['+00', '-00', '+12', '+0130', '+01:30', '-00:00']) {
  const instant = Temporal.Instant.from('1976-11-18T15:23:30.123456789Z[' + annotation + ']');
  if (instant.epochNanoseconds !== 217178610123456789n) throw new Error(annotation);
}
for (const annotation of ['+ab', '+24', '+01:60', '+0', '+000', '+00000', '+01-00', '+01:0', '+01:00x']) {
  let threw = false;
  try { Temporal.Instant.from('1976-11-18T15:23:30.123456789Z[' + annotation + ']'); }
  catch (error) { if (!(error instanceof RangeError)) throw error; threw = true; }
  if (!threw) throw new Error('invalid annotation accepted: ' + annotation);
}
true;
"#,
    );
}

#[test]
fn zoned_constructor_requires_a_time_zone_identifier() {
    assert_wasm_true(
        r#"
let threw = false;
try { new Temporal.ZonedDateTime(0n, '1997-12-04T12:34[+01:00]'); }
catch (error) { if (!(error instanceof RangeError)) throw error; threw = true; }
if (!threw) throw new Error('ISO date-time accepted as time zone');
true;
"#,
    );
}

#[test]
fn plain_date_conversion_methods_plan_their_own_time_zone_import() {
    // Separate programs ensure another Temporal method cannot supply the import.
    for source in [
        "Temporal.PlainDate.from('2000-01-01').year === 2000;",
        "Temporal.PlainDate.compare('2000-01-01', '2000-01-02') === -1;",
        "new Temporal.PlainDate(2000, 1, 1).equals('2000-01-01');",
        "new Temporal.PlainDate(2000, 1, 1).until('2000-01-02').days === 1;",
        "new Temporal.PlainDate(2000, 1, 1).since('2000-01-02').days === -1;",
    ] {
        assert_wasm_true(source);
    }
}
