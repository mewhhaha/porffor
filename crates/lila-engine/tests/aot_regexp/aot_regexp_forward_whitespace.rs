use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_whitespace(source: &str) {
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
        .unwrap_or_else(|error| panic!("whitespace matching failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn non_whitespace_captures_use_unicode_scalars_or_legacy_utf16_units() {
    assert_whitespace(
        r#"
var scalar = /^(\S)$/du.exec('😀');
var setScalar = /^(\S)$/dv.exec('😀');
var units = /^(\S)(\S)$/d.exec('😀');
scalar[1] === '😀' && scalar.indices[1][0] === 0 && scalar.indices[1][1] === 2 &&
setScalar[1] === '😀' && setScalar.indices[1][1] === 2 &&
units[1] === '\uD83D' && units[2] === '\uDE00' &&
units.indices[1][0] === 0 && units.indices[1][1] === 1 &&
units.indices[2][0] === 1 && units.indices[2][1] === 2 &&
/^\S$/.exec('😀') === null;
"#,
    );
}

#[test]
fn non_whitespace_keeps_sticky_and_global_utf16_positions() {
    assert_whitespace(
        r#"
var sticky = /\S/dy;
sticky.lastIndex = 1;
var low = sticky.exec('😀');
var singles = '😀 x'.match(/\S/g);
var scalars = '😀 x'.match(/\S/gu);
low[0] === '\uDE00' && low.index === 1 && low.indices[0][1] === 2 &&
sticky.lastIndex === 2 && singles.length === 3 &&
singles[0] === '\uD83D' && singles[1] === '\uDE00' && singles[2] === 'x' &&
scalars.length === 2 && scalars[0] === '😀' && scalars[1] === 'x';
"#,
    );
}

#[test]
fn whitespace_and_non_whitespace_restore_cursor_after_backtracking() {
    assert_whitespace(
        r#"
var backtrack = /^(\S+)\S\s(\S)$/du.exec('😀x y');
var halves = /^(\S+)\S$/d.exec('😀');
var lone = /^(\S)\s(\S)$/du.exec('\uD800 \uDC00');
backtrack[1] === '😀' && backtrack.indices[1][1] === 2 &&
backtrack[2] === 'y' && backtrack.indices[2][0] === 4 &&
halves[1] === '\uD83D' && halves.indices[1][1] === 1 &&
lone[1] === '\uD800' && lone[2] === '\uDC00' &&
!/^\s$/u.test('😀') && /^\S\s\S$/u.test('😀\u2028😀');
"#,
    );
}

#[test]
fn dot_and_negative_classes_share_the_same_character_advance() {
    assert_whitespace(
        r#"
var dot = /^(.)$/du.exec('😀');
var negative = /^([^a])$/du.exec('😀');
var dotHalves = /^(.)(.)$/d.exec('😀');
var classHalves = /^([^a])([^a])$/d.exec('😀');
dot[1] === '😀' && dot.indices[1][1] === 2 &&
negative[1] === '😀' && negative.indices[1][1] === 2 &&
dotHalves[1] === '\uD83D' && dotHalves[2] === '\uDE00' &&
classHalves[1] === '\uD83D' && classHalves[2] === '\uDE00';
"#,
    );
}
