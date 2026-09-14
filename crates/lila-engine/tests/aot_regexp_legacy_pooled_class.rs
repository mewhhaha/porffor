use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_pooled_class(source: &str) {
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
        .unwrap_or_else(|error| {
            panic!("pooled character class execution failed: {error}\n{source}")
        });
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn legacy_pooled_classes_consume_two_units_for_an_astral_scalar() {
    assert_pooled_class(
        r#"
var units = /^([\S])([\S])$/d.exec('😀');
var negative = /^([^é])([^é])$/d.exec('😀');
!/^([\S])$/.test('😀') && /^[\S][\S]$/.test('😀') &&
!/^([^é])$/.test('😀') && /^[^é][^é]$/.test('😀') &&
units[1] === '\uD83D' && units[2] === '\uDE00' &&
units.indices[1][0] === 0 && units.indices[1][1] === 1 &&
units.indices[2][0] === 1 && units.indices[2][1] === 2 &&
negative[1] === '\uD83D' && negative[2] === '\uDE00' &&
/^([\S])$/u.exec('😀')[1] === '😀' && /^([^é])$/v.exec('😀')[1] === '😀';
"#,
    );
}

#[test]
fn membership_uses_the_selected_surrogate_unit_and_preserves_raw_class_members() {
    assert_pooled_class(
        r#"
var ranges = /^([\uD800-\uDBFF])([\uDC00-\uDFFF])$/d.exec('😀');
var raw = /([😀])y/d.exec('😀y');
var excluded = /([^😀])y/d.exec('😁y');
ranges[1] === '\uD83D' && ranges[2] === '\uDE00' && ranges.indices[1][1] === 1 &&
/^[😀][😀]$/.test('😀') && /^[😀]$/.test('\uD83D') && /^[😀]$/.test('\uDE00') &&
/[😀]/.test('😁') && !/[😀]/u.test('😁') &&
raw.index === 1 && raw[1] === '\uDE00' && raw.indices[1][0] === 1 &&
excluded.index === 1 && excluded[1] === '\uDE01' && excluded.indices[1][1] === 2 &&
!/[\uD800-\uDBFF]/u.test('😀') && !/[\uDC00-\uDFFF]/v.test('😀') &&
/^[\uD800-\uDBFF]$/u.test('\uD800') && /[^😀]y/u.exec('😁y').index === 0;
"#,
    );
}

#[test]
fn sticky_and_global_pooled_classes_keep_utf16_positions_at_both_halves() {
    assert_pooled_class(
        r#"
var sticky = /[\uDC00-\uDFFF]/dy;
sticky.lastIndex = 1;
var low = sticky.exec('😀');
var failed = sticky.exec('😀');
var units = '😀 x'.match(/[\S]/g);
var scalars = '😀 x'.match(/[\S]/gu);
low.index === 1 && low[0] === '\uDE00' && low.indices[0][1] === 2 &&
failed === null && sticky.lastIndex === 0 && units.length === 3 &&
units[0] === '\uD83D' && units[1] === '\uDE00' && units[2] === 'x' &&
scalars.length === 2 && scalars[0] === '😀' && scalars[1] === 'x';
"#,
    );
}

#[test]
fn pooled_classes_restore_cursors_and_captures_across_choices_and_lookbehind() {
    assert_pooled_class(
        r#"
var greedy = /^([\S]+)[\S]$/d.exec('😀');
var choice = /^([\uD800-\uDFFF]+|é)[\uDC00-\uDFFF]$/d.exec('😀');
var reverse = /(?<=([\S])([\S]))x/d.exec('😀x');
var reverseRaw = /(?<=([😀])([😀]))x/d.exec('😀x');
var reverseUnicode = /(?<=([^é]))x/du.exec('😀x');
greedy[1] === '\uD83D' && greedy.indices[1][1] === 1 &&
choice[1] === '\uD83D' && choice.indices[1][1] === 1 &&
reverse[1] === '\uD83D' && reverse[2] === '\uDE00' &&
reverse.indices[1][0] === 0 && reverse.indices[2][0] === 1 &&
reverseRaw[1] === '\uD83D' && reverseRaw[2] === '\uDE00' &&
reverseUnicode[1] === '😀' && reverseUnicode.indices[1][1] === 2;
"#,
    );
}

#[test]
fn class_range_endpoints_and_case_folding_keep_legacy_and_unicode_semantics() {
    assert_pooled_class(
        r#"
var folded = /^([é])([\S])([\S])$/di.exec('É😀');
var rawRange = /^[a-😀]$/.test('\uDE00') && !/^[a-😀]$/.test('\uDE01');
var trailingRange = /^[😀-\uFFFF]$/.test('\uDE01') && !/^[😀-\uFFFF]$/.test('x');
folded[1] === 'É' && folded[2] === '\uD83D' && folded[3] === '\uDE00' &&
folded.indices[2][0] === 1 && folded.indices[3][1] === 3 &&
!/^[^é]$/i.test('É') && /^[^é][^é]$/i.test('😀') &&
/^[é😀][é😀]$/i.test('😀') && rawRange && trailingRange &&
/^[𐐀]$/ui.test('𐐨') && /^(?i:[𐐀])$/v.test('𐐨') &&
!/^[^𐐀]$/ui.test('𐐨') && /^[é😀]$/u.test('😀') &&
/[😀]/.source === '[😀]';
"#,
    );
}
