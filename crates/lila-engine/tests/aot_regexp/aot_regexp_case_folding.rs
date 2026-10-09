use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_case_folding(source: &str) {
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
        .unwrap_or_else(|error| panic!("case folding failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn simple_folding_distinguishes_unicode_and_legacy_character_sets() {
    assert_case_folding(
        r#"
/ß/ui.test('ẞ') && /[ß]/vi.test('ẞ') && !/[ß]/i.test('ẞ') &&
/é/i.test('É') && /[σ]/i.test('ς') &&
!/[i]/ui.test('ı') && !/[I]/vi.test('İ') &&
/𐐀/ui.test('𐐨') && /\u{10400}/vi.test('𐐨') &&
!/𐐀/i.test('𐐨') && !/ß/ui.test('ss');
"#,
    );
}

#[test]
fn word_escapes_and_negated_sets_include_unicode_fold_equivalents() {
    assert_case_folding(
        r#"
/^\w+$/ui.test('Kſ') && !/\W/ui.test('Kſ') &&
/[k]/ui.test('K') && /[s]/vi.test('ſ') &&
!/[^k]/ui.test('K') && !/[^s]/vi.test('ſ') &&
!/[k]/i.test('K') && !/[s]/i.test('ſ');
"#,
    );
}

#[test]
fn word_class_complements_preserve_their_unicode_fold_members() {
    assert_case_folding(
        r#"
!/[\W]/ui.test('kKsSKſ') && /[^\W]+/ui.test('kKsSKſ') &&
!/[\W]/vi.test('kKsSKſ') && /[^\W]+/vi.test('kKsSKſ') &&
/[\W]/ui.test('!') && !/[^\W]/ui.test('!') &&
/[\W]/i.test('K') && !/[\W]/i.test('k') &&
!/[[\W]&&[k]]/vi.test('k') && /[[^\W]&&[k]]/vi.test('K');
"#,
    );
}

#[test]
fn unicode_set_algebra_closes_operands_before_intersection_subtraction_and_complement() {
    assert_case_folding(
        r#"
/[a&&A]/vi.test('a') && /[a&&A]/vi.test('A') &&
!/[a--A]/vi.test('aA') && !/[A--a]/vi.test('aA') &&
!/[^a&&A]/vi.test('aA') && /[^a--A]/vi.test('a') &&
!/[[a-z]--[A-Z]]/vi.test('aAKſ') && /[[a-z]&&[A-Z]]/vi.test('A') &&
!/[[^k]&&[K]]/vi.test('kKK') && /[[^k]&&[s]]/vi.test('ſ') &&
/[[k]&&[K]]/vi.test('K') && !/[[k]--[K]]/vi.test('kKK') &&
!/[a&&A]/v.test('a') && /[a--A]/v.test('a') &&
/(?i:[a&&A])(?-i:[a--A])/v.test('Aa');
"#,
    );
}

#[test]
fn property_complements_apply_the_distinct_unicode_and_unicode_sets_order() {
    assert_case_folding(
        r#"
/\P{Lowercase_Letter}/ui.test('a') && !/\P{Lowercase_Letter}/vi.test('a') &&
/[\P{Lowercase_Letter}]/ui.test('A') && !/[\P{Lowercase_Letter}]/vi.test('A') &&
!/[^\P{Lowercase_Letter}]/ui.test('a') && /[^\P{Lowercase_Letter}]/vi.test('a') &&
!/\P{Lowercase_Letter}/vi.test('ſK') && /\P{Lowercase_Letter}/vi.test('!') &&
/[\p{Lowercase_Letter}&&\p{Uppercase_Letter}]/vi.test('A') &&
!/[\P{Lowercase_Letter}&&\p{Uppercase_Letter}]/vi.test('A');
"#,
    );
}

#[test]
fn scoped_modifiers_apply_to_literals_ranges_and_nested_assertions() {
    assert_case_folding(
        r#"
/(?i:ß)(?-i:ß)/u.test('ẞß') && !/(?i:ß)(?-i:ß)/u.test('ẞẞ') &&
/(?i:(?<=[k])ß)/u.test('Kẞ') && /(?i:(?=[é])é)/u.test('É') &&
!/(?i:(?-i:[é]))/u.test('É') && /(?i:é)/.test('É');
"#,
    );
}

#[test]
fn multi_digit_references_and_legacy_octal_keep_distinct_grammar() {
    assert_case_folding(
        r#"
/(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)(l)\12/u.test('abcdefghijkll') &&
/(a)\10/.test('a\b') && !/(a)\10/.test('aa0') &&
/(a)\18/.test('a\x018');
"#,
    );
}
