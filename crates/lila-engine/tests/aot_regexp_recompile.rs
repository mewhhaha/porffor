use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
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
        .expect("RegExp recompilation must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn invalid_pattern_preserves_source_flags_matcher_and_last_index() {
    assert_wasm_true(
        r#"
var expression = /ab+/gi, throws = 0;
expression.lastIndex = 2;
try { expression.compile('.{2,1}'); } catch (error) { if (error instanceof SyntaxError) throws++; }
var first = expression.source === 'ab+' && expression.flags === 'gi' && expression.lastIndex === 2;
try { expression.compile('\\2', 'u'); } catch (error) { if (error instanceof SyntaxError) throws++; }
var second = expression.source === 'ab+' && expression.flags === 'gi' && expression.lastIndex === 2;
var match = expression.exec('xxABB');
throws === 2 && first && second && match[0] === 'ABB' && match.index === 2 && expression.lastIndex === 5;
"#,
    );
}

#[test]
fn valid_pattern_is_installed_before_last_index_write_throws() {
    assert_wasm_true(
        r#"
var fromString = /old/, fromRegExp = /old/, throws = 0;
Object.defineProperty(fromString, 'lastIndex', { value: 3, writable: false });
Object.defineProperty(fromRegExp, 'lastIndex', { value: 4, writable: false });
try { fromString.compile('new+', 'i'); } catch (error) { if (error instanceof TypeError) throws++; }
try { fromRegExp.compile(/next+/i); } catch (error) { if (error instanceof TypeError) throws++; }
throws === 2 && fromString.source === 'new+' && fromString.flags === 'i' &&
  fromString.lastIndex === 3 && fromString.test('NEWW') &&
  fromRegExp.source === 'next+' && fromRegExp.flags === 'i' &&
  fromRegExp.lastIndex === 4 && fromRegExp.test('NEXTT');
"#,
    );
}

#[test]
fn pattern_and_flags_are_coerced_before_replacement() {
    assert_wasm_true(
        r#"
var expression = /old/g, trace = '', marker = {}, caught = false;
expression.lastIndex = 2;
var pattern = { toString() { trace += 'pattern;'; return 'new'; } };
var flags = { toString() { trace += 'flags;'; throw marker; } };
try { expression.compile(pattern, flags); } catch (error) { caught = error === marker; }
caught && trace === 'pattern;flags;' && expression.source === 'old' &&
  expression.flags === 'g' && expression.lastIndex === 2 && expression.test('xxold');
"#,
    );
}

#[test]
fn last_index_coercion_reloads_the_program_capture_metadata_and_flags() {
    assert_wasm_true(
        r#"
var expression = /a/g, calls = 0;
expression.lastIndex = {
  valueOf() { calls++; expression.compile(/(?<letter>b)/dg); return 0; }
};
var match = expression.exec('b');
calls === 1 && match !== null && match[0] === 'b' && match[1] === 'b' &&
  match.groups.letter === 'b' && match.indices[1][0] === 0 &&
  match.indices[1][1] === 1 && expression.lastIndex === 1;
"#,
    );
}

#[test]
fn last_index_coercion_can_enable_or_remove_global_matching() {
    assert_wasm_true(
        r#"
var remove = /a/g, enable = /a/, calls = 0;
remove.lastIndex = { valueOf() { calls++; remove.compile(/a/); return 1; } };
enable.lastIndex = { valueOf() { calls++; enable.compile(/a/g); return 1; } };
var removed = remove.exec('aa'), enabled = enable.exec('aa');
calls === 2 && removed.index === 0 && remove.global === false && remove.lastIndex === 0 &&
  enabled.index === 1 && enable.global === true && enable.lastIndex === 2;
"#,
    );
}

#[test]
fn last_index_coercion_runs_once_when_recompilation_changes_matching_paths() {
    assert_wasm_true(
        r#"
var fromCompiled = /a/g;
var fromRuntime = new RegExp(String.fromCharCode(100), 'gu');
var calls = 0;
fromCompiled.lastIndex = {
  valueOf() { calls++; fromCompiled.compile(String.fromCharCode(99), 'gu'); return 0; }
};
fromRuntime.lastIndex = {
  valueOf() { calls++; fromRuntime.compile(/(?<letter>b)/dg); return 0; }
};
var runtimeMatch = fromCompiled.exec(String.fromCharCode(99));
var compiledMatch = fromRuntime.exec('b');
calls === 2 && runtimeMatch !== null && runtimeMatch[0] === String.fromCharCode(99) &&
  fromCompiled.lastIndex === 1 && compiledMatch !== null && compiledMatch.groups.letter === 'b' &&
  compiledMatch.indices[1][0] === 0 && compiledMatch.indices[1][1] === 1 && fromRuntime.lastIndex === 1;
"#,
    );
}

#[test]
fn input_coercion_precedes_last_index_coercion_and_abrupt_values_propagate() {
    assert_wasm_true(
        r#"
var expression = /a/g, events = [], marker = {}, caught = false;
var input = { toString() {
  events.push('input');
  expression.lastIndex = { valueOf() { events.push('lastIndex'); expression.compile(/b/g); throw marker; } };
  return 'b';
} };
try { expression.exec(input); } catch (error) { caught = error === marker; }
caught && events.join(',') === 'input,lastIndex' && expression.source === 'b' && expression.lastIndex === 0;
"#,
    );
}
