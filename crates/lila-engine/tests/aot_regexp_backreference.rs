use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_backreference(source: &str) {
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
        .unwrap_or_else(|error| panic!("backreference execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn reverse_numbered_and_named_references_compare_the_captured_sequence() {
    assert_backreference(
        r#"
var numbered = /(?<=\1(ab))c/d.exec('ababc');
var named = /(?<=\k<pair>(?<pair>ab))c/d.exec('ababc');
numbered[0] === 'c' && numbered[1] === 'ab' && numbered.index === 4 &&
numbered.indices[1][0] === 2 && numbered.indices[1][1] === 4 &&
named[0] === 'c' && named.groups.pair === 'ab' && named.index === 4 &&
named.indices.groups.pair[0] === 2 && named.indices.groups.pair[1] === 4 &&
/(?<=^\1(ab))c/.test('ababc') && !/(?<=^\1(ab))c/.test('xababc') &&
!/(?<=\1(ab))c/.test('baabc') && !/(?<=\1(ab))c/.test('abc');
"#,
    );
}

#[test]
fn reverse_undefined_and_empty_captures_do_not_consume_input() {
    assert_backreference(
        r#"
var forward = /(?<=(ab)\1)c/.exec('abc');
var optional = /(?<=\1(a)?)b/.exec('b');
var self = /(?<=(\1a))b/.exec('ab');
var empty = /(?<=\1())$/d.exec('x');
var origin = /^(?<=\k<empty>(?<empty>))$/.exec('');
forward[0] === 'c' && forward[1] === 'ab' && forward.index === 2 &&
optional[0] === 'b' && optional[1] === undefined && optional.index === 0 &&
self[0] === 'b' && self[1] === 'a' && empty[0] === '' && empty.index === 1 &&
empty[1] === '' && empty.indices[1][0] === 1 && empty.indices[1][1] === 1 &&
origin[0] === '' && origin.groups.empty === '';
"#,
    );
}

#[test]
fn reverse_mismatch_and_underflow_restore_choices_and_capture_boundaries() {
    assert_backreference(
        r#"
var mismatch = /(?<=\1(abab|ab))c/d.exec('abxbababc');
var underflow = /(?<=\k<pair>(?<pair>abab|ab))c/d.exec('xababc');
var alternative = /(?<=\1(ab))c|(?<=z)c/.exec('zc');
mismatch[0] === 'c' && mismatch[1] === 'ab' && mismatch.index === 8 &&
mismatch.indices[1][0] === 6 && mismatch.indices[1][1] === 8 &&
underflow[0] === 'c' && underflow.groups.pair === 'ab' && underflow.index === 5 &&
underflow.indices.groups.pair[0] === 3 && underflow.indices.groups.pair[1] === 5 &&
alternative[0] === 'c' && alternative[1] === undefined && alternative.index === 1;
"#,
    );
}

#[test]
fn nested_lookarounds_restore_forward_matching_after_reverse_references() {
    assert_backreference(
        r#"
var positive = /(?=(?<=\1(ab))c)c/.exec('ababc');
var nested = /(?<=\k<pair>(?<pair>ab)(?=c))c/.exec('ababc');
var negative = /(?<!\1(ab))c/.exec('zzabc');
positive[0] === 'c' && positive[1] === 'ab' && positive.index === 4 &&
nested[0] === 'c' && nested.groups.pair === 'ab' && nested.index === 4 &&
negative[0] === 'c' && negative[1] === undefined && negative.index === 4 &&
/(?<!\1(ab))c/.exec('ababc') === null;
"#,
    );
}

#[test]
fn reverse_references_preserve_unicode_and_legacy_surrogate_boundaries() {
    assert_backreference(
        r#"
var astral = /(?<=\1(😀))x/du.exec('😀😀x');
var named = /(?<=\k<face>(?<face>😀))x/v.exec('😀😀x');
var lone = /(?<=\1(\uDE00))x/u.exec('\uDE00\uDE00x');
var half = /(?<=\1(\uDE00))x/d.exec('😀\uDE00x');
astral[0] === 'x' && astral[1] === '😀' && astral.index === 4 &&
astral.indices[1][0] === 2 && astral.indices[1][1] === 4 &&
named[0] === 'x' && named.groups.face === '😀' && named.index === 4 &&
lone[0] === 'x' && lone[1] === '\uDE00' && lone.index === 2 &&
half[0] === 'x' && half[1] === '\uDE00' && half.index === 3 &&
half.indices[1][0] === 2 && half.indices[1][1] === 3 &&
/(?<=\1(\uDE00))x/u.exec('😀\uDE00x') === null &&
/(?<=\1(\uDE00))x/v.exec('😀\uDE00x') === null;
"#,
    );
}

#[test]
fn reverse_reference_matches_preserve_last_index_and_forward_controls() {
    assert_backreference(
        r#"
var sticky = /(?<=\1(ab))c/dy;
sticky.lastIndex = 4;
var match = sticky.exec('ababc');
var failed = sticky.exec('ababc');
var global = /(?<=\k<pair>(?<pair>ab))c/g;
var first = global.exec('ababcababc');
var second = global.exec('ababcababc');
var forward = /(?<pair>ab)\k<pair>/d.exec('xabab');
match[0] === 'c' && match.index === 4 && match.indices[0][1] === 5 &&
failed === null && sticky.lastIndex === 0 && first.index === 4 && second.index === 9 &&
global.lastIndex === 10 && forward[0] === 'abab' && forward.groups.pair === 'ab' &&
forward.indices.groups.pair[0] === 1 && forward.indices.groups.pair[1] === 3 &&
/(😀)\1/u.test('😀😀') && /(\uD83D)\1/.test('\uD83D\uD83D') &&
/(a)?\1b/.test('b');
"#,
    );
}
