use super::*;

#[test]
fn intl_artifact_identity_gate_is_closed_before_wasmtime() {
    let expected = scriptw_intl::embedded_intl_data_identity()
        .expect("embedded Intl identity should be valid");
    let identity = expected.artifact_identity();
    let identity = identity.as_bytes();

    validate_wasm_intl_artifact_identity(
        &inert_wasm_with_intl_identity(true, &[identity]),
        &expected,
        IntlArtifactIdentityRequirement::HostImports,
    )
    .expect("one matching section should satisfy the Intl import");
    validate_wasm_intl_artifact_identity(
        &inert_wasm_with_intl_identity(false, &[]),
        &expected,
        IntlArtifactIdentityRequirement::HostImports,
    )
    .expect("an Intl-free artifact needs no identity");

    for (artifact, expected_error) in [
        (
            inert_wasm_with_intl_identity(true, &[]),
            IntlArtifactIdentityError::MissingSection,
        ),
        (
            inert_wasm_with_intl_identity(true, &[identity, identity]),
            IntlArtifactIdentityError::DuplicateSections,
        ),
        (
            inert_wasm_with_intl_identity(true, &[b"different identity"]),
            IntlArtifactIdentityError::IdentityMismatch,
        ),
        (
            inert_wasm_with_intl_identity(false, &[identity]),
            IntlArtifactIdentityError::UnexpectedSection,
        ),
    ] {
        let error = validate_wasm_intl_artifact_identity(
            &artifact,
            &expected,
            IntlArtifactIdentityRequirement::HostImports,
        )
        .expect_err("invalid Intl artifact relation must be rejected");
        assert_eq!(
            error.intl_artifact_identity_error(),
            Some(expected_error),
            "{error}"
        );
    }
}

#[test]
fn wasm_backend_uses_one_injected_clock_for_date_temporal_and_monotonic_reads() {
    const WALL_CLOCK_MILLISECONDS: i64 = 1_640_995_200_123;
    let clock = DeterministicHostClock {
        wall_clock: UtcEpochMilliseconds::new(WALL_CLOCK_MILLISECONDS)
            .expect("test wall clock is in range"),
        next_monotonic_nanoseconds: AtomicU64::new(1_000_000_000),
        monotonic_step_nanoseconds: 5_000_000,
    };
    let engine = Engine::new(RealmBuilder::new().with_host_clock(Box::new(clock)).build());
    let outcome = engine
        .run_script(
            r#"
let dateArgumentConversions = 0;
const ignoredDateArgument = {
  valueOf() {
    dateArgumentConversions += 1;
    throw new Error("Date function arguments must not be converted");
  }
};
class ClockDate extends Date {}
const currentDateString = "Sat Jan 01 2022 00:00:00 GMT+0000 (Coordinated Universal Time)";

Date.now() === 1640995200123
  && new Date().getTime() === 1640995200123
  && new ClockDate().getTime() === 1640995200123
  && Date() === currentDateString
  && Date(ignoredDateArgument) === currentDateString
  && dateArgumentConversions === 0
  && Temporal.Now.instant().epochMilliseconds === 1640995200123
  && __scriptwAgentMonotonicNow() === 5
  && __scriptwAgentMonotonicNow() === 10;
"#,
            test262_compile_options(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Wasm host clock reads should use the injected realm clock");

    assert!(
        outcome.note.contains("boolean(true)"),
        "unexpected result: {}",
        outcome.note
    );
}

#[test]
fn wasm_intl_canonical_locale_aliases_are_resolved_before_deduplication() {
    let outcome = engine()
        .run_script(
            r#"
const locales = Intl.getCanonicalLocales(["iw-IL", "he-IL"]);
locales.length === 1 && locales[0] === "he-IL";
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Wasm Intl locale canonicalization should use the embedded provider");

    assert!(
        outcome.note.contains("boolean(true)"),
        "unexpected result: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_brands_temporal_plain_dates_and_derives_iso_calendar_fields() {
    let outcome = engine()
        .run_script(
            r#"
var date = new Temporal.PlainDate(2020, 5, 2);
if (Object.getPrototypeOf(date) !== Temporal.PlainDate.prototype) throw "prototype";
if (Object.prototype.toString.call(date) !== "[object Temporal.PlainDate]") throw "tag";
if (date.year !== 2020 || date.month !== 5 || date.day !== 2) throw "iso fields";
if (date.monthCode !== "M05") throw "monthCode";
if (date.calendarId !== "iso8601") throw "calendarId";

// Derived ISO fields. 2020-05-02 is a Saturday; 2020 is a leap year, so the
// day-of-year count includes 29 February.
if (date.dayOfWeek !== 6) throw "dayOfWeek";
if (date.dayOfYear !== 123) throw "dayOfYear";
if (date.weekOfYear !== 18 || date.yearOfWeek !== 2020) throw "weekOfYear";
if (date.daysInWeek !== 7 || date.monthsInYear !== 12) throw "constant fields";
if (date.daysInMonth !== 31 || date.daysInYear !== 366) throw "length fields";
if (date.inLeapYear !== true) throw "inLeapYear";

// The ISO 8601 calendar has no eras.
if (date.era !== undefined || date.eraYear !== undefined) throw "era";

// The ISO week-numbering year runs ahead of the calendar year at the boundary:
// 1977-01-01 belongs to week 53 of 1976.
var boundary = new Temporal.PlainDate(1977, 1, 1);
if (boundary.weekOfYear !== 53 || boundary.yearOfWeek !== 1976) throw "week boundary";

// Pre-epoch dates must floor, not truncate, when reducing days to a weekday.
if (new Temporal.PlainDate(1969, 12, 31).dayOfWeek !== 3) throw "pre-epoch dayOfWeek";

// `RejectISODate` plus `ISODateWithinLimits`: one more day is representable
// below the epoch-day limit than above it.
if (new Temporal.PlainDate(-271821, 4, 19).toString() !== "-271821-04-19") throw "min";
if (new Temporal.PlainDate(275760, 9, 13).toString() !== "+275760-09-13") throw "max";
var limitErrors = 0;
for (var argv of [[-271821, 4, 18], [275760, 9, 14], [2020, 13, 1], [2021, 2, 29]]) {
  try {
    new Temporal.PlainDate(argv[0], argv[1], argv[2]);
  } catch (error) {
    if (error instanceof RangeError) limitErrors += 1;
  }
}
if (limitErrors !== 4) throw "limits";

// Temporal forbids implicit comparison outright.
var valueOfThrew = false;
try { date.valueOf(); } catch (error) { valueOfThrew = error instanceof TypeError; }
if (!valueOfThrew) throw "valueOf";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should brand Temporal.PlainDate instances");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_converts_strings_and_property_bags_to_temporal_plain_dates() {
    let outcome = engine()
        .run_script(
            r#"
if (Temporal.PlainDate.from("2020-05-02").toString() !== "2020-05-02") throw "string";
if (Temporal.PlainDate.from("2020-05-02T09:00:00").toString() !== "2020-05-02") throw "date-time";
if (Temporal.PlainDate.from("2020-05-02T09:00:00+01:00").toString() !== "2020-05-02") {
  throw "offset is ignored";
}
if (Temporal.PlainDate.from("2020-05-02[America/New_York]").toString() !== "2020-05-02") {
  throw "time zone annotation is accepted without being resolved";
}
// Only the FIRST calendar annotation is resolved, but a repeated critical one
// is still rejected.
if (Temporal.PlainDate.from("2000-05-02T15:23[u-ca=iso8601][u-ca=discord]").toString()
    !== "2000-05-02") {
  throw "second calendar annotation";
}

var rangeErrors = 0;
for (var text of [
  "2019-10-01T09:00:00Z",
  "2019-10-01T09:00:00Z[UTC]",
  "1970-01-01[u-ca=iso8601][!u-ca=iso8601]",
  "1970-01-01[u-ca=notacal]",
  "-000000-10-31",
  "",
]) {
  try { Temporal.PlainDate.from(text); } catch (error) {
    if (error instanceof RangeError) rangeErrors += 1;
  }
}
if (rangeErrors !== 6) throw "string rejections";

var typeErrors = 0;
for (var value of [undefined, null, true, 1, 1n]) {
  try { Temporal.PlainDate.from(value); } catch (error) {
    if (error instanceof TypeError) typeErrors += 1;
  }
}
if (typeErrors !== 5) throw "primitive rejections";

if (Temporal.PlainDate.from({ year: 2000, month: 5, day: 2 }).toString() !== "2000-05-02") {
  throw "property bag";
}
if (Temporal.PlainDate.from({ year: 2000, monthCode: "M05", day: 2 }).toString()
    !== "2000-05-02") {
  throw "monthCode bag";
}
if (Temporal.PlainDate.from({ year: 2021, month: 1, day: 32 }).toString() !== "2021-01-31") {
  throw "overflow constrain";
}
var rejected = false;
try {
  Temporal.PlainDate.from({ year: 2021, month: 1, day: 32 }, { overflow: "reject" });
} catch (error) { rejected = error instanceof RangeError; }
if (!rejected) throw "overflow reject";

// `CalendarResolveFields` validates presence before range, so a missing
// required key is a TypeError even when another field is also out of range.
var missingYear = false;
try { Temporal.PlainDate.from({ monthCode: "M99L", day: 1 }); } catch (error) {
  missingYear = error instanceof TypeError;
}
if (!missingYear) throw "missing year is a TypeError";
var badMonthCode = false;
try { Temporal.PlainDate.from({ year: 2021, monthCode: "M99L", day: 1 }); } catch (error) {
  badMonthCode = error instanceof RangeError;
}
if (!badMonthCode) throw "invalid monthCode is a RangeError";

if (Temporal.PlainDate.compare("2000-05-02", "2000-05-03") !== -1) throw "compare lt";
if (Temporal.PlainDate.compare("2001-05-02", "2000-05-03") !== 1) throw "compare gt";
if (Temporal.PlainDate.compare("2000-05-02", "2000-05-02") !== 0) throw "compare eq";

var date = new Temporal.PlainDate(2000, 5, 2);
if (!date.equals({ year: 2000, month: 5, day: 2 })) throw "equals";
if (date.with({ month: 2, day: 31 }).toString() !== "2000-02-29") throw "with constrain";
if (date.withCalendar("iso8601").toString() !== "2000-05-02") throw "withCalendar";
if (date.toString({ calendarName: "always" }) !== "2000-05-02[u-ca=iso8601]") throw "always";
if (date.toString({ calendarName: "critical" }) !== "2000-05-02[!u-ca=iso8601]") throw "critical";
if (date.toJSON() !== "2000-05-02") throw "toJSON";

// `with` takes a partial date, never a calendar or a time zone.
var withErrors = 0;
for (var bag of [{}, { calendar: "iso8601" }, { timeZone: "UTC" }]) {
  try { date.with(bag); } catch (error) {
    if (error instanceof TypeError) withErrors += 1;
  }
}
if (withErrors !== 3) throw "with rejections";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should convert values to Temporal.PlainDate");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_temporal_month_code_preserves_toprimitive_throw_identity() {
    let outcome = engine()
        .run_script(
            r#"
var sentinel = {};
var caught = false;
try {
  Temporal.PlainYearMonth.from({
    year: 2000,
    monthCode: { toString: function() { throw sentinel; } }
  });
} catch (error) {
  caught = error === sentinel;
}
caught;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Temporal monthCode coercion should preserve the thrown value");
    assert!(
        outcome.note.contains("boolean(true)"),
        "unexpected result: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_exposes_temporal_instant_private_epoch_slots() {
    let outcome = engine()
        .run_script(
            r#"
if (typeof Temporal !== "object") throw "Temporal";
if (typeof Temporal.Instant !== "function") throw "Instant";
if (Temporal.Instant.name !== "Instant" || Temporal.Instant.length !== 1) throw "metadata";
var positive = new Temporal.Instant(217175010123456789n);
if (!(positive instanceof Temporal.Instant)) throw "instanceof";
if (positive.epochNanoseconds !== 217175010123456789n) throw "nanoseconds";
if (positive.epochMilliseconds !== 217175010123) throw "milliseconds";
var negative = new Temporal.Instant("-217175010123456789");
if (negative.epochNanoseconds !== -217175010123456789n) throw "negative nanoseconds";
if (negative.epochMilliseconds !== -217175010124) throw "negative milliseconds";
if (new Temporal.Instant(true).epochNanoseconds !== 1n) throw "true";
if (new Temporal.Instant(false).epochNanoseconds !== 0n) throw "false";
var descriptor = Object.getOwnPropertyDescriptor(
  Temporal.Instant.prototype,
  "epochNanoseconds"
);
if (typeof descriptor.get !== "function") throw "getter";
if (descriptor.set !== undefined || descriptor.enumerable || !descriptor.configurable) {
  throw "descriptor";
}
var threw = 0;
try { descriptor.get.call({}); } catch (error) {
  if (error instanceof TypeError) threw += 1;
}
try { new Temporal.Instant("abc123"); } catch (error) {
  if (error instanceof SyntaxError) threw += 1;
}
try { Temporal.Instant(0n); } catch (error) {
  if (error instanceof TypeError) threw += 1;
}
if (threw !== 3) throw "errors";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should expose Temporal.Instant epoch slots");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_parses_temporal_instant_strings_exactly() {
    let outcome = engine()
        .run_script(
            r#"
var epoch = Temporal.Instant.from("1970-01-01T00:00Z");
if (epoch.epochNanoseconds !== 0n) throw "epoch";
if (epoch.toString() !== "1970-01-01T00:00:00Z") throw "epoch string";
var precise = Temporal.Instant.from("1976-11-18T15:23:30.123456789-02:00");
if (precise.epochNanoseconds !== 217185810123456789n) throw "precise";
if (precise.toString() !== "1976-11-18T17:23:30.123456789Z") throw "precise string";
var annotated = Temporal.Instant.from(
  "1976-11-18T15:23:30.123456Z[Europe/Vienna][u-ca=iso8601][foo=bar]"
);
if (annotated.epochNanoseconds !== 217178610123456000n) throw "annotations";
var compact = Temporal.Instant.from("+0019761118T152330.1+0000");
if (compact.epochNanoseconds !== 217178610100000000n) throw "compact";
var leap = Temporal.Instant.from("2016-12-31T23:59:60Z");
if (leap.epochNanoseconds !== 1483228799000000000n) throw "leap second";
var original = new Temporal.Instant(217175010123456789n);
var copy = Temporal.Instant.from(original);
if (copy === original || copy.epochNanoseconds !== original.epochNanoseconds) throw "copy";
var coercible = { toString: function () { return "1970-01-01T00:00Z"; } };
if (Temporal.Instant.from(coercible).epochNanoseconds !== 0n) throw "ToPrimitive";
var errors = 0;
for (var invalid of [
  "2021-02-29T00:00Z",
  "1970-01-01T00:00",
  "1970-01-01T00:00Z[!foo=bar]",
  "1970-01-01T00:00:00.1234567890Z",
  "-000000-03-30T00:45Z"
]) {
  try { Temporal.Instant.from(invalid); } catch (error) {
    if (error instanceof RangeError) errors += 1;
  }
}
try { Temporal.Instant.from(1); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { Temporal.Instant.from(Temporal.Instant.prototype); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
if (errors !== 7) throw "errors";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should parse Temporal.Instant strings exactly");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_compares_temporal_instants_by_epoch_nanoseconds() {
    let outcome = engine()
        .run_script(
            r#"
var instant = new Temporal.Instant(1000000000987654321n);
if (Temporal.Instant.prototype.equals.name !== "equals") throw "name";
if (Temporal.Instant.prototype.equals.length !== 1) throw "length";
if (instant.equals(new Temporal.Instant(1000000000987654321n)) !== true) throw "heap equal";
if (instant.equals(new Temporal.Instant(1000000000987654322n)) !== false) throw "heap unequal";
if (new Temporal.Instant(1n).equals("1970-01-01T00:00:00.000000001Z") !== true) {
  throw "string conversion";
}
if (new Temporal.Instant(1n).equals(
  new Temporal.ZonedDateTime(1n, "UTC")
) !== true) throw "zoned date time conversion";
var descriptor = Object.getOwnPropertyDescriptor(Temporal.Instant.prototype, "equals");
if (!descriptor.writable || !descriptor.configurable || descriptor.enumerable) {
  throw "descriptor";
}
var errors = 0;
try { Temporal.Instant.prototype.equals.call({}, instant); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { instant.equals(1); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.Instant(0n).equals(); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.Instant(0n).equals(new Temporal.Instant(0n)); } catch (error) {
  throw "nonconstructable setup";
}
try { new Temporal.Instant.prototype.equals(instant); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
if (errors !== 4) throw "errors";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should compare Temporal.Instant epoch nanoseconds");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_uses_branded_temporal_zoned_date_time_slots() {
    let outcome = engine()
        .run_script(
            r#"
if (typeof Temporal.ZonedDateTime !== "function") throw "constructor";
if (Temporal.ZonedDateTime.name !== "ZonedDateTime") throw "name";
if (Temporal.ZonedDateTime.length !== 2) throw "length";
var epochNanoseconds = 1000000000987654321n;
if (epochNanoseconds !== 1000000000987654321n) throw "epoch input";
var datetime = new Temporal.ZonedDateTime(epochNanoseconds, "uTc");
if (!(datetime instanceof Temporal.ZonedDateTime)) throw "instanceof";
var smallEpochNanoseconds = new Temporal.ZonedDateTime(1n, "UTC").epochNanoseconds;
if (smallEpochNanoseconds !== 1n) {
  if (smallEpochNanoseconds === 0) throw "small getter routed to epochMilliseconds";
  if (smallEpochNanoseconds === "UTC") throw "small getter routed to timeZoneId";
  if (smallEpochNanoseconds === "iso8601") throw "small getter routed to calendarId";
  if (smallEpochNanoseconds === undefined) throw "small getter returned undefined";
  throw "small epochNanoseconds";
}
if (datetime.epochNanoseconds !== 1000000000987654321n) throw "epochNanoseconds";
if (datetime.epochMilliseconds !== 1000000000987) throw "epochMilliseconds";
if (datetime.timeZoneId !== "UTC") throw "timeZoneId";
if (datetime.calendarId !== "iso8601") throw "calendarId";
if (datetime.toInstant().epochNanoseconds !== epochNanoseconds) throw "toInstant";
Object.defineProperty(datetime, "toString", {
  get: function () { throw "must not read toString"; }
});
var instant = Temporal.Instant.from(datetime);
if (instant.epochNanoseconds !== epochNanoseconds) throw "Instant.from";
var offset = new Temporal.ZonedDateTime(0n, "+01:30", "ISO8601");
if (offset.timeZoneId !== "+01:30" || offset.calendarId !== "iso8601") throw "ids";
class CustomZonedDateTime extends Temporal.ZonedDateTime {}
var custom = new CustomZonedDateTime(1n, "UTC");
if (Object.getPrototypeOf(custom) !== CustomZonedDateTime.prototype) throw "subclass prototype";
if (!(custom instanceof CustomZonedDateTime)) throw "subclass instanceof";
var descriptor = Object.getOwnPropertyDescriptor(
  Temporal.ZonedDateTime.prototype,
  "epochNanoseconds"
);
if (typeof descriptor.get !== "function" || descriptor.set !== undefined) throw "descriptor";
var errors = 0;
try { Temporal.ZonedDateTime(0n, "UTC"); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.ZonedDateTime(); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.ZonedDateTime(0n, ""); } catch (error) {
  if (error instanceof RangeError) errors += 1;
}
try { new Temporal.ZonedDateTime(0n, 1); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.ZonedDateTime(8640000000000000000001n, "UTC"); } catch (error) {
  if (error instanceof RangeError) errors += 1;
}
try { descriptor.get.call({}); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
if (errors !== 6) throw "errors";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should use branded Temporal.ZonedDateTime slots");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_parses_fixed_offset_temporal_zoned_date_times() {
    let outcome = engine()
        .run_script(
            r#"
if (typeof Temporal.ZonedDateTime.from !== "function") throw "from";
if (Temporal.ZonedDateTime.from.name !== "from") throw "name";
if (Temporal.ZonedDateTime.from.length !== 1) throw "length";
var descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime, "from");
if (!descriptor.writable || !descriptor.configurable || descriptor.enumerable) {
  throw "descriptor";
}

var utc = Temporal.ZonedDateTime.from("1970-01-01T00:00Z[UTC]");
if (utc.epochNanoseconds !== 0n || utc.timeZoneId !== "UTC") throw "UTC";
var absentOffset = Temporal.ZonedDateTime.from("1970-01-01T00:00[+01:00]");
if (absentOffset.epochNanoseconds !== -3600000000000n) throw "absent offset";
// UTCOffset permits an ASCII sign followed by an hour without minutes.
// https://tc39.es/proposal-temporal/#prod-UTCOffset
var hourOnly = Temporal.ZonedDateTime.from("1970-01-01T00:00[+01]");
if (hourOnly.epochNanoseconds !== -3600000000000n
    || hourOnly.timeZoneId !== "+01:00") throw "hour-only annotation";
var exactZ = Temporal.ZonedDateTime.from("1970-01-01T00:00Z[+01:00]");
if (exactZ.epochNanoseconds !== 0n) throw "Z exact";
var exactNumeric = Temporal.ZonedDateTime.from("1970-01-01T00:00+01:00[+01:00]");
if (exactNumeric.epochNanoseconds !== -3600000000000n) throw "numeric exact";
var dateOnly = Temporal.ZonedDateTime.from("2020-01-01[+09:00]");
if (dateOnly.epochNanoseconds !== 1577804400000000000n) throw "date only";
var leap = Temporal.ZonedDateTime.from("2016-12-31T23:59:60+00:00[UTC]");
if (leap.epochNanoseconds !== 1483228799000000000n) throw "leap second";
var expanded = Temporal.ZonedDateTime.from("-009999-11-18T15:23:30.12+00:00[UTC]");
if (expanded.epochNanoseconds !== -377677326989880000000n) throw "expanded year";
var calendar = Temporal.ZonedDateTime.from(
  "1976-11-18T00:00[UTC][u-ca=ISO8601]"
);
if (calendar.calendarId !== "iso8601") throw "calendar";

var mismatch = "1970-01-01T00:00-04:15[+01:00]";
var rejected = 0;
try { Temporal.ZonedDateTime.from(mismatch); } catch (error) {
  if (error instanceof RangeError) rejected += 1;
}
try { Temporal.ZonedDateTime.from(mismatch, { offset: "reject" }); } catch (error) {
  if (error instanceof RangeError) rejected += 1;
}
if (rejected !== 2) throw "reject mismatch";
if (Temporal.ZonedDateTime.from(mismatch, { offset: "use" }).epochNanoseconds
    !== 15300000000000n) throw "use";
if (Temporal.ZonedDateTime.from(mismatch, { offset: "prefer" }).epochNanoseconds
    !== -3600000000000n) throw "prefer";
if (Temporal.ZonedDateTime.from(mismatch, { offset: "ignore" }).epochNanoseconds
    !== -3600000000000n) throw "ignore";

var original = new Temporal.ZonedDateTime(946684800000000010n, "+01:30");
for (var property of ["epochNanoseconds", "timeZoneId", "calendarId"]) {
  Object.defineProperty(original, property, {
    get: function () { throw "must not read " + property; }
  });
}
var copy = Temporal.ZonedDateTime.from(original);
if (copy === original || copy.epochNanoseconds !== 946684800000000010n) throw "copy";
if (copy.timeZoneId !== "+01:30" || copy.calendarId !== "iso8601") throw "copy slots";

var optionReads = [];
var options = new Proxy({}, {
  get: function (_target, property) {
    optionReads.push(property);
    return undefined;
  }
});
Temporal.ZonedDateTime.from("1970-01-01T00:00Z[UTC]", options);
if (optionReads.join(",") !== "disambiguation,offset,overflow") throw "option order";
optionReads.length = 0;
try {
  Temporal.ZonedDateTime.from("2020-13-34T25:60:60+99:99[UTC]", options);
} catch (error) {
  if (!(error instanceof RangeError)) throw error;
}
if (optionReads.length !== 0) throw "invalid before options";

var errors = 0;
for (var invalid of [
  "1970-01-01T00:00",
  "1970-01-01T00:00[NoSuch/Zone]",
  "1970-01-01T00:00[+01:]",
  "1970-01-01T00:00[+24:00]",
  "-000000-01-01T00:00Z[UTC]"
]) {
  try { Temporal.ZonedDateTime.from(invalid); } catch (error) {
    if (error instanceof RangeError) errors += 1;
  }
}
try { Temporal.ZonedDateTime.from({ year: 1970, timeZone: "UTC" }); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { Temporal.ZonedDateTime.from("1970-01-01T00:00Z[UTC]", null); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.ZonedDateTime.from("1970-01-01T00:00Z[UTC]"); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
if (errors !== 8) throw "errors";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should parse fixed-offset Temporal.ZonedDateTime strings");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_constructs_fixed_zone_temporal_zoned_date_times_from_property_bags() {
    let outcome = engine()
            .run_script(
                r#"
function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

var plain = Temporal.ZonedDateTime.from({
  year: 1976,
  month: 11,
  monthCode: "M11",
  day: 18,
  millisecond: 123,
  timeZone: "uTc"
});
same(plain.epochNanoseconds, 217123200123000000n, "plain epoch");
same(plain.timeZoneId, "UTC", "UTC canonicalization");

var monthCodeOnly = Temporal.ZonedDateTime.from({
  year: 1976,
  monthCode: "M11",
  day: 18,
  timeZone: "+01:00"
});
same(monthCodeOnly.epochNanoseconds, 217119600000000000n, "monthCode");

function functionBag() {}
functionBag.year = 1970;
functionBag.month = 1;
functionBag.day = 1;
functionBag.timeZone = "+01:30";
same(
  Temporal.ZonedDateTime.from(functionBag, function () {}).epochNanoseconds,
  -5400000000000n,
  "function bag and options"
);

var arrayBag = [];
arrayBag.year = 1970;
arrayBag.month = 1;
arrayBag.day = 1;
arrayBag.timeZone = "UTC";
same(Temporal.ZonedDateTime.from(arrayBag, []).epochNanoseconds, 0n, "array bag");

var constrained = Temporal.ZonedDateTime.from(
  { year: 2019, month: 13, day: 32, hour: 25, timeZone: "UTC" },
  { overflow: "constrain" }
);
same(constrained.epochNanoseconds, 1577833200000000000n, "constrain");

var mismatch = {
  year: 1970,
  month: 1,
  day: 1,
  offset: "-07:00",
  timeZone: "+01:00"
};
same(
  Temporal.ZonedDateTime.from(mismatch, { offset: "use" }).epochNanoseconds,
  25200000000000n,
  "offset use"
);
same(
  Temporal.ZonedDateTime.from(mismatch, { offset: "prefer" }).epochNanoseconds,
  -3600000000000n,
  "offset prefer"
);
same(
  Temporal.ZonedDateTime.from(mismatch, { offset: "ignore" }).epochNanoseconds,
  -3600000000000n,
  "offset ignore"
);

var reads = [];
var observed = new Proxy({
  year: 2001,
  month: 5,
  monthCode: "M05",
  day: 2,
  hour: 6,
  minute: 54,
  second: 32,
  millisecond: 987,
  microsecond: 654,
  nanosecond: 321,
  offset: "+00:00",
  calendar: "iso8601",
  timeZone: "UTC"
}, {
  get: function (target, property) {
    reads.push(property);
    return target[property];
  }
});
Temporal.ZonedDateTime.from(observed, {});
same(
  reads.join(","),
  "calendar,day,hour,microsecond,millisecond,minute,month,monthCode,nanosecond,offset,second,timeZone,year",
  "field order"
);

var errors = 0;
for (var call of [
  function () { Temporal.ZonedDateTime.from({ year: 1970, month: 1, timeZone: "UTC" }); },
  function () { Temporal.ZonedDateTime.from({ year: 1970, day: 1, timeZone: "UTC" }); },
  function () { Temporal.ZonedDateTime.from({ month: 1, day: 1, timeZone: "UTC" }); },
  function () { Temporal.ZonedDateTime.from({ year: 1970, month: 1, day: 1 }); }
]) {
  try { call(); } catch (error) {
    if (error instanceof TypeError) errors += 1;
  }
}
try {
  Temporal.ZonedDateTime.from(
    { year: 2019, month: 1, day: 32, timeZone: "UTC" },
    { overflow: "reject" }
  );
} catch (error) {
  if (error instanceof RangeError) errors += 1;
}
try { Temporal.ZonedDateTime.from(mismatch); } catch (error) {
  if (error instanceof RangeError) errors += 1;
}
try {
  Temporal.ZonedDateTime.from({
    year: 2000,
    month: 1,
    day: 1,
    hour: Infinity,
    timeZone: "UTC"
  });
} catch (error) {
  if (error instanceof RangeError) errors += 1;
}
same(errors, 7, "errors");
271;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should construct fixed-zone Temporal property bags");
    assert!(outcome.note.contains("number(271"));
}

#[test]
fn wasm_backend_exposes_complete_temporal_namespace_from_a_bare_reference() {
    let outcome = engine()
            .run_script(
                r#"
var namespace = Temporal;
for (var constructorName of [
  "Instant",
  "PlainDate",
  "ZonedDateTime",
  "PlainTime",
  "PlainDateTime",
  "PlainYearMonth",
  "PlainMonthDay",
  "Duration"
]) {
  var constructorDescriptor = Object.getOwnPropertyDescriptor(namespace, constructorName);
  if (typeof constructorDescriptor.value !== "function") throw constructorName;
  if (!constructorDescriptor.writable || constructorDescriptor.enumerable) throw constructorName + " flags";
  if (!constructorDescriptor.configurable) throw constructorName + " configurable";
}

var namespaceDescriptor = Object.getOwnPropertyDescriptor(namespace, "Now");
if (!namespaceDescriptor.writable) throw "Now writable";
if (namespaceDescriptor.enumerable) throw "Now enumerable";
if (!namespaceDescriptor.configurable) throw "Now configurable";
var now = namespaceDescriptor.value;

if (typeof now !== "object") throw "typeof";
if (Object.getPrototypeOf(now) !== Object.prototype) throw "prototype";
if (now.prototype !== undefined) throw "prototype property";
if (!Object.isExtensible(now)) throw "extensible";
if (Object.prototype.toString.call(now) !== "[object Temporal.Now]") {
  throw "toStringTag";
}

var tagDescriptor = Object.getOwnPropertyDescriptor(
  now,
  Symbol.toStringTag
);
if (tagDescriptor.value !== "Temporal.Now") throw "tag value";
if (tagDescriptor.writable || tagDescriptor.enumerable) throw "tag flags";
if (!tagDescriptor.configurable) throw "tag configurable";

for (var name of ["timeZoneId", "instant", "zonedDateTimeISO"]) {
  var descriptor = Object.getOwnPropertyDescriptor(now, name);
  if (typeof descriptor.value !== "function") throw "member " + name;
  if (!descriptor.writable || descriptor.enumerable) throw "flags " + name;
  if (!descriptor.configurable) throw "configurable " + name;
  if (descriptor.value.length !== 0) throw "length " + name;
  if (descriptor.value.name !== name) throw "name " + name;
}

var constructErrors = 0;
for (var member of [
  now.timeZoneId,
  now.instant,
  now.zonedDateTimeISO,
]) {
  try { new member(); } catch (error) {
    if (error instanceof TypeError) constructErrors += 1;
  }
}
if (constructErrors !== 3) throw "constructability";
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should expose the complete Temporal namespace");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_reads_the_wall_clock_through_temporal_now() {
    let outcome = engine()
        .run_script(
            r#"
if (Temporal.Now.timeZoneId() !== "UTC") throw "timeZoneId";

var before = Date.now();
var instant = Temporal.Now.instant();
var after = Date.now();
if (!(instant instanceof Temporal.Instant)) throw "instant brand";
if (Object.getPrototypeOf(instant) !== Temporal.Instant.prototype) throw "instant proto";
var millis = Number(instant.epochNanoseconds / 1000000n);
if (millis < before || millis > after) throw "instant clock";
if (instant.epochMilliseconds !== millis) throw "epochMilliseconds";
if (Temporal.Now.instant() === Temporal.Now.instant()) throw "distinct";

var zoned = Temporal.Now.zonedDateTimeISO();
if (!(zoned instanceof Temporal.ZonedDateTime)) throw "zoned brand";
if (zoned.calendarId !== "iso8601") throw "calendar";
if (zoned.timeZoneId !== Temporal.Now.timeZoneId()) throw "zone";
if (Temporal.Now.zonedDateTimeISO(undefined).timeZoneId !== "UTC") throw "undefined zone";
if (Temporal.Now.zonedDateTimeISO("UtC").timeZoneId !== "UTC") throw "case folding";
if (Temporal.Now.zonedDateTimeISO("+01:30").timeZoneId !== "+01:30") throw "offset zone";
if (Temporal.Now.zonedDateTimeISO("2021-08-19T17:30Z[UTC]").timeZoneId !== "UTC") {
  throw "annotated zone";
}

var zoneErrors = 0;
for (var zone of [null, true, 1, 1n, {}]) {
  try { Temporal.Now.zonedDateTimeISO(zone); } catch (error) {
    if (error instanceof TypeError) zoneErrors += 1;
  }
}
if (zoneErrors !== 5) throw "zone type errors";

var rangeErrors = 0;
for (var zone of ["", "2021-08-19T17:30", "NoSuch/Zone", "-12:12:59.9"]) {
  try { Temporal.Now.zonedDateTimeISO(zone); } catch (error) {
    if (error instanceof RangeError) rangeErrors += 1;
  }
}
if (rangeErrors !== 4) throw "zone range errors";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should answer Temporal.Now clock reads");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_exposes_fixed_temporal_zoned_date_time_offsets() {
    let outcome = engine()
        .run_script(
            r#"
var cases = [
  ["UTC", "+00:00", 0],
  ["+01:00", "+01:00", 3600000000000],
  ["-05:00", "-05:00", -18000000000000],
  ["+0130", "+01:30", 5400000000000]
];
for (var entry of cases) {
  var datetime = new Temporal.ZonedDateTime(0n, entry[0]);
  if (datetime.offset !== entry[1]) throw "offset " + entry[0];
  if (datetime.offsetNanoseconds !== entry[2]) {
    throw "offsetNanoseconds " + entry[0];
  }
}

var offsetDescriptor = Object.getOwnPropertyDescriptor(
  Temporal.ZonedDateTime.prototype,
  "offset"
);
var nanosecondsDescriptor = Object.getOwnPropertyDescriptor(
  Temporal.ZonedDateTime.prototype,
  "offsetNanoseconds"
);
for (var descriptor of [offsetDescriptor, nanosecondsDescriptor]) {
  if (typeof descriptor.get !== "function" || descriptor.set !== undefined) {
    throw "accessor";
  }
  if (descriptor.enumerable || !descriptor.configurable) throw "descriptor";
  if (descriptor.get.length !== 0) throw "length";
}
if (offsetDescriptor.get.name !== "get offset") throw "offset name";
if (nanosecondsDescriptor.get.name !== "get offsetNanoseconds") {
  throw "offsetNanoseconds name";
}

var brandErrors = 0;
for (var getter of [offsetDescriptor.get, nanosecondsDescriptor.get]) {
  for (var receiver of [
    undefined,
    null,
    true,
    "",
    Symbol(),
    1,
    {},
    Temporal.ZonedDateTime,
    Temporal.ZonedDateTime.prototype
  ]) {
    try { getter.call(receiver); } catch (error) {
      if (error instanceof TypeError) brandErrors += 1;
    }
  }
}
if (brandErrors !== 18) throw "branding";

var constructErrors = 0;
try { new offsetDescriptor.get(); } catch (error) {
  if (error instanceof TypeError) constructErrors += 1;
}
try { new nanosecondsDescriptor.get(); } catch (error) {
  if (error instanceof TypeError) constructErrors += 1;
}
if (constructErrors !== 2) throw "constructability";

var namedZoneErrors = 0;
try { new Temporal.ZonedDateTime(0n, "NoSuch/Zone"); } catch (error) {
  if (error instanceof RangeError) namedZoneErrors += 1;
}
if (namedZoneErrors !== 1) throw "unknown named zone";
262;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should expose fixed Temporal.ZonedDateTime offsets");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_projects_temporal_zoned_date_time_civil_fields_and_compares_slots() {
    let outcome = engine()
            .run_script(
                r#"
function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

var negative = new Temporal.ZonedDateTime(-1n, "UTC");
for (var entry of [
  ["year", 1969],
  ["month", 12],
  ["monthCode", "M12"],
  ["day", 31],
  ["hour", 23],
  ["minute", 59],
  ["second", 59],
  ["millisecond", 999],
  ["microsecond", 999],
  ["nanosecond", 999]
]) {
  same(negative[entry[0]], entry[1], "negative " + entry[0]);
}

var offset = new Temporal.ZonedDateTime(0n, "+01:30");
same(offset.year, 1970, "offset year");
same(offset.month, 1, "offset month");
same(offset.monthCode, "M01", "offset monthCode");
same(offset.day, 1, "offset day");
same(offset.hour, 1, "offset hour");
same(offset.minute, 30, "offset minute");
same(offset.second, 0, "offset second");

var reference = new Temporal.ZonedDateTime(988786472987654321n, "UTC");
same(reference.equals(new Temporal.ZonedDateTime(988786472987654321n, "UTC")), true, "equal slots");
same(reference.equals(new Temporal.ZonedDateTime(988786472987654322n, "UTC")), false, "epoch");
same(reference.equals(new Temporal.ZonedDateTime(988786472987654321n, "+00:00")), false, "zone");
same(reference.equals(new Temporal.ZonedDateTime(988786472987654321n, "UTC", "iso8601")), true, "calendar");
same(reference.equals({
  year: 2001,
  month: 5,
  day: 2,
  hour: 6,
  minute: 54,
  second: 32,
  millisecond: 987,
  microsecond: 654,
  nanosecond: 321,
  timeZone: "UTC"
}), true, "property bag");

var fields = ["year", "month", "monthCode", "day", "hour", "minute", "second",
  "millisecond", "microsecond", "nanosecond"];
var brandErrors = 0;
for (var field of fields) {
  var descriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, field);
  if (descriptor.get.name !== "get " + field || descriptor.get.length !== 0) throw "metadata " + field;
  if (descriptor.enumerable || !descriptor.configurable || descriptor.set !== undefined) {
    throw "descriptor " + field;
  }
  try { descriptor.get.call({}); } catch (error) {
    if (error instanceof TypeError) brandErrors += 1;
  }
}
same(brandErrors, fields.length, "brands");
same(Temporal.ZonedDateTime.prototype.equals.name, "equals", "equals name");
same(Temporal.ZonedDateTime.prototype.equals.length, 1, "equals length");
var equalsDescriptor = Object.getOwnPropertyDescriptor(Temporal.ZonedDateTime.prototype, "equals");
if (!equalsDescriptor.writable || equalsDescriptor.enumerable || !equalsDescriptor.configurable) {
  throw "equals descriptor";
}
var errors = 0;
try { Temporal.ZonedDateTime.prototype.equals.call({}, reference); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { reference.equals(); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
try { new Temporal.ZonedDateTime.prototype.equals(reference); } catch (error) {
  if (error instanceof TypeError) errors += 1;
}
same(errors, 3, "equals errors");
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should project and compare Temporal.ZonedDateTime slots");
    assert!(outcome.note.contains("number(262"));
}

#[test]
fn wasm_backend_create_realm_exposes_array_mutator_and_locale_methods() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let array = new other.Array(2);
array[0] = 1;
array[1] = 2;
let pushedLength = other.Array.prototype.push.call(array, 3);
let popped = other.Array.prototype.pop.call(array);
[
  typeof other.Array.prototype.toLocaleString,
  typeof other.Array.prototype.push,
  typeof other.Array.prototype.pop,
  other.Array.prototype.toLocaleString === Array.prototype.toLocaleString,
  other.Array.prototype.push === Array.prototype.push,
  other.Array.prototype.pop === Array.prototype.pop,
  other.Array.prototype.toLocaleString.call(array),
  pushedLength,
  popped,
  array.length,
  array[0],
  array[1],
  Object.getPrototypeOf(array) === other.Array.prototype
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should expose Array prototype mutator and locale methods");
    assert!(
        outcome
            .note
            .contains("string(function|function|function|false|false|false|1,2|3|3|2|1|2|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_mutator_and_locale_methods_throw_other_realm_type_error() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm();
let names = ["toLocaleString", "pop", "push"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let bound = method.bind(null, 1);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null, 1);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array mutator/locale methods should throw in defining realm",
            );
    assert!(
            outcome.note.contains(
                "string(toLocaleString=true:true:false|true:true:false,pop=true:true:false|true:true:false,push=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_string_locale_compare_handles_canonical_equivalence_and_coercion() {
    let source = r#"
let receiver = {
  toString() { return "o\u0308"; }
};
let that = {
  toString() { return "\u00F6"; }
};
[
  String.prototype.localeCompare.call(receiver, that),
  "a".localeCompare("b"),
  "b".localeCompare("a"),
  "undefined".localeCompare(),
  String.prototype.localeCompare.length,
  String.prototype.localeCompare.name
].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("String.prototype.localeCompare should normalize and compare strings");
    assert!(
        outcome.note.contains("string(0|-1|1|0|1|localeCompare)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_locale_string_preserves_primitive_receivers() {
    let source = r#"
"use strict";
Boolean.prototype.toLocaleString = function() {
  return typeof this;
};
let direct = [true, false].toLocaleString();
Object.defineProperty(Boolean.prototype, "toLocaleString", {
  get: function() {
    let receiverType = typeof this;
    return function() {
      return receiverType;
    };
  }
});
direct + "|" + [true, false].toLocaleString();
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toLocaleString should preserve primitive receivers");
    assert!(
        outcome
            .note
            .contains("string(boolean,boolean|boolean,boolean)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_locale_string_snapshots_resizable_typed_array_length() {
    let source = r#"
let original = Number.prototype.toLocaleString;
let shrinkingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let shrinking = new Uint8Array(shrinkingBuffer, 0, 4);
let shrinkCalls = 0;
Number.prototype.toLocaleString = function() {
  shrinkCalls += 1;
  if (shrinkCalls === 2) shrinkingBuffer.resize(2);
  return original.call(this);
};
let shrunk = Array.prototype.toLocaleString.call(shrinking);

let growingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let growing = new Uint8Array(growingBuffer);
let growCalls = 0;
Number.prototype.toLocaleString = function() {
  growCalls += 1;
  if (growCalls === 2) growingBuffer.resize(6);
  return original.call(this);
};
let grown = Array.prototype.toLocaleString.call(growing);
Number.prototype.toLocaleString = original;
shrunk + "|" + grown + "|" + shrinkCalls + "|" + growCalls;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toLocaleString should snapshot typed array length");
    assert!(
        outcome.note.contains("string(0,0,,|0,0,0,0|2|4)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_to_locale_string_uses_internal_receiver_state() {
    let source = "const method = Uint8Array.prototype.toLocaleString; let forged = 'missing'; try { forged = method.call({ $TypedArrayViewedArrayBuffer: {}, length: 1, 0: 1 }); } catch (error) { forged = error.name; } const values = new Uint8Array([1, 2]); let lengthReads = 0; Object.defineProperty(values, 'length', { get() { lengthReads++; return 0; } }); const typed = method.call(values); const generic = Array.prototype.toLocaleString.call({ length: 2, 0: 1, 1: 2 }); forged + '|' + typed + '|' + lengthReads + '|' + generic;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.toLocaleString should use internal receiver state");
    assert!(
        outcome.note.contains("string(TypeError|1,2|0|1,2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_to_locale_string_calls_proxies_with_a_length_snapshot() {
    let source = "const original = Number.prototype.toLocaleString; const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const values = new Uint8Array(buffer); values[0] = 1; values[1] = 2; values[2] = 3; values[3] = 4; const receivers = []; function format() { 'use strict'; receivers.push(typeof this); if (receivers.length === 1) buffer.resize(2); return Number(this) * 10; } Number.prototype.toLocaleString = new Proxy(format, {}); const result = values.toLocaleString(); Number.prototype.toLocaleString = original; result + '|' + receivers.length + '|' + receivers.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.toLocaleString should call callable proxies");
    assert!(
        outcome.note.contains("string(10,20,,|2|number,number)"),
        "note: {}",
        outcome.note
    );
}
