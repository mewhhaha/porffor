use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_compound_addition(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("compound addition failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn compound_addition_preserves_runtime_number_string_and_bigint_results() {
    assert_compound_addition(
        r#"
function add(value, right) { value += right; return value; }
add(2, 3) === 5 && add('2', 3) === '23' && add(2, '3') === '23' &&
add(2n, 3n) === 5n && add(undefined, 1) !== add(undefined, 1) &&
add(null, 1) === 1 && add(true, 1) === 2;
"#,
    );
}

#[test]
fn nested_cursor_scanner_keeps_numeric_index_after_a_skipped_compound_assignment() {
    assert_compound_addition(
        r#"
function scan(source, start) {
  let pos = start;
  const isNewline = c => /[\n\r\u2028\u2029]/.test(c);
  const isWhitespace = c => /\s/.test(c);
  const eatWhitespace = () => {
    while (pos < source.length) {
      const c = source[pos];
      if (isWhitespace(c) || isNewline(c)) { pos += 1; continue; }
      if (c === '/') {
        if (source[pos + 1] === '/') {
          while (pos < source.length) {
            if (isNewline(source[pos])) break;
            pos += 1;
          }
          continue;
        }
        if (source[pos + 1] === '*') {
          const end = source.indexOf('*/', pos);
          if (end === -1) throw new SyntaxError();
          pos = end + 2;
          continue;
        }
      }
      break;
    }
  };
  eatWhitespace();
  return pos;
}
scan('function a(){ /* } */ [native code] }', 13) === 22 &&
scan('/*x*/[', 0) === 5 && scan('//x\n[', 0) === 4;
"#,
    );
}

#[test]
fn compound_addition_converts_objects_with_default_hint_after_evaluating_both_operands() {
    assert_compound_addition(
        r#"
var trace = [];
var left = {[Symbol.toPrimitive](hint) { trace.push('left:' + hint); return 2; }};
var right = {[Symbol.toPrimitive](hint) { trace.push('right:' + hint); return 3; }};
function add(value, rhs) { value += rhs; return value; }
var result = add(left, right);
if (result !== 5 || trace.join('|') !== 'left:default|right:default') throw 'numeric conversion';
trace = [];
result = add('x', right);
if (result !== 'x3' || trace.join('|') !== 'right:default') throw 'string conversion';
var marker = Symbol('abrupt'), caught;
try { add(1, {[Symbol.toPrimitive]() { throw marker; }}); } catch (error) { caught = error; }
caught === marker;
"#,
    );
}

#[test]
fn statement_branches_merge_lexical_values_for_skipped_and_else_paths() {
    assert_compound_addition(
        r#"
function choose(flag) { let value = 1; if (flag) value = 'x'; return value + 1; }
function alternate(flag) { let value = 1; if (flag) value = 'x'; else value += 1; return value + 1; }
function captured(flag) {
  let value = 1;
  const read = () => { if (flag) value = 'x'; return value + 1; };
  return read();
}
function capturedAlternate(flag) {
  let value = 1;
  const read = () => { if (flag) value = 'x'; else value += 1; return value + 1; };
  return read();
}
choose(false) === 2 && choose(true) === 'x1' &&
alternate(false) === 3 && alternate(true) === 'x1' &&
captured(false) === 2 && captured(true) === 'x1' &&
capturedAlternate(false) === 3 && capturedAlternate(true) === 'x1';
"#,
    );
}

#[test]
fn right_side_binding_mutation_does_not_replace_the_saved_left_value() {
    assert_compound_addition(
        r#"
function mutate() { let value = 1; value += (value = 'changed', 2); return value; }
function mutateString() { let value = 'x'; value += (value = 10, 2); return value; }
mutate() === 3 && mutateString() === 'x2';
"#,
    );
}
