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
