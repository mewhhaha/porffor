use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Date parsing regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn reduced_iso_dates_accept_each_time_precision() {
    assert_wasm_true(
        r#"
var dates = ["1970", "1970-01", "1970-01-01", "+001970", "+001970-01"];
var times = ["T12:34Z", "T12:34:56Z", "T12:34:56.789Z"];
var values = [45240000, 45296000, 45296789];
var ok = true;
for (var d = 0; d < dates.length; d++) {
  for (var t = 0; t < times.length; t++) {
    if (Date.parse(dates[d] + times[t]) !== values[t]) ok = false;
  }
}
ok && Date.parse("2000-02T12:34Z") === 949408440000;
"#,
    );
}

#[test]
fn iso_midnight_rolls_over_after_calendar_validation() {
    assert_wasm_true(
        r#"
var cases = [
  ["1970T24:00Z", 86400000],
  ["1995-02-04T24:00Z", 791942400000],
  ["1999-12-31T24:00:00.000Z", 946684800000],
  ["2000-02-28T24:00:00Z", 951782400000],
  ["2000-02-29T24:00Z", 951868800000],
  ["1970-01-01T24:00+01:30", 81000000],
  ["1970-01-01T24:00-01:30", 91800000],
  ["-000001-12-31T24:00Z", -62167219200000]
];
var ok = true;
for (var i = 0; i < cases.length; i++) {
  if (Date.parse(cases[i][0]) !== cases[i][1]) ok = false;
}
ok;
"#,
    );
}

#[test]
fn invalid_midnight_and_calendar_fields_remain_nan() {
    assert_wasm_true(
        r#"
var strings = [
  "2000-01-01T24:01Z", "2000-01-01T24:00:01Z",
  "2000-01-01T24:00:00.001Z", "2000-01-01T25:00Z",
  "2001-02-29T24:00Z", "2000-02-30T24:00Z",
  "2000-00-01T24:00Z", "2000-13-01T24:00Z",
  "2000-01-00T24:00Z", "2000-01-32T24:00Z",
  "2000-01-01T23:60Z", "2000-01-01T23:59:60Z",
  "2000-01-01T00:00+24:00", "2000-01-01T00:00-00:60"
];
var ok = true;
for (var i = 0; i < strings.length; i++) {
  if (!Number.isNaN(Date.parse(strings[i]))) ok = false;
}
ok;
"#,
    );
}

#[test]
fn iso_offsets_are_applied_before_time_clip() {
    assert_wasm_true(
        r#"
var max = 8640000000000000;
var min = -8640000000000000;
Date.parse("+275760-09-12T24:00:00.000Z") === max &&
Date.parse("-271821-04-19T24:00:00.000Z") === min &&
Date.parse("+275760-09-13T01:00:00.000+01:00") === max &&
Date.parse("-271821-04-19T23:00:00.000-01:00") === min &&
Number.isNaN(Date.parse("+275760-09-13T00:00:00.001Z")) &&
Number.isNaN(Date.parse("-271821-04-19T23:59:59.999Z")) &&
Number.isNaN(Date.parse("+275760-09-13T24:00Z"));
"#,
    );
}

#[test]
fn canonical_display_formats_round_trip_beyond_the_epoch() {
    assert_wasm_true(
        r#"
var values = [
  -8640000000000000, -62198755200000, -62167219200000,
  -59011459200000, -2208988800000, -1000, 0, 1000,
  123456789000, 951827696000, 1582979696000,
  253402300800000, 8640000000000000
];
var ok = true;
for (var i = 0; i < values.length; i++) {
  var date = new Date(values[i]);
  if (Date.parse(date.toString()) !== values[i]) ok = false;
  if (Date.parse(date.toUTCString()) !== values[i]) ok = false;
  if (Date.parse(date.toISOString()) !== values[i]) ok = false;
}
ok;
"#,
    );
}

#[test]
fn display_parser_covers_all_months_and_weekdays() {
    assert_wasm_true(
        r#"
var ok = true;
for (var month = 0; month < 12; month++) {
  var value = Date.UTC(2000, month, 15, 12, 34, 56);
  var date = new Date(value);
  if (Date.parse(date.toString()) !== value) ok = false;
  if (Date.parse(date.toUTCString()) !== value) ok = false;
}
for (var day = 1; day <= 7; day++) {
  var value = Date.UTC(1999, 0, day, 1, 2, 3);
  var date = new Date(value);
  if (Date.parse(date.toString()) !== value) ok = false;
  if (Date.parse(date.toUTCString()) !== value) ok = false;
}
ok;
"#,
    );
}

#[test]
fn date_constructor_and_detached_parse_use_the_same_runtime_parser() {
    assert_wasm_true(
        r#"
var value = 951827696000;
var source = new Date(value);
var local = source.toString();
var utc = source.toUTCString();
var parse = Date.parse;
new Date(local).getTime() === value && new Date(utc).getTime() === value &&
parse(local) === value && parse(utc) === value &&
new Date("1970T24:00Z").getTime() === 86400000;
"#,
    );
}

#[test]
fn truncated_display_strings_are_not_completed_from_adjacent_bytes() {
    assert_wasm_true(
        r#"
var date = new Date(951827696000);
var strings = [date.toString(), date.toUTCString()];
var ok = true;
for (var s = 0; s < strings.length; s++) {
  for (var i = 0; i < strings[s].length; i++) {
    var prefix = strings[s].slice(0, i);
    if (!Number.isNaN(Date.parse(prefix))) ok = false;
  }
  if (Date.parse(strings[s]) !== 951827696000) ok = false;
}
ok;
"#,
    );
}

#[test]
fn malformed_iso_strings_and_unicode_do_not_pass_ascii_grammar() {
    assert_wasm_true(
        r#"
var strings = [
  "", "1", "19", "197", "+", "-", "+00197", "-000000",
  "-000000-01-01T00:00Z", "1970-", "1970-0", "1970-01-",
  "1970T", "1970T0", "1970T00:", "1970T00:0",
  "1970T00:00:", "1970T00:00:00.", "1970T00:00:00.00Z",
  "1970T00:00+", "1970T00:00+01:", "1970T00:00Zjunk",
  "1970T00:00Z\u0000", "1970\u201001\u201001T00:00Z",
  "\uff11\uff19\uff17\uff10-01-01T00:00Z", "1970-01-01Z"
];
var ok = true;
for (var i = 0; i < strings.length; i++) {
  if (!Number.isNaN(Date.parse(strings[i]))) ok = false;
  if (Date.parse("1970-01-01T00:00Z") !== 0) ok = false;
}
ok;
"#,
    );
}

#[test]
fn display_fields_are_validated_instead_of_matching_only_a_prefix() {
    assert_wasm_true(
        r#"
var strings = [
  "Invalid Date", "xxx, 01 Jan 1970 00:00:00 GMT",
  "Thu, 01 xxx 1970 00:00:00 GMT", "Thu, 00 Jan 1970 00:00:00 GMT",
  "Thu, 01 Jan 1970 24:00:00 GMT", "Thu, 01 Jan 1970 00:60:00 GMT",
  "Thu, 01 Jan 1970 00:00:60 GMT", "Thu, 01 Jan 1970 00:00:00 GMTjunk",
  "Thu, 01 Jan 1970000 00:00:00 GMT", "Thu, 01 Jan -0000 00:00:00 GMT"
];
var ok = true;
for (var i = 0; i < strings.length; i++) {
  if (!Number.isNaN(Date.parse(strings[i]))) ok = false;
}
ok && Date.parse("Thu, 01 Jan 1970 00:00:00 GMT") === 0 &&
Date.parse("Thu Jan 01 1970 00:00:00 GMT+0000 (Coordinated Universal Time)") === 0;
"#,
    );
}

#[test]
fn parse_coerces_once_and_propagates_abrupt_completion() {
    assert_wasm_true(
        r#"
var calls = 0;
var hints = "";
var object = {
  [Symbol.toPrimitive]: function(hint) {
    calls++;
    hints += hint;
    return "1970T24:00Z";
  }
};
var value = Date.parse(object);
var marker = {};
var caught = false;
try { Date.parse({ toString: function() { throw marker; } }); }
catch (error) { caught = error === marker; }
value === 86400000 && calls === 1 && hints === "string" && caught;
"#,
    );
}

#[test]
fn existing_iso_defaults_fractional_seconds_and_positive_zero_are_preserved() {
    assert_wasm_true(
        r#"
Date.parse("1970") === 0 && Date.parse("1970-01") === 0 &&
Date.parse("1970-01-01") === 0 && Date.parse("1970-01-01T00:00") === 0 &&
Date.parse("1970-01-01T00:00:00.123Z") === 123 &&
Date.parse("1970-01-01T00:00+01:00") === -3600000 &&
Date.parse("1970-01-01T00:00-01:00") === 3600000 &&
1 / Date.parse("1970-01-01T00:00:00.000Z") === Infinity;
"#,
    );
}
