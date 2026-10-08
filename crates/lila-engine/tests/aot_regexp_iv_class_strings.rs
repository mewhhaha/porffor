use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_iv_class_strings(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &script,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("/iv class strings failed: {error}\n{script}"));
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{script}"
        );
        assert!(outcome.output_events.is_empty(), "{script}");
    }
}

#[test]
fn singletons_fold_before_intersection_subtraction_and_complement() {
    assert_iv_class_strings(
        r#"
    /^[\q{a}&&A]$/iv.test('a') && /^[\q{a}&&A]$/iv.test('A') &&
    /^[\q{K}&&\u212A]$/iv.test('k') && /^[\q{K}&&\u212A]$/iv.test('K') &&
    !/[\q{a}--A]/iv.test('aA') && !/[^\q{K}]/iv.test('kKK') &&
    /^[^\q{K}]$/iv.test('b') && /^[\q{ab}&&A]$/iv.test('x') === false &&
    /^[^\q{ab}&&A]$/iv.test('a') && /^[[^\q{K}]&&\q{K|b}]$/iv.test('B');
    "#,
    );
}

#[test]
fn sequences_fold_before_union_intersection_and_subtraction() {
    assert_iv_class_strings(
        r#"
    /^[\q{Ab|aB}\q{CD}]$/iv.test('AB') && /^[\q{Ab|aB}\q{CD}]$/iv.test('cd') &&
    /^[\q{Ab|CD}&&\q{aB|ef}]$/iv.test('aB') &&
    !/^[\q{Ab|CD}&&\q{aB|ef}]$/iv.test('CD') &&
    /^[\q{Ab|CD}--\q{aB|ef}]$/iv.test('cD') &&
    !/[\q{Ab}--\q{aB}]/iv.test('ABab') &&
    /^[\q{Ab|CD}&&[\q{AB|cd}]]$/iv.test('Cd') &&
    !/^[\q{Ab}&&\q{aB}]$/v.test('AB');
    "#,
    );
}

#[test]
fn unicode_sequences_use_simple_folding_without_full_fold_expansions() {
    assert_iv_class_strings(
        r#"
    /^[\q{ΣKſ}]$/iv.test('ςKS') && /^[\q{ΣKſ}]$/iv.test('σks') &&
    /^[\q{ßß}]$/iv.test('ẞß') && !/^[\q{ßß}]$/iv.test('SSSS') &&
    /^[\q{SS}]$/iv.test('sſ') && !/^[\q{SS}]$/iv.test('ß') &&
    /^[\q{IX}]$/iv.test('ix') && !/^[\q{IX}]$/iv.test('ıx') &&
    !/^[\q{IX}]$/iv.test('İx');
    "#,
    );
}

#[test]
fn supplementary_and_lone_surrogate_members_preserve_utf16_indices() {
    assert_iv_class_strings(
        r#"
    var scalar = /[\q{\u{10400}X}]/div.exec('!\u{10428}x?');
    var lone = /[\q{\uD800X}]/div.exec('!\uD800x?');
    scalar[0] === '\u{10428}x' && scalar.index === 1 &&
    scalar.indices[0][0] === 1 && scalar.indices[0][1] === 4 &&
    lone[0] === '\uD800x' && lone.index === 1 &&
    lone.indices[0][0] === 1 && lone.indices[0][1] === 3 &&
    !/^[\q{\uD800X}]$/iv.test('\uDC00x');
    "#,
    );
}

#[test]
fn alternatives_try_longest_then_singleton_then_empty_and_can_backtrack() {
    assert_iv_class_strings(
        r#"
    var longest = /[\q{a|ab|abc|}]/iv.exec('ABC');
    var shorter = /^([\q{ab|abc|}])c$/iv.exec('ABC');
    var singleton = /[\q{a|abc|}]/iv.exec('A!');
    var empty = /[\q{a|abc|}]/iv.exec('!');
    longest[0] === 'ABC' && shorter[1] === 'AB' &&
    singleton[0] === 'A' && empty[0] === '' && empty.index === 0;
    "#,
    );
}

#[test]
fn backward_matching_keeps_folded_sequence_order_and_capture_indices() {
    assert_iv_class_strings(
        r#"
    var longest = /(?<=^([\q{ab|abc|}]))z/div.exec('ABCz');
    var empty = /(?<=^([\q{ab|abc|}]))z/div.exec('z');
    var folded = /(?<=^([\q{Kſ}]))z/div.exec('KSz');
    longest.index === 3 && longest[1] === 'ABC' &&
    longest.indices[1][0] === 0 && longest.indices[1][1] === 3 &&
    empty.index === 0 && empty[1] === '' &&
    empty.indices[1][0] === 0 && empty.indices[1][1] === 0 &&
    folded[1] === 'KS' && folded.indices[1][1] === 2 &&
    !/(?<![\q{Kſ}])z/iv.test('KSz');
    "#,
    );
}

#[test]
fn scoped_modifiers_control_operand_and_matching_folding_together() {
    assert_iv_class_strings(
        r#"
    /^(?i:[\q{Ab}])(?-i:[\q{CD}])$/v.test('aBCD') &&
    !/^(?i:[\q{Ab}])(?-i:[\q{CD}])$/v.test('ABcd') &&
    /^(?-i:[\q{Ab}])(?i:[\q{CD}])$/iv.test('Abcd') &&
    !/^(?-i:[\q{Ab}])(?i:[\q{CD}])$/iv.test('abcd') &&
    /^[\q{Ab}]$/v.test('Ab') && !/^[\q{Ab}]$/v.test('ab');
    "#,
    );
}

#[test]
fn nullable_folded_string_repetitions_keep_progress_and_property_adjacency() {
    assert_iv_class_strings(
        r#"
    /^(?:[\q{ab|}])*$/iv.test('ABab') && /^(?:[\q{ab|}])*$/iv.test('') &&
    !/^(?:[\q{ab|}])*$/iv.test('ABx') &&
    /^[\q{9\uFE0F\u20E3}&&\p{Emoji_Keycap_Sequence}]$/iv.test('9\uFE0F\u20E3') &&
    /^[\q{AB|}--\q{ab}]$/iv.test('') &&
    !/^[\q{AB|}--\q{ab}]$/iv.test('AB');
    "#,
    );
}

#[test]
fn property_and_direct_strings_fold_circled_m_before_algebra() {
    assert_iv_class_strings(
        r#"
    /^\p{Basic_Emoji}$/iv.test('\u24DC\uFE0F') &&
    /^\p{RGI_Emoji}$/iv.test('\u24DC\uFE0F') &&
    /^[\p{Basic_Emoji}&&\q{\u24C2\uFE0F}]$/iv.test('\u24DC\uFE0F') &&
    /^[\p{RGI_Emoji}&&\q{\u24DC\uFE0F}]$/iv.test('\u24C2\uFE0F') &&
    !/^[\p{Basic_Emoji}--\q{\u24DC\uFE0F}]$/iv.test('\u24C2\uFE0F') &&
    !/^[\q{\u24C2\uFE0F}--\p{RGI_Emoji}]$/iv.test('\u24DC\uFE0F') &&
    /^\p{Basic_Emoji}$/v.test('\u24C2\uFE0F') &&
    !/^\p{Basic_Emoji}$/v.test('\u24DC\uFE0F') &&
    /^(?i:[\p{Basic_Emoji}&&\q{\u24DC\uFE0F}])$/v.test('\u24C2\uFE0F') &&
    !/^(?-i:[\p{RGI_Emoji}&&\q{\u24DC\uFE0F}])$/iv.test('\u24C2\uFE0F');
    "#,
    );
}
