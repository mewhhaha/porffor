use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};
fn wasm_run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    }
}

fn assert_backreference(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(source, CompileOptions::default(), wasm_run_options())
        .unwrap_or_else(|error| {
            panic!("case-folded backreference execution failed: {error}\n{source}")
        });
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn numbered_and_named_references_follow_local_modifiers_and_preserve_capture_text() {
    assert_backreference(
        r#"
var numbered = /(a)\1/di.exec('aA');
var named = /(?<pair>ab)\k<pair>/di.exec('aBAb');
numbered[0] === 'aA' && numbered[1] === 'a' && numbered.indices[1][1] === 1 &&
named[0] === 'aBAb' && named.groups.pair === 'aB' && named.indices.groups.pair[1] === 2 &&
/(a)(?i:\1)/.test('aA') && /(?<a>a)(?i:\k<a>)/.test('aA') &&
!/(?i:(a))\1/.test('Aa') && !/(?i:(?<a>a))\k<a>/.test('Aa') &&
!/(a)(?-i:\1)/i.test('aA') && !/(?<a>a)(?-i:\k<a>)/i.test('aA') &&
/(?-i:(a))\1/i.test('aA') &&
/(a)(?i:(?-i:\1)\1)\1/.test('aaAa') &&
!/(a)(?i:(?-i:\1)\1)\1/.test('aAAa') &&
/^(?:(?<a>a)|(?<a>b))(?i:\k<a>)$/.test('aA') &&
/^(?:(?<a>a)|(?<a>b))(?i:\k<a>)$/.test('bB');
"#,
    );
}

#[test]
fn legacy_and_unicode_canonicalization_have_distinct_case_equivalence() {
    assert_backreference(
        r#"
/^(.)\1$/i.test('éÉ') && /^(.)\1$/i.test('σς') &&
!/^(.)\1$/i.test('Kk') && !/^(.)\1$/i.test('ſs') && !/^(.)\1$/i.test('ßẞ') &&
/^(.)\1$/ui.test('Kk') && /^(.)\1$/ui.test('ſS') && /^(.)\1$/ui.test('ßẞ') &&
/^(?<letter>.)\k<letter>$/vi.test('KK') && /^(?<letter>.)\k<letter>$/vi.test('ßẞ') &&
!/^(.+)\1$/ui.test('ßSS') && !/^(.+)\1$/vi.test('SSß') &&
!/^(.)\1$/ui.test('Iı') && !/^(.)\1$/vi.test('iİ') && !/^(.)\1$/i.test('Iı') &&
/^(.)\1$/ui.test('𐐀𐐨') && /^(?<letter>.)\k<letter>$/vi.test('𐐨𐐀') &&
!/^(.+)\1$/i.test('𐐀𐐨') &&
/^(.)\1$/ui.test('\uD800\uD800') && /^(.)\1$/i.test('\uDC00\uDC00');
"#,
    );
}

#[test]
fn reverse_folding_reads_complete_scalars_and_resolves_modifiers_at_the_reference() {
    assert_backreference(
        r#"
var numbered = /(?<=\1(ab))x/di.exec('ABaBx');
var named = /(?<=\k<pair>(?<pair>ab))x/di.exec('ABaBx');
var astral = /(?<=\1(.))x/dui.exec('𐐀𐐨x');
var unicodeSets = /(?<=\k<letter>(?<letter>.))x/dvi.exec('𐐨𐐀x');
numbered.index === 4 && numbered[1] === 'aB' && numbered.indices[1][0] === 2 &&
named.index === 4 && named.groups.pair === 'aB' &&
astral.index === 4 && astral[1] === '𐐨' && astral.indices[1][0] === 2 && astral.indices[1][1] === 4 &&
unicodeSets.index === 4 && unicodeSets.groups.letter === '𐐀' &&
/(?<=(?i:\1)(a))x/.test('Aax') && /(?<=(?i:\k<a>)(?<a>a))x/.test('Aax') &&
!/(?<=(?-i:\1)(a))x/i.test('aAx') && !/(?<=(?-i:\k<a>)(?<a>a))x/i.test('aAx') &&
!/(?<=\1(.))x/i.test('Kkx') && /(?<=\1(.))x/ui.test('Kkx') &&
!/(?<=\1(.))x/ui.test('Iıx') &&
/(?<=\1(.))x/vi.exec('ßSSx').index === 3 && !/(?<=^\1(.))x/vi.test('ßSSx');
"#,
    );
}

#[test]
fn undefined_and_empty_captures_keep_zero_width_behavior_with_folding() {
    assert_backreference(
        r#"
var optional = /^(a)?\1b$/i.exec('b');
var empty = /()\1$/ui.exec('x');
var reverseEmpty = /(?<=\1())$/vi.exec('x');
var reverseUndefined = /(?<=(a)\1)b/i.exec('ab');
var repeated = /^(a*)\1*$/i.exec('');
optional[1] === undefined && empty.index === 1 && empty[0] === '' && empty[1] === '' &&
reverseEmpty.index === 1 && reverseEmpty[1] === '' &&
reverseUndefined.index === 1 && reverseUndefined[1] === 'a' && repeated[1] === '' &&
/^(?<a>a)?\k<a>b$/ui.test('b') && /^(?<a>)\k<a>$/vi.test('') &&
/^(?<=\k<a>(?<a>))$/ui.test('') && /^(\1a)$/i.test('a');
"#,
    );
}

#[test]
fn partial_mismatch_and_underflow_restore_cursors_captures_and_direction() {
    assert_backreference(
        r#"
var forward = /(abab|ab)\1c/di.exec('abABc');
var reverse = /(?<=\1(abab|ab))c/di.exec('abxbABabc');
var underflow = /(?<=\k<pair>(?<pair>abab|ab))c/di.exec('bABabc');
var failedAlternative = /(?<=\1(ab))c|(?<=z)c/i.exec('zc');
var nested = /(?=(?<=\1(ab))c)c/i.exec('ABabc');
var negative = /(?<!\1(ab))c/i.exec('zzabc');
forward[0] === 'abABc' && forward[1] === 'ab' && forward.indices[1][1] === 2 &&
reverse.index === 8 && reverse[1] === 'ab' && reverse.indices[1][0] === 6 &&
underflow.index === 5 && underflow.groups.pair === 'ab' && underflow.indices.groups.pair[0] === 3 &&
failedAlternative.index === 1 && failedAlternative[1] === undefined &&
nested.index === 4 && nested[1] === 'ab' && negative.index === 4 && negative[1] === undefined &&
/(?<!\1(ab))c/i.exec('ABabc') === null;
"#,
    );
}

#[test]
fn folded_references_preserve_legacy_half_surrogate_captures_and_unicode_boundaries() {
    assert_backreference(
        r#"
var reverse = /(?<=\1(\uDE00))x/di.exec('😀\uDE00x');
var leading = /(?<=\1(\uD83D))\uDE00x/di.exec('\uD83D😀x');
var forward = /(\uD83D)\1/di.exec('\uD83D😀');
reverse.index === 3 && reverse[1] === '\uDE00' && reverse.indices[1][0] === 2 &&
leading.index === 2 && leading[1] === '\uD83D' && leading.indices[1][0] === 1 &&
forward[0] === '\uD83D\uD83D' && forward.indices[0][1] === 2 &&
/(?<=\1(\uDE00))x/ui.exec('😀\uDE00x') === null &&
/(?<=\1(\uDE00))x/vi.exec('😀\uDE00x') === null &&
/(?<=\1(\uDE00))x/ui.test('\uDE00\uDE00x') &&
/(?<=\k<face>(?<face>😀))x/ui.test('😀😀x');
"#,
    );
}

#[test]
fn shared_programs_select_both_fold_tables_and_preserve_construction_and_last_index() {
    assert_backreference(
        r#"
var legacy = /^(.)\1$/i;
var unicode = /^(.)\1$/ui;
var constructed = new RegExp('^(.)\\1$', 'vi');
var sticky = /(a)\1/diy;
sticky.lastIndex = 1;
var match = sticky.exec('xaA');
var failed = sticky.exec('xaA');
var global = /(?<=\1(a))x/gi;
var first = global.exec('AaxAAx');
var second = global.exec('AaxAAx');
!legacy.test('Kk') && unicode.test('Kk') && constructed.test('Kk') &&
legacy.source === '^(.)\\1$' && unicode.source === '^(.)\\1$' && constructed.source === '^(.)\\1$' &&
match.index === 1 && match[1] === 'a' && match.indices[0][1] === 3 &&
failed === null && sticky.lastIndex === 0 && first.index === 2 && second.index === 5 && global.lastIndex === 6;
"#,
    );
}
