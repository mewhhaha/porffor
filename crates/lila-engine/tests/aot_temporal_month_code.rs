use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_month_code_semantics(source: &str) {
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
        .expect("Temporal month-code validation must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn date_and_date_time_require_string_month_code_primitives() {
    assert_month_code_semantics(
        r#"
function check(convert) {
  for (const monthCode of [5, 5n, false, Symbol(), null, {toString() { return 5; }}]) {
    let received;
    try { convert({year: 2026, monthCode, day: 1}); }
    catch (error) { received = error; }
    if (!(received instanceof TypeError)) throw new Error('month-code primitive type');
  }
  const boxed = convert({year: 2026, monthCode: new String('M05'), day: 1});
  if (boxed.month !== 5) throw new Error('boxed month code');
  const absent = convert({year: 2026, month: 6, monthCode: undefined, day: 1});
  if (absent.month !== 6) throw new Error('absent month code');
}
check(fields => Temporal.PlainDate.from(fields));
check(fields => Temporal.PlainDateTime.from(fields));
check(fields => new Temporal.PlainDate(2025, 1, 1).with(fields));
check(fields => new Temporal.PlainDateTime(2025, 1, 1).with(fields));
true;
"#,
    );
}

#[test]
fn month_code_syntax_precedes_later_fields_and_suitability_follows_them() {
    assert_month_code_semantics(
        r#"
function check(convert) {
  let laterReads = 0;
  let syntaxError;
  try {
    convert({day: 1, monthCode: 'L99M', get year() { laterReads++; throw {}; }});
  } catch (error) { syntaxError = error; }
  if (!(syntaxError instanceof RangeError) || laterReads !== 0) {
    throw new Error('month-code syntax must precede year');
  }
  const marker = {};
  let received;
  try {
    convert({day: 1, monthCode: 'M99L', get year() { laterReads++; throw marker; }});
  } catch (error) { received = error; }
  if (received !== marker || laterReads !== 1) throw new Error('suitability follows year');
  let suitabilityError;
  try { convert({day: 1, monthCode: 'M99L', year: 2026}); }
  catch (error) { suitabilityError = error; }
  if (!(suitabilityError instanceof RangeError)) throw new Error('invalid ISO month code');
}
check(fields => Temporal.PlainDate.from(fields));
check(fields => Temporal.PlainDateTime.from(fields));
check(fields => new Temporal.PlainDate(2025, 1, 1).with(fields));
check(fields => new Temporal.PlainDateTime(2025, 1, 1).with(fields));
true;
"#,
    );
}

#[test]
fn month_code_coercion_uses_string_hint_once_and_preserves_thrown_values() {
    assert_month_code_semantics(
        r#"
function check(convert) {
  const trace = [];
  const result = convert({
    day: 1,
    monthCode: {
      [Symbol.toPrimitive](hint) { trace.push(hint); return 'M05'; },
      toString() { throw new Error('unexpected fallback'); }
    },
    get year() { trace.push('year'); return 2026; }
  });
  if (result.month !== 5 || trace.join(',') !== 'string,year') {
    throw new Error('month-code coercion order');
  }
  const marker = {};
  let received;
  try {
    convert({
      day: 1,
      monthCode: { toString() { throw marker; } },
      get year() { throw new Error('year read after abrupt coercion'); }
    });
  } catch (error) { received = error; }
  if (received !== marker) throw new Error('coercion throw identity');
}
check(fields => Temporal.PlainDate.from(fields));
check(fields => Temporal.PlainDateTime.from(fields));
check(fields => new Temporal.PlainDate(2025, 1, 1).with(fields));
check(fields => new Temporal.PlainDateTime(2025, 1, 1).with(fields));
true;
"#,
    );
}
