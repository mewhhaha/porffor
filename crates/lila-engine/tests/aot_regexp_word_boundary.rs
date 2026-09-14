use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_boundaries(source: &str) {
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
        .unwrap_or_else(|error| panic!("word boundary failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn boundaries_distinguish_ascii_words_and_non_word_characters() {
    assert_boundaries(
        r#"
/\babc\b/.test('abc') && /\babc\b/.test('!abc?') &&
!/\babc\b/.test('xabc') && !/\babc\b/.test('abc0') &&
/\b_9\b/.test('_9') && !/\b_9\b/.test('a_9') &&
/a\Bb/.test('ab') && !/a\bb/.test('ab') &&
/!\B\?/.test('!?') && !/!\b\?/.test('!?') &&
/\babc\b/.test('éabcé') && !/\bé\b/.test('é');
"#,
    );
}

#[test]
fn non_boundary_matches_empty_input_and_both_non_word_edges() {
    assert_boundaries(
        r#"
!(/\b/.test('')) && /\B/.test('') && /\B/u.test('') && /\B/v.test('') &&
!(/\b/.test('!')) && /^\B!\B$/.test('!') &&
/^\bword\b$/.test('word') && !/^\Bword/.test('word') &&
/word\b$/.test('word') && !/word\B$/.test('word');
"#,
    );
}

#[test]
fn unicode_ignore_case_adds_only_canonical_ascii_word_equivalents() {
    assert_boundaries(
        r#"
/^\bſK\b$/ui.test('ſK') && /^\bſK\b$/vi.test('ſK') &&
!/^\bſK\b$/i.test('ſK') && !/^\bſK\b$/u.test('ſK') &&
/^\BſK\B$/i.test('ſK') && !/^\BſK\B$/ui.test('ſK') &&
!/^\bı\b$/ui.test('ı') && !/^\bİ\b$/ui.test('İ') &&
!/^\bß\b$/ui.test('ß') && !/^\bẞ\b$/ui.test('ẞ');
"#,
    );
}

#[test]
fn scoped_modifiers_keep_each_boundary_word_set() {
    assert_boundaries(
        r#"
/(?i:^\bſ\b$)/u.test('ſ') && !/(?i:(?-i:^\bſ\b$))/u.test('ſ') &&
/(?i:^\bſ)(?-i:\B$)/u.test('ſ') &&
/(?i:^\bſ)(?-i:\B$)/vi.test('ſ') &&
!/(?i:^\bſ)(?-i:\b$)/u.test('ſ') &&
/(?i:^\Bſ\B$)/.test('ſ');
"#,
    );
}

#[test]
fn utf16_positions_handle_astral_pairs_and_lone_surrogates() {
    assert_boundaries(
        r#"
var unicode = /\bx\b/du.exec('😀x');
var legacy = /\bx\b/d.exec('😀x');
/^\uD83D\B\uDE00$/.test('😀') && !/^\uD83D\b\uDE00$/.test('😀') &&
/^\uD83D\bx$/.test('\uD83Dx') && /^\uDE00\bx$/u.test('\uDE00x') &&
/^\uD83D\B!$/u.test('\uD83D!') && /^😀\B!$/u.test('😀!') &&
unicode.index === 2 && unicode.indices[0][0] === 2 && unicode.indices[0][1] === 3 &&
legacy.index === 2 && legacy.indices[0][0] === 2;
"#,
    );
}

#[test]
fn assertions_keep_their_orientation_inside_lookbehind_and_nested_lookahead() {
    assert_boundaries(
        r#"
var word = /(?<=\bfoo\b)/d.exec('foo ');
var nested = /(?<=a(?=\Bb))b/.exec('ab');
word.index === 3 && word[0] === '' && word.indices[0][0] === 3 && word.indices[0][1] === 3 &&
/(?<=\bfoo\b)/.exec('xfoo ') === null &&
/(?<=a\Bb)c/.test('abc') && !/(?<=a\bb)c/.test('abc') &&
/(?<!\bfoo)bar/.test('xfoobar') && !/(?<!\bfoo)bar/.test('foobar') &&
nested.index === 1 && nested[0] === 'b' &&
/(?<=😀)\bx\b/u.exec('😀x').index === 2;
"#,
    );
}

#[test]
fn zero_width_global_and_sticky_matches_preserve_utf16_indices() {
    assert_boundaries(
        r#"
var matches = 'ab cd'.match(/\b/g);
var empty = ''.match(/\B/g);
var sticky = /\b/dy;
sticky.lastIndex = 1;
var absent = sticky.exec('ab');
var reset = sticky.lastIndex === 0;
sticky.lastIndex = 2;
var end = sticky.exec('ab');
var middle = /\B/dy;
middle.lastIndex = 1;
var pair = middle.exec('😀');
matches.length === 4 && empty.length === 1 && empty[0] === '' &&
absent === null && reset && end.index === 2 && sticky.lastIndex === 2 &&
end.indices[0][0] === 2 && end.indices[0][1] === 2 &&
pair.index === 1 && pair.indices[0][0] === 1 && pair.indices[0][1] === 1;
"#,
    );
}

#[test]
fn quantified_groups_backtrack_without_repeating_empty_assertions_forever() {
    assert_boundaries(
        r#"
var nonword = /(?:(\b)|(\B))+/d.exec('!');
var consumed = /(?:(\b)|x)+/.exec('x');
var required = /(\b){2}x/.exec('x');
var empty = /(?:\B)*/.exec('');
nonword[0] === '' && nonword[1] === undefined && nonword[2] === '' &&
nonword.indices[2][0] === 0 && nonword.indices[2][1] === 0 &&
consumed[0] === 'x' && consumed[1] === undefined &&
required[0] === 'x' && required[1] === '' && empty[0] === '';
"#,
    );
}

#[test]
fn character_classes_keep_backspace_and_legacy_identity_escapes() {
    assert_boundaries(
        r#"
/^[\b]+$/.test('\b\b') && /^[\b]$/u.test('\b') && /^[\b]$/v.test('\b') &&
!/[\b]/.test('b') && /^[\B]$/.test('B') && !/\b/.test('\b');
"#,
    );
}
