use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../../src/builtins/promise.rs");
const REACTION_PAIR_SOURCE: &str =
    include_str!("../../src/builtins/promise/promise_combinator_reaction_pair.rs");
const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/promise-combinator-reaction-pair-ownership.md"
);
const MODE_CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/promise-combinator-mode-domains.md");
const PROMISE_TASK: &str = include_str!("../../../../tasks/14-promises-jobs-async.md");
const MODULARITY_TASK: &str =
    include_str!("../../../../tasks/02-modularize-ir-and-wasm-backend.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn rust_sources(dir: &Path) -> String {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            if path.is_dir() {
                return rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return String::new();
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        })
        .collect()
}

#[test]
fn reaction_pair_is_the_exact_private_non_copy_authority() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_combinator_reaction_pair;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_combinator_reaction_pair;"));
    assert!(!PROMISE_SOURCE.contains("promise_combinator_reaction_pair::"));
    assert!(!PROMISE_SOURCE.contains("PromiseCombinatorReactionPairLocals"));
    assert!(REACTION_PAIR_SOURCE.lines().count() <= 90);
    assert!(REACTION_PAIR_SOURCE.contains(concat!(
        "#[must_use = \"a Promise combinator reaction pair must be consumed by its then ",
        "invocation\"]\nstruct PromiseCombinatorReactionPairLocals<'a> {"
    )));
    let declaration = normalized(bounded(
        REACTION_PAIR_SOURCE,
        "struct PromiseCombinatorReactionPairLocals<'a> {",
        "impl FunctionBuilder<'_>",
    ));
    assert_eq!(
        declaration,
        "on_fulfilled:&'aValueLocals,on_rejected:&'aValueLocals,}"
    );
    for forbidden in [
        "#[derive",
        "#[derive(Clone, Copy)]\nstruct PromiseCombinatorReactionPairLocals",
        "impl Clone for PromiseCombinatorReactionPairLocals",
        "impl Copy for PromiseCombinatorReactionPairLocals",
        "impl Debug for PromiseCombinatorReactionPairLocals",
        "impl PartialEq for PromiseCombinatorReactionPairLocals",
        "impl Eq for PromiseCombinatorReactionPairLocals",
        "impl Default for PromiseCombinatorReactionPairLocals",
    ] {
        assert!(
            !REACTION_PAIR_SOURCE.contains(forbidden),
            "found `{forbidden}`"
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let all_sources = rust_sources(&source_root);
    assert_eq!(
        all_sources
            .matches("PromiseCombinatorReactionPairLocals")
            .count(),
        6
    );
    assert!(!all_sources.contains("promise_combinator_reaction_pair::"));
    assert_eq!(
        REACTION_PAIR_SOURCE
            .matches("pub(super) fn emit_invoke_promise_combinator_reaction_pair(")
            .count(),
        1,
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches(".emit_invoke_promise_combinator_reaction_pair(")
            .count(),
        1,
    );
}

#[test]
fn one_exhaustive_mode_match_selects_both_callback_roles() {
    let selection = normalized(bounded(
        REACTION_PAIR_SOURCE,
        "        let reaction_pair = match mode {",
        "        let PromiseCombinatorReactionPairLocals {",
    ));
    assert_eq!(
        selection
            .matches("PromiseCombinatorReactionPairLocals{")
            .count(),
        4
    );
    for (mode, fulfilled, rejected) in [
        ("Values", "resolve_element", "reject"),
        ("SettledRecords", "resolve_element", "reject_element"),
        ("FirstFulfillment", "resolve", "reject_element"),
        ("Race", "resolve", "reject"),
    ] {
        let expected = format!(
            "PromiseCombinatorMode::{mode}=>PromiseCombinatorReactionPairLocals{{on_fulfilled:{fulfilled},on_rejected:{rejected},}}"
        );
        assert_eq!(
            selection.matches(&expected).count(),
            1,
            "complete callback roles for {mode}"
        );
    }
    assert!(!selection.contains("_=>"));
}

#[test]
fn then_invocation_consumes_only_the_selected_pair() {
    let handoff = normalized(bounded(
        REACTION_PAIR_SOURCE,
        "        let PromiseCombinatorReactionPairLocals {",
        "        emitted\n",
    ));
    assert_eq!(
        handoff
            .matches("on_fulfilled,on_rejected,}=reaction_pair;")
            .count(),
        1
    );
    assert!(handoff
        .contains("self.emit_pre_evaluated_arg_vector(&[on_fulfilled,on_rejected],function)"));
    assert!(handoff.contains(
        "self.emit_function_or_proxy_call_with_argv(then,promise,&arguments,result,function)"
    ));
    assert!(
        handoff.find("emit_pre_evaluated_arg_vector").unwrap()
            < handoff
                .find("emit_function_or_proxy_call_with_argv")
                .unwrap()
    );
    assert!(
        handoff
            .find("emit_function_or_proxy_call_with_argv")
            .unwrap()
            < handoff.find("arguments.clear(function)").unwrap()
    );
    for removed in [
        "on_fulfilled_payload_local",
        "on_fulfilled_tag_local",
        "on_rejected_payload_local",
        "on_rejected_tag_local",
        "matchmode",
    ] {
        assert!(!handoff.contains(removed), "found `{removed}`");
    }

    for text in [CONTRACT, MODE_CONTRACT, PROMISE_TASK, MODULARITY_TASK] {
        assert!(text.contains("PromiseCombinatorReactionPairLocals"));
        assert!(text.contains("promise_combinator_reaction_pair_ownership_structure"));
    }
    assert!(CONTRACT.contains("run_wasm_backend_distinguishes_all_promise_combinator_modes"));
    assert!(CONTRACT.contains("source-equivalent"));
}
