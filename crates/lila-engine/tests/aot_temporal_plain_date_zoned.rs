use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_plain_date_zoned(source: &str) {
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
        .expect("PlainDate conversion must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn omitted_and_explicit_times_preserve_date_calendar_and_nanoseconds() {
    assert_plain_date_zoned(
        r#"
const date = new Temporal.PlainDate(1970, 1, 1);
if (date.toZonedDateTime('UTC').epochNanoseconds !== 0n) throw new Error('midnight');
if (date.toZonedDateTime('+01:30').epochNanoseconds !== -5400000000000n) throw new Error('fixed midnight');
const explicit = date.toZonedDateTime({timeZone: '-01:30', plainTime: '01:02:03.004005006'});
if (explicit.epochNanoseconds !== 9123004005006n || explicit.calendarId !== 'iso8601') throw new Error('explicit clock');
const zone = new Temporal.ZonedDateTime(0n, '+02:00');
if (date.toZonedDateTime(zone).epochNanoseconds !== -7200000000000n) throw new Error('branded time zone');
if (date.toZonedDateTime({timeZone: zone}).epochNanoseconds !== -7200000000000n) throw new Error('branded nested zone');
true;
"#,
    );
}

#[test]
fn conversion_observes_zone_before_time_and_preserves_abrupt_identity() {
    assert_plain_date_zoned(
        r#"
const date = new Temporal.PlainDate(2000, 2, 29);
let trace = '';
const result = date.toZonedDateTime({
  get timeZone() { trace += 'Z'; return '+00:00'; },
  get plainTime() { trace += 'T'; return {get hour() { trace += 'H'; return 3; }}; }
});
if (trace !== 'ZTH' || result.hour !== 3 || result.day !== 29) throw new Error('getter order');
let timeReads = 0;
try { date.toZonedDateTime({timeZone: 'invalid', get plainTime() { timeReads++; }}); }
catch (error) { if (!(error instanceof RangeError)) throw error; }
if (timeReads !== 0) throw new Error('time observed before zone validation');
const marker = {};
let caught;
try { date.toZonedDateTime({timeZone: 'UTC', get plainTime() { throw marker; }}); }
catch (error) { caught = error; }
if (caught !== marker) throw new Error('getter throw');
true;
"#,
    );
}

#[test]
fn conversion_and_calendar_arithmetic_enforce_both_epoch_boundaries() {
    assert_plain_date_zoned(
        r#"
function range(action) { let caught; try { action(); } catch (error) { caught = error; }
  if (!(caught instanceof RangeError)) throw new Error('missing range error'); }
const maximum = new Temporal.PlainDate(275760, 9, 13);
if (maximum.toZonedDateTime('UTC').epochNanoseconds !== 8640000000000000000000n) throw new Error('max midnight');
range(() => maximum.toZonedDateTime({timeZone: 'UTC', plainTime: '00:00:00.000000001'}));
range(() => new Temporal.PlainDate(-271821, 4, 19).toZonedDateTime('UTC'));
const maximumYear = new Temporal.PlainDate(275760, 1, 1).toZonedDateTime('UTC');
const duration = new Temporal.Duration(0, 5432, 5432);
range(() => maximumYear.add(duration));
range(() => new Temporal.ZonedDateTime(-8640000000000000000000n, 'UTC').subtract(duration));
true;
"#,
    );
}
