use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_literal_and_computed(cases: &[(&str, &str, &str, &str)]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let mut source = String::from(
        "function computed(units, flags) { var text = ''; for (var i = 0; i < units.length; i++) text += String.fromCharCode(units[i]); return new RegExp(text, flags); }\n",
    );
    for (pattern, flags, input, expected) in cases {
        let units: Vec<u16> = pattern.encode_utf16().collect();
        source.push_str(&format!(
            "var expressions = [/{pattern}/{flags}, computed({units:?}, {flags:?})];\nfor (var e = 0; e < expressions.length; e++) {{ var m = expressions[e].exec({input:?}); if (!({expected})) throw new Error('required choice run failed: ' + e); }}\n",
        ));
    }
    source.push_str("true;");
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
            .unwrap_or_else(|error| panic!("required choice run failed: {error}\n{script}"));
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{script}"
        );
    }
}

#[test]
fn pure_empty_producer_composition_preserves_real_continuation_effects() {
    assert_literal_and_computed(&[
        (r"^(?:(?:|){18446744073709551616}){18446744073709551616}$", "d", "", "m !== null && m[0] === '' && m.length === 1"),
        (r"^(?:(?:|){18446744073709551616}){18446744073709551616}b$", "d", "a", "m === null"),
        (r"^(?:(?:(?:|){18446744073709551616}){18446744073709551616}b|(?<older>a))$", "d", "a", "m !== null && m[0] === 'a' && m.groups.older === 'a' && m.indices.groups.older[0] === 0 && m.indices.groups.older[1] === 1"),
        (r"^(?:(?i:|)(?-i:||)){18446744073709551616,18446744073709551618}?a$", "d", "a", "m !== null && m[0] === 'a' && m.length === 1"),
        (r"^(?<value>(?:|){18446744073709551616})\k<value>a$", "d", "a", "m !== null && m.groups.value === '' && m.indices.groups.value[0] === 0 && m.indices.groups.value[1] === 0"),
        (r"(?<=^((?:|){18446744073709551616}))a", "d", "a", "m !== null && m[0] === 'a' && m.index === 0 && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        // These superficially empty atoms retain their actual capture,
        // failable assertion, reference and finite-set operations.
        (r"^(?:()|){3}a$", "d", "a", "m !== null && m[1] === '' && m.indices[1][0] === 0"),
        (r"a(?:^|){3}b$", "d", "ab", "m !== null && m[0] === 'ab'"),
        (r"^(?:(?!)|){3}a$", "d", "a", "m !== null && m[0] === 'a'"),
        (r"^(a)(?:\1|){3}b$", "d", "aaaab", "m !== null && m[0] === 'aaaab' && m[1] === 'a'"),
        (r"^(?:[\q{|a}]){3}b$", "dv", "aab", "m !== null && m[0] === 'aab'"),
    ]);
}

#[test]
fn independent_zero_width_choices_keep_last_iteration_priority_and_exact_optional_gap() {
    assert_literal_and_computed(&[
        // Both successful alternatives reach the actual End. The whole body
        // proof keeps one real iteration rather than weakening End observation.
        (r"^(?:()|()){18446744073709551616}b$", "d", "c", "m === null"),
        (r"^(?:(?<first>)|(?<second>)){18446744073709551616}b$", "d", "b", "m !== null && m.groups.first === '' && m.indices.groups.first[0] === 0 && m.groups.second === undefined"),
        // Continuation failure must select the last iteration's second capture
        // alternative; earlier iterations have no state after its entry Clear.
        (r"^(?:(?=(?<short>a))|(?=(?<long>ab))){18446744073709551616}\k<long>c$", "d", "abc", "m !== null && m.groups.short === undefined && m.groups.long === 'ab' && m.indices.groups.long[1] === 2"),
        (r"^(?:(?!(?<never>z))(?=(?<head>a))|(?<empty>)){18446744073709551616,18446744073709551618}?\k<head>b$", "d", "ab", "m !== null && m.groups.never === undefined && m.groups.head === 'a' && m.groups.empty === undefined"),
        (r"^(a)(?:(?=\1)|(?<empty>)){18446744073709551616}ab$", "d", "aab", "m !== null && m[1] === 'a' && m.groups.empty === undefined"),
        (r"a(?:^|(?<empty>)){18446744073709551616}b$", "d", "ab", "m !== null && m.groups.empty === '' && m.indices.groups.empty[0] === 1"),
        (r"(?<=^ab(?:(?<first>)|(?<second>)){18446744073709551616})c", "d", "abc", "m !== null && m.index === 2 && m.groups.first === '' && m.indices.groups.first[0] === 2 && m.groups.second === undefined"),
        (r"^(?:(?:(?<first>)|(?<second>)){18446744073709551616}){18446744073709551616}b$", "d", "c", "m === null"),
        // Zero mandatory repetitions keep the original optional no-progress
        // refusal and its unmatched captures. Nonzero alternatives remain real.
        (r"^(?:(?<first>)|(?<second>)){0,18446744073709551616}c$", "d", "c", "m !== null && m.groups.first === undefined && m.groups.second === undefined"),
        (r"^(?:(?<empty>)|(?<atom>a)){3,5}?b$", "d", "aab", "m !== null && m.groups.empty === undefined && m.groups.atom === 'a' && m.indices.groups.atom[0] === 1"),
        (r"^(a)(?:\1|(?<empty>)){3}b$", "d", "aaaab", "m !== null && m[1] === 'a' && m.groups.empty === undefined"),
    ]);
}

#[test]
fn independent_consuming_choices_stabilize_in_priority_above_the_utf16_input_bound() {
    assert_literal_and_computed(&[
        // N=3: compare the actual N+1 path with N+2 and huge counts. Zero
        // alternatives before, between and after consuming alternatives keep
        // their distinct last-iteration capture priority.
        (r"^(?:(?<zero>)|(?<a>a)|(?<b>b)){4}c$", "d", "abc", "m !== null && m.groups.zero === undefined && m.groups.a === undefined && m.groups.b === 'b'"),
        (r"^(?:(?<zero>)|(?<a>a)|(?<b>b)){5}c$", "d", "abc", "m !== null && m.groups.zero === undefined && m.groups.a === undefined && m.groups.b === 'b'"),
        (r"^(?:(?<zero>)|(?<a>a)|(?<b>b)){18446744073709551616}c$", "d", "abc", "m !== null && m.groups.zero === undefined && m.groups.a === undefined && m.groups.b === 'b'"),
        (r"^(?:(?<a>a)|(?<zero>)|(?<b>b)){4,6}?c$", "d", "abc", "m !== null && m.groups.a === undefined && m.groups.zero === undefined && m.groups.b === 'b'"),
        (r"^(?:(?<a>a)|(?<zero>)|(?<b>b)){5,7}?c$", "d", "abc", "m !== null && m.groups.a === undefined && m.groups.zero === undefined && m.groups.b === 'b'"),
        (r"^(?:(?<a>a)|(?<b>b)|(?<zero>)){4}c$", "d", "abc", "m !== null && m.groups.a === undefined && m.groups.b === undefined && m.groups.zero === '' && m.indices.groups.zero[0] === 2"),
        (r"^(?:(?<a>a)|(?<b>b)|(?<zero>)){5}c$", "d", "abc", "m !== null && m.groups.a === undefined && m.groups.b === undefined && m.groups.zero === '' && m.indices.groups.zero[0] === 2"),
        (r"^(?:(?=(?<peek>a))|(?<atom>a)|(?<empty>)){4}\k<peek>b$", "d", "aab", "m !== null && m.groups.peek === 'a' && m.indices.groups.peek[0] === 1 && m.groups.atom === undefined && m.groups.empty === undefined"),
        (r"^(?:(?=(?<peek>a))|(?<atom>a)|(?<empty>)){5}\k<peek>b$", "d", "aab", "m !== null && m.groups.peek === 'a' && m.indices.groups.peek[0] === 1 && m.groups.atom === undefined && m.groups.empty === undefined"),
        (r"^(?<outside>a)(?:(?<empty>)|\k<outside>){18446744073709551616,18446744073709551618}?b$", "d", "aaab", "m !== null && m.groups.outside === 'a' && m.groups.empty === undefined"),
        (r"^(?:(?<empty>)|(?<atom>😀)){18446744073709551616}b$", "du", "😀😀b", "m !== null && m.groups.empty === undefined && m.groups.atom === '😀' && m.indices.groups.atom[0] === 2 && m.indices.groups.atom[1] === 4"),
        (r"(?<=^(?:(?<empty>)|(?<atom>a)){4})b", "d", "aab", "m !== null && m.index === 2 && m.groups.empty === undefined && m.groups.atom === 'a' && m.indices.groups.atom[0] === 0"),
        (r"(?<=^(?:(?<empty>)|(?<atom>a)){5})b", "d", "aab", "m !== null && m.index === 2 && m.groups.empty === undefined && m.groups.atom === 'a' && m.indices.groups.atom[0] === 0"),
        (r"^(?:(?:(?<empty>)|a){18446744073709551616}){18446744073709551616}b$", "d", "c", "m === null"),
        (r"^(?:(?:(?<empty>)|a){18446744073709551616}){18446744073709551616}b$", "d", "aab", "m !== null && m.groups.empty === undefined"),
    ]);
}

#[test]
fn exact_required_runs_keep_ordered_fallbacks_and_every_capture_snapshot() {
    assert_literal_and_computed(&[
        (r"^(?:()|a){18446744073709551616}$", "d", "", "m !== null && m[0] === '' && m[1] === '' && m.indices[1][0] === 0 && m.indices[1][1] === 0"),
        // The last virtual iteration must still try its consuming alternative.
        (r"^(?:()|a){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined && m.indices[1] === undefined"),
        // Failure after that alternative reaches the preceding virtual group;
        // its restored min=2 must run a real final iteration rather than exit.
        (r"^(?:()|a){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === undefined && m.indices[1] === undefined"),
        (r"^(?:(?<first>)|(?<second>)|a){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m.groups.first === undefined && m.groups.second === undefined && m.indices.groups.first === undefined && m.indices.groups.second === undefined"),
        (r"^(?:(?:()|a)(?:()|b)){18446744073709551616}c$", "d", "abc", "m !== null && m[0] === 'abc' && m[1] === undefined && m[2] === undefined"),
        (r"^(?:()\1|a){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined"),
        (r"^(?:(?<x>)\k<x>|a){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m.groups.x === undefined"),
    ]);
}

#[test]
fn required_run_affine_maximum_preserves_optional_and_reverse_continuations() {
    assert_literal_and_computed(&[
        (
            r"^(?:()|a){18446744073709551616,18446744073709551618}b$",
            "d",
            "aaab",
            "m !== null && m[0] === 'aaab' && m[1] === undefined",
        ),
        (
            r"^(?:()|a){18446744073709551616,18446744073709551617}?b$",
            "d",
            "aab",
            "m !== null && m[0] === 'aab' && m[1] === undefined",
        ),
        (
            r"(?<=^(?:()|a){18446744073709551616})b",
            "d",
            "ab",
            "m !== null && m[0] === 'b' && m.index === 1 && m[1] === undefined",
        ),
        // A sibling outer fallback keeps its earlier full capture slab.
        (
            r"^(?:(?:()|a){3}b|(?<outer>c))$",
            "d",
            "c",
            "m !== null && m[0] === 'c' && m[1] === undefined && m.groups.outer === 'c'",
        ),
    ]);
}

#[test]
fn ordinary_run_refusals_keep_normal_choice_assertion_and_nested_counter_semantics() {
    assert_literal_and_computed(&[
        // The initial compressor deliberately needs a surviving ordinary
        // template. A failed first alternative keeps the normal small path.
        (r"^(?:a|()){3}$", "d", "", "m !== null && m[1] === ''"),
        // Progress and optional nested children use their original authority.
        (
            r"^(?:(a?)){2}b$",
            "d",
            "ab",
            "m !== null && m[0] === 'ab' && m[1] === '' && m.indices[1][0] === 1",
        ),
        (
            r"^(?:(?=(?<x>a?))|()){2}\k<x>$",
            "d",
            "a",
            "m !== null && m.groups.x === 'a' && m[2] === undefined",
        ),
        // A finite run can actually exhaust every virtual fallback in LIFO order.
        (r"^(?:()|a){3}b$", "d", "c", "m === null"),
        (
            r"^(?:()|a){3}b$",
            "d",
            "aaab",
            "m !== null && m[0] === 'aaab' && m[1] === undefined",
        ),
    ]);
}

#[test]
fn exact_completed_child_owners_remain_inside_full_run_snapshots() {
    assert_literal_and_computed(&[
        (r"^(?:(?:()|a)(){2}){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined && m[2] === '' && m.indices[2][0] === 1 && m.indices[2][1] === 1"),
        (r"^(?:(?:()|a)(){2}){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === undefined && m[2] === '' && m.indices[2][0] === 2 && m.indices[2][1] === 2"),
        (r"^(?:(?:()|a)(?:(){2}){3}){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined && m[2] === '' && m.indices[2][0] === 1"),
        // Optional and zero-bound child rows retain the original full lifecycle.
        (r"^(?:(?:()|a)(){1,2}){4}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === undefined && m[2] === ''"),
        (r"^(?:(?:()|a)(){0}){4}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === undefined && m[2] === undefined"),
    ]);
}

#[test]
fn completed_optional_and_choice_owning_children_preserve_virtual_guard_fallbacks() {
    assert_literal_and_computed(&[
        (r"^(?:a{0,1}?){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab'"),
        (r"^(?:(?<x>a){0,1}?){18446744073709551616}b$", "d", "ab", "m !== null && m.groups.x === 'a' && m.indices.groups.x[0] === 0 && m.indices.groups.x[1] === 1"),
        (r"^(?:(?:()|a){1,2}){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined"),
        (r"^(?:(?:()|a){2}){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === undefined"),
        (r"^(?:(?:()|a)a{0}){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined"),
        (r"(?<=^(?:a{0,1}?){18446744073709551616})b", "d", "aab", "m !== null && m[0] === 'b' && m.index === 2"),
        // Exhaustion is finite here: every child Guard restores its original
        // optional row before trying the next older outer iteration.
        (r"^(?:a{0,1}?){4}b$", "d", "c", "m === null"),
        (r"^(?:(a){0,1}?){4}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === 'a' && m.indices[1][0] === 1"),
    ]);
}

#[test]
fn discharged_input_assertions_share_the_original_proof_inside_required_choice_runs() {
    assert_literal_and_computed(&[
        (
            r"^(?:(?=a+)|b){18446744073709551616}a$",
            "d",
            "a",
            "m !== null && m[0] === 'a' && m.indices[0][0] === 0 && m.indices[0][1] === 1",
        ),
        (
            r"^(?:(?!b+)|b){18446744073709551616}a$",
            "d",
            "a",
            "m !== null && m[0] === 'a'",
        ),
        // The assertion has already removed its own sentinel and progress
        // choices. An outer virtual fallback still consumes the original atom.
        (
            r"^(?:(?=a*)|a){18446744073709551616}b$",
            "d",
            "ab",
            "m !== null && m[0] === 'ab'",
        ),
        (
            r"^(?:(?!b)|a){18446744073709551616}b$",
            "d",
            "aab",
            "m !== null && m[0] === 'aab'",
        ),
        (
            r"^a(?:(?<=a)|b){18446744073709551616}$",
            "d",
            "a",
            "m !== null && m[0] === 'a'",
        ),
        (
            r"(?<=^a(?:(?=b)|c){18446744073709551616})b",
            "d",
            "ab",
            "m !== null && m[0] === 'b' && m.index === 1",
        ),
        // Capture-writing and nested assertions keep their full real lifecycle.
        (
            r"^(?:(?=(?<seen>a))|b){3}a$",
            "d",
            "a",
            "m !== null && m.groups.seen === 'a' && m.indices.groups.seen[0] === 0",
        ),
        (
            r"^(?:(?!(a))|b){3}b$",
            "d",
            "b",
            "m !== null && m[1] === undefined && m.indices[1] === undefined",
        ),
        (
            r"^(?:(?=(?=a))|b){3}a$",
            "d",
            "a",
            "m !== null && m[0] === 'a'",
        ),
    ]);
}

#[test]
fn discharged_capture_assertions_keep_the_complete_observed_choice_state() {
    assert_literal_and_computed(&[
        (
            r"^(?:(?=(?<seen>a))|b){18446744073709551616}a$",
            "d",
            "a",
            "m !== null && m.groups.seen === 'a' && m.indices.groups.seen[0] === 0",
        ),
        (
            r"^(?:(?=(?<x>a?))|()){18446744073709551616}\k<x>$",
            "d",
            "a",
            "m !== null && m.groups.x === 'a' && m[2] === undefined",
        ),
        (
            r"^(?:(?!(a))|b){18446744073709551616}b$",
            "d",
            "b",
            "m !== null && m[1] === undefined && m.indices[1] === undefined",
        ),
        (
            r"^(?:(?=(?<x>a?))|a){18446744073709551616}b$",
            "d",
            "aab",
            "m !== null && m[0] === 'aab' && m.groups.x === undefined",
        ),
        (
            r"(?<=^(?:(?<=(?<x>a))|a){18446744073709551616})b",
            "d",
            "ab",
            "m !== null && m[0] === 'b' && m.index === 1 && m.groups.x === undefined",
        ),
        (r"^(?:(?=(?<x>a))|a){4}b$", "d", "c", "m === null"),
    ]);
}

#[test]
fn discharged_nested_assertion_owners_preserve_direction_polarity_and_counter_state() {
    assert_literal_and_computed(&[
        (r"^(?:(?=(?=a))|b){18446744073709551616}a$", "d", "a", "m !== null && m[0] === 'a'"),
        (r"^(?:(?=(?=a))){18446744073709551616}a$", "d", "a", "m !== null && m[0] === 'a'"),
        (r"^a(?:(?=(?<=(?<seen>a)))(?=b)|c){18446744073709551616}b$", "d", "ab", "m !== null && m.groups.seen === 'a' && m.indices.groups.seen[0] === 0 && m.indices.groups.seen[1] === 1"),
        (r"^(?:(?!(?=b))|a){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab'"),
        (r"^(?:(?=(?!(b))(?<seen>a?))|a){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m.groups.seen === undefined && m[1] === undefined"),
        (r"^(?:(?=a{0,2}(?=a))|b){18446744073709551616}a$", "d", "a", "m !== null && m[0] === 'a'"),
        (r"(?<=^(?:(?<=(?<seen>a))|a){4})b", "d", "aab", "m !== null && m[0] === 'b' && m.index === 2 && m.groups.seen === undefined"),
        (r"^(?:(?=(?=a))|a){4}b$", "d", "c", "m === null"),
    ]);
}

#[test]
fn completed_greedy_attempts_without_live_choices_keep_the_original_continuation() {
    assert_literal_and_computed(&[
        (r"^(?:a*){18446744073709551616}b$", "d", "aaab", "m !== null && m[0] === 'aaab'"),
        (r"^(?:a?){18446744073709551616}b$", "d", "aaab", "m !== null && m[0] === 'aaab'"),
        (r"^(a*){18446744073709551616}b$", "d", "aaab", "m !== null && m[0] === 'aaab' && m[1] === '' && m.indices[1][0] === 3 && m.indices[1][1] === 3"),
        (r"^(?:(?:(?=b))*){18446744073709551616}b$", "d", "b", "m !== null && m[0] === 'b'"),
        (r"^(?:(?:(?!a))?){18446744073709551616}b$", "d", "b", "m !== null && m[0] === 'b'"),
        (r"^(?:(?:a*|b?)*){18446744073709551616}b$", "d", "aaab", "m !== null && m[0] === 'aaab'"),
        (r"^(?:(?:a{0,2})*){18446744073709551616}b$", "d", "aaab", "m !== null && m[0] === 'aaab'"),
        (r"^((?:a*|b?)*){18446744073709551616}b$", "d", "aaab", "m !== null && m[1] === '' && m.indices[1][0] === 3"),
        (r"^(?:a*){18446744073709551616,18446744073709551618}b$", "d", "aaab", "m !== null && m[0] === 'aaab'"),
        (r"(?<=^(?:a*){18446744073709551616})b", "d", "aaab", "m !== null && m[0] === 'b' && m.index === 3"),
        // The earlier real greedy choices survive batching at the input end;
        // a later failure must restore them to leave the suffix unconsumed.
        (r"^(?:.*){18446744073709551616}b$", "du", "😀b", "m !== null && m[0] === '😀b' && m.indices[0][1] === 3"),
        (r"^(?:(?:a*){18446744073709551616}b|(?<fallback>c))$", "d", "c", "m !== null && m.groups.fallback === 'c'"),
        (r"^(?:a*){3}b$", "d", "c", "m === null"),
        // Capture-owning attempts and ordinary stars with branched atoms
        // retain their original path rather than this input-only proof.
        (r"^(?:(a?)){3}b$", "d", "ab", "m !== null && m[1] === '' && m.indices[1][0] === 1"),
        (r"^(?:(?:a|b)*){3}c$", "d", "abc", "m !== null && m[0] === 'abc'"),
    ]);
}

#[test]
fn live_nullable_progress_templates_keep_logical_fallbacks_and_atomic_truncation() {
    assert_literal_and_computed(&[
        // The lazy child first omits its body. Suffix failure must search the
        // logical newest progress frame, then retain every older alternative.
        (r"^(?:(?:|a)??){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab'"),
        (r"^(?:(?:|a)??){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab'"),
        (r"^(?:(?<x>|a)??){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m.groups.x === 'a' && m.indices.groups.x[0] === 1 && m.indices.groups.x[1] === 2"),
        (r"^(?:(?<x>|😀)??){18446744073709551616}b$", "du", "😀b", "m !== null && m[0] === '😀b' && m.groups.x === '😀' && m.indices.groups.x[0] === 0 && m.indices.groups.x[1] === 2"),
        // The assertion completes atomically after searching a compressed
        // progress group. Its logical truncation must preserve the outer choice.
        (r"^(?=(?:(?:|a)??){18446744073709551616}b$)(?<seen>a*)b$", "d", "aab", "m !== null && m.groups.seen === 'aa' && m.indices.groups.seen[0] === 0 && m.indices.groups.seen[1] === 2"),
        (r"^(?:(?=(?:(?<inside>|a)??){18446744073709551616}b$)a+c|(?<fallback>aab))$", "d", "aab", "m !== null && m.groups.inside === undefined && m.indices.groups.inside === undefined && m.groups.fallback === 'aab'"),
        // Exhausting a negative assertion is intentionally finite; successful
        // negative completion clears its captures before the huge required run.
        (r"^(?!(?:(?<negative>|a)??){4}c$)(?:(?:|a)??){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m.groups.negative === undefined && m.indices.groups.negative === undefined"),
        (r"(?<=^(?:(?<x>|a)??){18446744073709551616})b", "d", "aab", "m !== null && m.index === 2 && m[0] === 'b' && m.groups.x === 'a' && m.indices.groups.x[0] === 0 && m.indices.groups.x[1] === 1"),
        (r"^(?:(?<x>|a)??){4}b$", "d", "c", "m === null"),
    ]);
}

#[test]
fn fully_exhausted_virtual_groups_keep_failure_and_outer_capture_restoration() {
    assert_literal_and_computed(&[
        // Every alternative in the observed group fails before reading the
        // outer remaining count. Its identical older groups must also fail.
        (r"^(?:()|a){18446744073709551616}b$", "d", "c", "m === null"),
        (r"^(?:()|a){18446744073709551616,18446744073709551618}?b$", "d", "c", "m === null"),
        (r"^(?:a{0,1}?){18446744073709551616}b$", "d", "c", "m === null"),
        (r"^(?:(?<x>|a)??){18446744073709551616}b$", "d", "c", "m === null"),
        (r"^(?:(?=(?<seen>))|a){18446744073709551616}b$", "d", "c", "m === null"),
        (r"^(?:()|😀){18446744073709551616}b$", "du", "c", "m === null"),
        // Retirement must restore the original older outer snapshot, including
        // captures written before the compressed group and assertion sentinels.
        (r"^(?:(?:()|a){18446744073709551616}b|(?<fallback>c))$", "d", "c", "m !== null && m[1] === undefined && m.indices[1] === undefined && m.groups.fallback === 'c'"),
        (r"^(?!(?:(?<inside>|a)??){18446744073709551616}b$)(?<outside>c)$", "d", "c", "m !== null && m.groups.inside === undefined && m.indices.groups.inside === undefined && m.groups.outside === 'c'"),
        (r"^(?:(?=(?:()|a){18446744073709551616}b$)c|(?<fallback>c))$", "d", "c", "m !== null && m[1] === undefined && m.groups.fallback === 'c'"),
        (r"(?<!^(?:()|a){18446744073709551616})b", "d", "cb", "m !== null && m.index === 1 && m[0] === 'b' && m[1] === undefined"),
        // Reaching the actual outer End invalidates the failure witness: a
        // consuming fallback must still run the remaining required iterations.
        (r"^(?:()|a){18446744073709551616}b$", "d", "aaab", "m !== null && m[0] === 'aaab' && m[1] === undefined"),
        (r"^(?:()|a){4}b$", "d", "aaaac", "m === null"),
    ]);
}

#[test]
fn nested_required_runs_restore_each_ancestor_counter_and_original_logical_choice_path() {
    assert_literal_and_computed(&[
        // Each outer template contains a whole compressed child, including its
        // current partial playback and its independent original reset state.
        (r"^(?:(?:|){18446744073709551616}){18446744073709551616}$", "d", "", "m !== null && m[0] === ''"),
        (r"^(?:(?:()|a){18446744073709551616}){18446744073709551616}b$", "d", "aab", "m !== null && m[0] === 'aab' && m[1] === undefined"),
        (r"^(?:(?:(?:()|a){18446744073709551616}){18446744073709551616}){18446744073709551616}b$", "d", "ab", "m !== null && m[0] === 'ab' && m[1] === undefined"),
        (r"^(?:(?:(?<empty>)|(?<atom>a)){18446744073709551616}){18446744073709551616}b$", "d", "aab", "m !== null && m.groups.empty === undefined && m.indices.groups.empty === undefined && m.groups.atom === 'a' && m.indices.groups.atom[0] === 1 && m.indices.groups.atom[1] === 2"),
        (r"^(?:(?<last>(?:()|a){18446744073709551616})){18446744073709551616}b$", "d", "b", "m !== null && m.groups.last === '' && m.indices.groups.last[0] === 0 && m.indices.groups.last[1] === 0 && m[2] === ''"),
        // Failed descendant groups must retain the ancestor's failure trace;
        // otherwise the older outer fallback is hidden behind H real retries.
        (r"^(?:(?:(?:|){18446744073709551616}){18446744073709551616}b|(?<older>a))$", "d", "a", "m !== null && m.groups.older === 'a'"),
        (r"^(?:(?:(?:()|a){18446744073709551616}){18446744073709551616}b|(?<older>c))$", "d", "c", "m !== null && m[1] === undefined && m.indices[1] === undefined && m.groups.older === 'c'"),
        (r"^(?:(?:(?<x>|a)??){18446744073709551616}){18446744073709551616}b$", "d", "aab", "m !== null && m.groups.x === 'a' && m.indices.groups.x[0] === 1 && m.indices.groups.x[1] === 2"),
        // The child succeeds only after consuming its newest zero-width
        // fallback. Its captured current path is partial, while its own older
        // group reset still needs both original alternatives.
        (r"^(?:(?:(?:()|(?=a))(?:()|(?=(?<mark>a)))){18446744073709551616}(?=\k<mark>b)){18446744073709551616}ab$", "d", "ab", "m !== null && m[1] === '' && m[2] === undefined && m.groups.mark === 'a' && m.indices.groups.mark[0] === 0 && m.indices.groups.mark[1] === 1"),
        (r"^(?=(?:(?:|a){18446744073709551616}){18446744073709551616}b$)(?<seen>a*)b$", "d", "aab", "m !== null && m.groups.seen === 'aa' && m.indices.groups.seen[1] === 2"),
        (r"^(?:(?=(?:(?:(?<inside>)|a){18446744073709551616}){18446744073709551616}b$)a+c|(?<older>aab))$", "d", "aab", "m !== null && m.groups.inside === undefined && m.indices.groups.inside === undefined && m.groups.older === 'aab'"),
        (r"^(?!(?:(?:|a){18446744073709551616}){18446744073709551616}b$)(?<outside>c)$", "d", "c", "m !== null && m.groups.outside === 'c'"),
        (r"(?<=^(?:(?:()|(?<x>a)){18446744073709551616}){18446744073709551616})b", "d", "aab", "m !== null && m.index === 2 && m.groups.x === 'a' && m.indices.groups.x[0] === 0 && m.indices.groups.x[1] === 1 && m[1] === undefined"),
        (r"^(?:(?:()|(?<x>😀)){18446744073709551616}){18446744073709551616}b$", "du", "😀b", "m !== null && m.groups.x === '😀' && m.indices.groups.x[0] === 0 && m.indices.groups.x[1] === 2"),
        // Finite counterparts exercise actual complete resets and exhaustion.
        (r"^(?:(?:()|a){3}){4}b$", "d", "aaaaab", "m !== null && m[0] === 'aaaaab' && m[1] === undefined"),
        (r"^(?:(?:()|a){3}){4}b$", "d", "aaaac", "m === null"),
    ]);
}
