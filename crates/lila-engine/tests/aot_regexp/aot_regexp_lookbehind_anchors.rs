use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_lookbehind(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("lookbehind execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn anchored_negative_lookbehind_restores_captures_after_failed_alternatives() {
    assert_lookbehind(
        r#"
var result = 'abcdef'.match(/(?<!(^|[ab]))\w{2}/);
result.length === 2 && result[0] === 'de' && result[1] === undefined && result.index === 3 &&
  /(?<!^abc)def/.exec('abcdef') === null && /(?<!^abc)def/.exec('xabcdef')[0] === 'def';
"#,
    );
}

#[test]
fn anchored_positive_lookbehind_checks_the_reverse_cursor_without_consuming_it() {
    assert_lookbehind(
        r#"
var result = /(?<=^(abc))def/.exec('abcdef');
result[0] === 'def' && result[1] === 'abc' && result.index === 3 &&
  /(?<=^abc)def/.exec('xabcdef') === null &&
  /(?<=^[^a-c]{3})def/.exec('abcdef') === null &&
  'foo'.match(/^foo(?<=^fo+)$/)[0] === 'foo' &&
  'foooo'.match(/^foooo(?<=^fo*)/)[0] === 'foooo';
"#,
    );
}

#[test]
fn multiline_lookbehind_anchors_use_line_boundaries_in_both_directions() {
    assert_lookbehind(
        r#"
var text = 'ab\ncd\nefg';
var starts = text.match(/(?<=^)\w+/gm);
var ends = text.match(/\w+(?<=$)/gm);
var both = text.match(/(?<=^)\w+(?<=$)/gm);
starts.join(',') === 'ab,cd,efg' && ends.join(',') === starts.join(',') &&
  both.join(',') === starts.join(',') && /(?<=^abc)def/m.exec('xyz\nabcdef')[0] === 'def' &&
  /(?<=^abc)def/m.exec('xyz\u2028abcdef')[0] === 'def' &&
  /(?<=^abc)def/m.exec('xyz\u2029abcdef')[0] === 'def';
"#,
    );
}

#[test]
fn multiline_start_anchors_reject_positions_between_utf16_surrogates() {
    assert_lookbehind(
        r#"
for (var separator of ['\n', '\r', '\u2028', '\u2029']) {
  var text = separator + '\uD83D\uDE00';
  var high = /^\uD83D/m.exec(text);
  var reverseHighBoundary = /(?<=^)\uD83D/m.exec(text);
  var reverseHigh = /(?<=^\uD83D)\uDE00/m.exec(text);
  var negativeBoundary = /(?<!^)\uDE00/m.exec(text);
  if (high === null || high[0] !== '\uD83D' || high.index !== 1 ||
      reverseHighBoundary === null || reverseHighBoundary.index !== 1 ||
      /^\uDE00/m.exec(text) !== null || /(?<=^)\uDE00/m.exec(text) !== null ||
      reverseHigh === null || reverseHigh[0] !== '\uDE00' || reverseHigh.index !== 2 ||
      negativeBoundary === null || negativeBoundary[0] !== '\uDE00' || negativeBoundary.index !== 2) {
    throw 'multiline surrogate boundary';
  }
}
true;
"#,
    );
}
