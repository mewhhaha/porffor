use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_reverse_whitespace(source: &str) {
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
        .unwrap_or_else(|error| panic!("reverse whitespace failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn reverse_whitespace_includes_every_ecmascript_space_and_line_terminator() {
    assert_reverse_whitespace(
        r#"
var spaces = ['\t', '\n', '\v', '\f', '\r', ' ', '\u00A0', '\u1680',
  '\u2000', '\u2001', '\u2002', '\u2003', '\u2004', '\u2005', '\u2006',
  '\u2007', '\u2008', '\u2009', '\u200A', '\u2028', '\u2029', '\u202F',
  '\u205F', '\u3000', '\uFEFF'];
var valid = true;
for (var space of spaces) {
  var input = 'a' + space + 'b';
  valid = valid && /(?<=a\s)b/.test(input) && /(?<=a\s)b/u.test(input) &&
    /(?<=a\s)b/v.test(input) && !/(?<=a\S)b/.test(input) &&
    !/(?<=a\S)b/u.test(input) && !/(?<=a\S)b/v.test(input) &&
    /^a\sb$/.test(input) && !/^a\Sb$/.test(input);
}
valid && !/(?<=\s)/.test('') && !/(?<=\S)/.test('');
"#,
    );
}

#[test]
fn reverse_non_whitespace_uses_the_same_complement_in_every_grammar() {
    assert_reverse_whitespace(
        r#"
var characters = ['x', '_', '\0', '\u0085', '\u180E', '\u200B', '\u2060',
  '\uFFFF', '\uD800', '\uDC00'];
var valid = true;
for (var character of characters) {
  var input = 'a' + character + 'b';
  valid = valid && /(?<=a\S)b/.test(input) && /(?<=a\S)b/u.test(input) &&
    /(?<=a\S)b/v.test(input) && !/(?<=a\s)b/.test(input) &&
    !/(?<=a\s)b/u.test(input) && !/(?<=a\s)b/v.test(input) &&
    /^a\Sb$/.test(input) && !/^a\sb$/.test(input);
}
valid && /(?<=a\S)b/u.test('a😀b') && /(?<=a\S)b/v.test('a😀b') &&
!/(?<=a\S)b/.test('a😀b') && /(?<=a\S\S)b/.test('a😀b') &&
!/(?<=a\s)b/u.test('a😀b');
"#,
    );
}

#[test]
fn reverse_non_whitespace_captures_follow_utf16_or_scalar_direction() {
    assert_reverse_whitespace(
        r#"
var unit = /(?<=(\S))b/d.exec('😀b');
var scalar = /(?<=^(\S))b/du.exec('😀b');
var pair = /(?<=^(\S)(\S))b/d.exec('😀b');
var middle = /^\uD83D(?<=(\S))\uDE00$/d.exec('😀');
var lone = /(?<=^(\S))b/du.exec('\uD800b');
unit[1] === '\uDE00' && unit.indices[1][0] === 1 && unit.indices[1][1] === 2 &&
scalar[1] === '😀' && scalar.indices[1][0] === 0 && scalar.indices[1][1] === 2 &&
pair[1] === '\uD83D' && pair[2] === '\uDE00' &&
pair.indices[1][0] === 0 && pair.indices[1][1] === 1 &&
pair.indices[2][0] === 1 && pair.indices[2][1] === 2 &&
middle[1] === '\uD83D' && middle.indices[1][0] === 0 && middle.indices[1][1] === 1 &&
lone[1] === '\uD800' && lone.indices[1][0] === 0 && lone.indices[1][1] === 1 &&
/(?<=😀\s)b/u.exec('😀 b').index === 3 &&
/(?<=\uD83D\uDE00\s)b/.exec('😀 b').index === 3;
"#,
    );
}

#[test]
fn nested_assertions_and_repetitions_restore_reverse_whitespace_captures() {
    assert_reverse_whitespace(
        r#"
var repeated = /(?<=a(\s)+)b/d.exec('a \t\nb');
var words = /(?<=^(\S)+)b/d.exec('a_b');
/(?<=a(?=\s)\s)b/.test('a b') && /(?<=a\s(?<=\s))b/.test('a b') &&
/(?<=a(?=\S)\S)b/.test('a_b') && !/(?<=a(?=\S)\s)b/.test('a b') &&
/(?<!\s)b/.test('_b') && !/(?<!\s)b/.test(' b') &&
/(?<=\s\b)word/.test(' word') && /(?<=\s\b)word/u.test(' word') &&
/(?<=^\s+)word/m.test('previous\n \tword') &&
repeated[1] === ' ' && repeated.indices[1][0] === 1 && repeated.indices[1][1] === 2 &&
words[1] === 'a' && words.indices[1][0] === 0 && words.indices[1][1] === 1;
"#,
    );
}
