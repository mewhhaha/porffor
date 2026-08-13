//! `date` CLI integration tests.

use crate::*;

#[test]
fn run_wasm_backend_succeeds_for_date_annexb_legacy_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_annexb_legacy_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_core_time_values_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_core_time_values.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_parse_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_parse.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_object_to_string_brand_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_object_to_string_brand.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_component_getters_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_component_getters.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_component_setters_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_component_setters.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_to_utc_string_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_to_utc_string.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_locale_strings_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_locale_strings.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

/// Pins the observable ECMA-402 `CanonicalizeLocaleList` array-like walk:
/// length is captured once, and every index performs HasProperty before its
/// conditional Get, coercion, provider canonicalization and deduplication.
#[test]
fn run_wasm_backend_succeeds_for_intl_canonical_locale_list_observation_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_intl_canonical_locale_list_observation.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(stdout.contains("number(262"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_date_to_iso_string_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_to_iso_string.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_to_temporal_instant_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_to_temporal_instant.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_temporal_instant_statics_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_temporal_instant_statics.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    // The expected rendering is spelled out here rather than recomputed in the
    // fixture, so a wrong ISO string cannot match itself.
    //
    // Asserted in two halves on purpose. The seven comparison results are
    // `Temporal.Instant.compare` itself; the eight rendered strings also depend
    // on machinery it does not own — the leap-second clamp (the seventh
    // comparison reads `2016-12-31T23:59:60Z`), the hour-only UTC offset the
    // ISO parser must accept, the two-limb heap-BigInt millisecond quotient,
    // and the six-digit-year branch at both epoch extremes. One combined
    // `contains` would report a break in any of those as a failure of
    // `compare`.
    assert!(
        stdout.contains("temporal-instant-statics:lt|gt|eq|gt|eq|gt|eq|"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "1970-01-01T00:00:00Z|\
             1976-11-18T14:23:30.123Z|\
             1963-02-13T09:36:29.124Z|\
             1976-11-18T14:23:30.123456789Z|\
             1969-07-24T16:50:35.000000001Z|\
             1970-01-01T00:00:30.123456Z|\
             +275760-09-13T00:00:00Z|\
             -271821-04-20T00:00:00Z"
        ),
        "{stdout}"
    );
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_resolves_temporal_time_string_calendars_by_consumer() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_temporal_time_calendar_resolution.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_temporal_zoned_date_time_era_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_temporal_zoned_date_time_era.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));

    // Asserted in three halves, because the three carry different mechanisms
    // and one combined `contains` would report any of them as a failure of the
    // first.
    //
    // (1) The accessor pair itself: `"ce"`/1970 for a gregory receiver and
    // `undefined` for an iso8601 one. The `undefined` half is the one that used
    // to "pass" in Test262 because the property did not exist at all.
    assert!(
        stdout.contains("temporal-zdt-era:ce|1970|undefined|undefined|"),
        "{stdout}"
    );
    // (2) The year-0 boundary from both sides, through `Temporal.ZonedDateTime.from`.
    // `bce` counts backwards from ISO year 1, so ISO 0 is bce 1 and ISO -1 is
    // bce 2; a getter that read the local ISO year as an unconverted f64 bit
    // pattern would answer `ce` for all three of these.
    assert!(stdout.contains("1/ce/1|0/bce/1|-1/bce/2|"), "{stdout}");
    // (3) `toPlainDateTime`, rendered component by component rather than through
    // `toString`, so a wrong component cannot hide inside a formatted string.
    // The four rows are: the `bce` property-bag receiver, a positive
    // sub-millisecond instant, a pre-epoch instant (the negative-remainder
    // correction), and the same instant as row 1 of the boundary pair read
    // through a -12:00 offset, which moves it across the era boundary.
    assert!(
        stdout.contains(
            "-1-6-15T12:34:0.0.0.0/bce/2|\
             1976-11-18T14:23:30.123.456.789/ce/1976|\
             1969-7-24T16:50:35.0.0.1/ce/1969|\
             0-12-31T12:0:0.0.0.0/bce/1"
        ),
        "{stdout}"
    );
    assert!(stdout.contains("number(262"));
}

/// `Temporal.ZonedDateTime.prototype.{add,subtract,until,since,withCalendar}`,
/// the five members added in batch 6.
///
/// This is the affordable regression test for the 28
/// `intl402/Temporal/ZonedDateTime/prototype/{add,subtract,since,until}/era-boundary-*.js`
/// cases: a cold Test262 case in that family was measured at ~300 s in batch 5,
/// so 28 of them is not an inner-loop gate on any machine this project runs on.
/// Every value the fixture asserts is copied from one of those files.
#[test]
fn run_wasm_backend_succeeds_for_temporal_zoned_date_time_arithmetic_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_temporal_zoned_date_time_arithmetic.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));

    // Asserted in three halves, because the three carry different mechanisms
    // and one combined `contains` would report any of them as a failure of the
    // first.
    //
    // (1) `add`/`subtract` across the year-0 boundary, rendered component by
    // component. The gregory era numbering has no year zero, so "1 BCE + 1
    // year" is 1 CE and "1 CE - 1 year" is 1 BCE; the third and first rows
    // being the same value is `add(-1)` and `subtract(+1)` agreeing.
    assert!(
        stdout.contains(
            "temporal-zdt-arith:\
             0-6-15T12:34/bce/1|\
             1-6-15T12:34/ce/1|\
             0-6-15T12:34/bce/1|\
             -5-3-1T12:34/bce/6|"
        ),
        "{stdout}"
    );
    // (2) `until`/`since`. `9y`, not `10y`, from 5 BCE to 5 CE — there is no
    // year zero to count, which is the single sharpest value in the fixture.
    // The last row is `since` answering the negation of the matching `until`.
    assert!(stdout.contains("9y0mo|0y108mo|2y1mo|1y0mo|"), "{stdout}");
    // (3) `withCalendar`: same instant, re-labelled. The ISO calendar has no
    // eras, so `era` goes `undefined` while the ISO year stays -1 — a
    // `withCalendar` that rebuilt the value from era fields instead of from the
    // record's epoch nanoseconds would move the year.
    assert!(stdout.contains("iso8601/-1/undefined"), "{stdout}");
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_to_json_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_to_json.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_date_symbol_to_primitive_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_symbol_to_primitive.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_copies_date_values_without_observing_coercion_hooks() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_constructor_date_value.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

/// `class S extends Intl.DateTimeFormat {}` — a subclass with no explicit
/// constructor — must produce an ordinary object, and its own methods must be
/// called.
///
/// This lives in `date::` rather than a new `intl::` module because
/// `known_failures::rung_1c_chunks_cover_every_cli_area_module` requires a
/// bijection between `tests/cli/*.rs`, `main.rs`'s `mod` lines and
/// `scripts/rung1c-chunks.sh`'s `run_chunk` lines, and those two files belong to
/// a different lane. `date::` already roots the `Intl` namespace through
/// `wasm_date_locale_strings.js`.
///
/// # It starts from a measured red, not a deduced one
///
/// Before the `lila-ir` change this test exists for,
/// `lila run --execution-backend wasm` on the fixture exited 1 with
/// `uncaught throw: wasm-aot completion: string(dtf subclass reduceRight was not
/// called)`. The fixture's own header records the mechanism and, importantly,
/// why the obvious assertions (`format`, `resolvedOptions`, `instanceof`) do
/// **not** exercise it — a first draft asserting only those passed unpatched.
#[test]
fn run_wasm_backend_succeeds_for_intl_date_time_format_subclass_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_intl_date_time_format_subclass.js"))
        .output()
        .expect("run command should run");

    // Both streams: the fixture reports a defect by throwing a named string,
    // and which stream carries `uncaught throw: …` is not worth guessing when
    // the whole point of the message is to say which assertion failed.
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_date_utc_and_exponent_tonumber_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_date_utc_and_exponent_tonumber.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}
