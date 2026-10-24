use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, ObservedNumber, RealmBuilder,
};

fn assert_result(source: &str, expected: ObservedJsValue) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let compilation = engine
        .profile_script_compilation(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
        )
        .unwrap_or_else(|error| {
            panic!("ordinary call sequence compilation failed: {error}\n{source}")
        });
    // This precondition applies to the whole rich program, including ordinary
    // construction, spread boundaries and result checks. Keep its largest
    // body well below the measured giant-body/JIT case (over one MiB), while
    // allowing fixed code unrelated to the repeated calls. The AOT emission
    // controls separately enforce the tighter source-body and growth bounds.
    // Inspect the same validated artifact executed below; compile only once.
    assert!(
        compilation.footprint.largest_code_body_bytes < 128 * 1024,
        "large ordinary source body: {:?}\n{source}",
        compilation.footprint
    );
    eprintln!(
        "numeric call control P: {} bytes, {} code bytes, largest {} bytes",
        compilation.footprint.module_bytes,
        compilation.footprint.code_body_bytes,
        compilation.footprint.largest_code_body_bytes
    );
    let execution = engine
        .profile_wasm_execution(&compilation.artifact, Some(30_000), true)
        .unwrap_or_else(|error| {
            panic!("ordinary call sequence execution failed: {error}\n{source}")
        });
    assert_eq!(execution.outcome.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        execution.outcome.completion,
        ObservedCompletion::Normal(expected),
        "{}\n{source}",
        execution.outcome.note
    );
}

fn calls(name: &str, first: usize, last: usize) -> String {
    (first..=last)
        .map(|value| format!("{name}({value});\n"))
        .collect()
}

#[test]
fn scalar_number_regions_keep_coercion_order_and_abrupt_boundaries() {
    assert_result(
        r#"
var log = [], sentinel = {}, caught;
function value(name, number) {
  return {valueOf() { log.push(name); return number; }};
}
function calculate(a, b, c) { return ((+a + 1) * (+b - 2)) / (+c + 3); }
if (calculate(value('a', 4), value('b', 8), value('c', 2)) !== 6 ||
    log.join('|') !== 'a|b|c') throw 'scalar coercion order';
log = [];
try {
  calculate(value('left', 4), {valueOf() { log.push('throw'); throw sentinel; }}, value('skipped', 2));
} catch (error) { caught = error; }
if (caught !== sentinel || log.join('|') !== 'left|throw') throw 'scalar abrupt';
log = [];
var left = {valueOf() { log.push('old conversion'); return 1; }};
function right() {
  log.push('evaluate right');
  left.valueOf = function() { log.push('convert left'); return 8; };
  return {valueOf() { log.push('convert right'); return 3; }};
}
var answer = (left - right()) * 2;
answer === 10 && log.join('|') === 'evaluate right|convert left|convert right';
"#,
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn scalar_number_regions_keep_ieee_edges_and_bigint_dispatch() {
    assert_result(
        r#"
function arithmetic(a, b, c) { return (+a + +b) * -(+c); }
function difference(a, b, c) { return (+a + +b) - +c; }
function remainder(a, b) { return -(+a) % +b; }
function compare(a, b) { return ((+a + 1) * 2) < ((+b - 1) * 3); }
Object.is(arithmetic(0, -0, 1), -0) &&
Object.is(arithmetic(0, -0, -1), 0) &&
Number.isNaN(arithmetic(Infinity, 0, 0)) &&
difference(9007199254740992, 1, 9007199254740992) === 0 &&
Object.is(remainder(5, 5), -0) &&
compare(2, 4) && !compare(NaN, 4) && !compare(2, NaN) &&
((2n + 3n) * -(4n - 1n)) === -15n;
"#,
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn fixed_argument_snapshots_keep_gc_roots_identity_aliasing_and_abrupt_order() {
    assert_result(
        r#"
var first = {value: 1}, old = first, sentinel = {}, called = 0, caught, log = [];
function capture(a, b, c, d, e) {
  called++;
  gc();
  return [a, b, c, d, e, arguments];
}
var held = capture(first, (first = {value: 2}, gc(), first), -0, 2n, Symbol('key'));
if (held[0] !== old || held[0].value !== 1 || held[1] !== first ||
    held[1].value !== 2 || !Object.is(held[2], -0) || held[3] !== 2n ||
    typeof held[4] !== 'symbol' || held[5][0] !== old) throw 'snapshot values';
function later() { log.push('throw'); gc(); throw sentinel; }
try { capture((log.push('first'), {}), later(), (log.push('skipped'), {})); }
catch (error) { caught = error; }
if (caught !== sentinel || called !== 1 || log.join('|') !== 'first|throw') throw 'snapshot abrupt';
function mapped(a, b) { a = 9; arguments[1] = 8; return arguments[0] === 9 && b === 8; }
function fresh() { return arguments; }
function rest(a, ...values) { gc(); return a === old && values[0] === first && values[1] === 'tail'; }
mapped(1, 2) && rest(old, first, 'tail') && fresh() !== fresh() &&
fresh(old)[0] === old && fresh(first)[0] === first;
"#,
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn repeated_numeric_calls_keep_live_callee_captured_locals_and_hoisted_declarations() {
    let rows = calls("step", 1, 1024);
    assert_result(
        &format!(
            r#"
function run(seed) {{
  let total = seed, count = 0;
  const read = () => total;
  {rows}
  function step(value) {{
    count++; total += value;
    if (count === 17) step = replacement;
  }}
  function replacement(value) {{ count++; total += value * 2; }}
  return read() === 1049454 && count === 1024 && step === replacement;
}}
run(7);
"#
        ),
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn repeated_numeric_calls_observe_each_global_getter_and_stop_on_getter_throw() {
    let rows = calls("consumeNumeric", 1, 1024);
    assert_result(
        &format!(
            r#"
var marker = {{}}, lookups = 0, count = 0, total = 0, caught, finalized = 0, atFinally;
function consume(value) {{
  if (this !== globalThis) throw new Error('bare call receiver');
  count++; total += value; return value;
}}
Object.defineProperty(globalThis, 'consumeNumeric', {{
  configurable: true,
  get() {{ if (++lookups === 37) throw marker; return consume; }}
}});
try {{ try {{ {rows} }} finally {{ finalized++; atFinally = total; }} }}
catch (error) {{ caught = error; }}
caught === marker && lookups === 37 && count === 36 && total === 666
  && atFinally === 666 && finalized === 1;
"#
        ),
        ObservedJsValue::Boolean(true),
    );
    // A finite non-string eval makes this ordinary activation environment
    // visible. The with Reference must be resolved and read anew for every
    // row, supply the held object's receiver, and stop on a getter throw.
    assert_result(
        &format!(
            r#"
var holder, marker = {{}}, lookups = 0, count = 0, total = 0, caught, finalized = 0;
function consume(value) {{
  if (this !== holder) throw new Error('with call receiver');
  count++; total += value; return value;
}}
holder = {{ get consumeNumeric() {{ if (++lookups === 37) throw marker; return consume; }} }};
function drive() {{
  eval(0);
  try {{ with (holder) {{ {rows} }} }} finally {{ finalized++; }}
}}
try {{ drive(); }} catch (error) {{ caught = error; }}
caught === marker && lookups === 37 && count === 36 && total === 666 && finalized === 1;
"#
        ),
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn repeated_numeric_call_throw_preserves_catch_finally_and_captured_environment() {
    let rows = calls("step", 1, 1024);
    assert_result(
        &format!(
            r#"
var marker = {{}};
function run() {{
  let total = 0, count = 0, caught, finalized = 0, atFinally;
  const read = () => total;
  function step(value) {{
    count++;
    if (value === 53) {{ gc(); throw marker; }}
    total += value; return value;
  }}
  try {{ try {{ {rows} }} finally {{ finalized++; atFinally = read(); }} }}
  catch (error) {{ caught = error; }}
  return caught === marker && count === 53 && total === 1378
    && read() === 1378 && atFinally === 1378 && finalized === 1;
}}
run();
"#
        ),
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn repeated_numeric_calls_keep_argument_bits_freshness_gc_roots_and_source_caller() {
    let rows = (0..1024)
        .map(|index| match index % 4 {
            0 => "save();\n",
            1 => "save(0);\n",
            2 => "save(1e400, 2);\n",
            _ => "save(0.5, 9007199254740992, 7);\n",
        })
        .collect::<String>();
    assert_result(
        &format!(
            r#"
var records = [], valid = true, source;
function save() {{
  var index = records.length;
  valid = valid && save.caller === drive && drive.toString() === source;
  if (index === 1024) valid = valid && arguments.length === 1 && Object.is(arguments[0], -0);
  else {{
    if (index % 4 === 0) valid = valid && arguments.length === 0;
    if (index % 4 === 1) valid = valid && arguments.length === 1 && Object.is(arguments[0], 0);
    if (index % 4 === 2) valid = valid && arguments.length === 2 && arguments[0] === 1e400 && arguments[1] === 2;
    if (index % 4 === 3) valid = valid && arguments.length === 3 && arguments[0] === 0.5
      && arguments[1] === 9007199254740992 && arguments[2] === 7;
  }}
  records.push(arguments);
  if (index % 16 === 0) gc();
}}
function drive() {{
  {rows}
  // Unary minus remains an ordinary argument evaluation after the large run.
  save(-0);
}}
source = Function.prototype.toString.call(drive);
drive();
gc();
records[1][0] = 99;
valid && records.length === 1025 && records[1] !== records[5] && Object.is(records[5][0], 0)
  && records[3][1] === 9007199254740992 && records[1023][2] === 7 && Object.is(records[1024][0], -0)
  && save.caller === null && drive.toString() === source && source.indexOf('save(0.5') >= 0;
"#
        ),
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn numeric_call_runs_keep_proxy_bound_dispatch_and_effectful_argument_boundaries() {
    let before = calls("visit", 1, 512);
    let after = calls("visit", 513, 1024);
    assert_result(
        &format!(
            r#"
var receiver = {{}}, trace = [], proxyCalls = 0, vectors = [];
function old(value) {{
  if (this !== receiver) throw new Error('bound this');
  trace.push('o' + value); return value;
}}
function replacement(value) {{ trace.push('r' + value); return value; }}
var visit = new Proxy(old.bind(receiver), {{
  apply(target, thisArg, argumentsList) {{
    proxyCalls++; vectors.push(argumentsList);
    return Reflect.apply(target, thisArg, argumentsList);
  }}
}});
{before}
visit((visit = replacement, 99));
{after}
visit(...[1026]);
proxyCalls === 513 && trace.length === 1026 && vectors[0] !== vectors[1]
  && vectors[0][0] === 1 && vectors[512][0] === 99
  && trace[0] === 'o1' && trace[511] === 'o512' && trace[512] === 'o99'
  && trace[513] === 'r513' && trace[1024] === 'r1024' && trace[1025] === 'r1026';
"#
        ),
        ObservedJsValue::Boolean(true),
    );
}

#[test]
fn numeric_call_runs_keep_entry_and_prepared_realm_script_last_completion() {
    let rows = calls("finish", 1, 1024);
    let script = format!("function finish(value) {{ return value; }}\n{rows}");
    assert_result(
        &script,
        ObservedJsValue::Number(ObservedNumber::from_f64(1024.0)),
    );
    assert_result(
        &format!(
            r#"
var realm = __scriptwCreateRealm();
realm.evalScript({script:?}) === 1024 && realm.evalScript('finish.caller;') === null;
"#
        ),
        ObservedJsValue::Boolean(true),
    );
}
