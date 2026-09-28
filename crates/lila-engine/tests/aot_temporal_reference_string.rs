use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .expect("Temporal reference string should execute through Wasm");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn month_day_reference_year_and_annotation_follow_separate_rules() {
    run_boolean(
        r#"
function check(value, expected) {
  var modes = ["auto", "always", "critical", "never"];
  for (var i = 0; i < modes.length; i++) {
    if (value.toString({ calendarName: modes[i] }) !== expected[i]) return false;
  }
  return value.toString() === expected[0] && value.toJSON() === expected[0];
}
var iso = new Temporal.PlainMonthDay(5, 2, "iso8601", 2001);
var gregory = new Temporal.PlainMonthDay(5, 2, "gregory", 2001);
var expanded = new Temporal.PlainMonthDay(5, 2, "iso8601", -10000);
check(iso, ["05-02", "2001-05-02[u-ca=iso8601]",
            "2001-05-02[!u-ca=iso8601]", "05-02"])
  && check(gregory, ["2001-05-02[u-ca=gregory]", "2001-05-02[u-ca=gregory]",
                     "2001-05-02[!u-ca=gregory]", "2001-05-02"])
  && expanded.toString({ calendarName: "always" }) === "-010000-05-02[u-ca=iso8601]";
"#,
    );
}

#[test]
fn year_month_reference_day_and_annotation_follow_separate_rules() {
    run_boolean(
        r#"
function check(value, expected) {
  var modes = ["auto", "always", "critical", "never"];
  for (var i = 0; i < modes.length; i++) {
    if (value.toString({ calendarName: modes[i] }) !== expected[i]) return false;
  }
  return value.toString() === expected[0] && value.toJSON() === expected[0];
}
var iso = new Temporal.PlainYearMonth(2000, 5, "iso8601", 17);
var gregory = new Temporal.PlainYearMonth(2000, 5, "gregory", 17);
var expanded = new Temporal.PlainYearMonth(10000, 5, "iso8601", 17);
check(iso, ["2000-05", "2000-05-17[u-ca=iso8601]",
            "2000-05-17[!u-ca=iso8601]", "2000-05"])
  && check(gregory, ["2000-05-17[u-ca=gregory]", "2000-05-17[u-ca=gregory]",
                     "2000-05-17[!u-ca=gregory]", "2000-05-17"])
  && expanded.toString({ calendarName: "always" }) === "+010000-05-17[u-ca=iso8601]";
"#,
    );
}

#[test]
fn serialization_reads_options_once_and_preserves_brand_and_error_order() {
    run_boolean(
        r#"
var monthDay = new Temporal.PlainMonthDay(5, 2, "gregory", 2001);
var yearMonth = new Temporal.PlainYearMonth(2000, 5, "gregory", 17);
var reads = 0;
var coercions = 0;
var option = {
  get calendarName() {
    reads++;
    return { toString: function () { coercions++; return "never"; } };
  }
};
var monthText = monthDay.toString(option);
var yearText = yearMonth.toString(option);
var poison = { get calendarName() { reads++; throw new Error("unexpected option read"); } };
var json = monthDay.toJSON(poison) === "2001-05-02[u-ca=gregory]"
  && yearMonth.toJSON(poison) === "2000-05-17[u-ca=gregory]";
var monthBrand;
var yearBrand;
try { Temporal.PlainMonthDay.prototype.toString.call({}, poison); }
catch (error) { monthBrand = error; }
try { Temporal.PlainYearMonth.prototype.toJSON.call({}); }
catch (error) { yearBrand = error; }
var monthOptionError;
var yearOptionError;
try { monthDay.toString({ calendarName: "invalid" }); }
catch (error) { monthOptionError = error; }
try { yearMonth.toString({ calendarName: "invalid" }); }
catch (error) { yearOptionError = error; }
monthText === "2001-05-02" && yearText === "2000-05-17"
  && reads === 2 && coercions === 2 && json
  && monthBrand instanceof TypeError && yearBrand instanceof TypeError
  && monthOptionError instanceof RangeError && yearOptionError instanceof RangeError;
"#,
    );
}
