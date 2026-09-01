use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/lowering/dynamic_source.rs");
const IR_SOURCE: &str = include_str!("../src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const NON_PROPERTY_CALL_SOURCE: &str =
    include_str!("../src/lowering/call_expression/non_property_call.rs");
const CALL_CANDIDATE_SOURCE: &str = include_str!("../src/lowering/call_candidate_analysis.rs");
const INVOCATION_PROVENANCE_SOURCE: &str = include_str!("../src/lowering/define_property_call.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/dynamic-source-capability.md");
const TASK: &str = include_str!("../../../tasks/13-dynamic-source-evaluation.md");

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

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn direct_eval_context_requires_the_exact_private_call_site_witness() {
    let declaration = normalized(bounded(
        SOURCE,
        "/// The closed call-site contexts observed by standard-builtin analysis.",
        "/// A possible intrinsic direct-eval target captured before argument",
    ));
    assert_eq!(
        declaration,
        normalized(
            r#"
#[derive(Debug, PartialEq, Eq)]
pub(super) enum BuiltinCallContext {
    Call,
    DirectEval(DirectEvalCallSite),
    Construct,
    RegExpLiteral,
}

/// Proof that a resolved `%eval%` target still has direct-reference syntax.
///
/// The field is private to this module, so sibling lowering modules can route
/// ordinary calls but cannot manufacture caller-environment eval semantics.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct DirectEvalCallSite(());

"#,
        )
    );
    assert_eq!(SOURCE.matches("DirectEvalCallSite").count(), 4);
    assert!(!declaration.contains("Clone"));
    assert!(!declaration.contains("Copy"));
}

#[test]
fn only_intrinsic_eval_and_a_direct_global_reference_produce_direct_eval_authority() {
    let classifier = normalized(bounded(
        SOURCE,
        "pub(super) fn resolved_builtin_call_context(",
        "    pub(super) fn register_dynamic_source_intrinsic_signatures(",
    ));
    assert_eq!(
        classifier,
        normalized(
            r#"
        &self,
        source_callee: &Expression,
    callee: &TypedExpr,
    function_id: &FunctionId,
) -> BuiltinCallContext {
    let source_is_eval_identifier = matches!(
        Self::unwrap_parenthesized_expr(source_callee),
        Expression::Identifier(identifier)
            if self.interner.resolve_expect(identifier.sym()).to_string() == "eval"
    );
    if StandardBuiltinId::from_function_id(function_id) == Some(StandardBuiltinId::EvalFunction)
        && source_is_eval_identifier
        && matches!(
            &callee.expr,
            ExprIr::GlobalPropertyRead { name } | ExprIr::GlobalIdentifierRead { name }
                if name == "eval"
        )
    {
        BuiltinCallContext::DirectEval(DirectEvalCallSite(()))
    } else {
        BuiltinCallContext::Call
    }
}

"#,
        )
    );
    assert_eq!(
        count_in_rust_sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            "BuiltinCallContext::DirectEval(DirectEvalCallSite(()))",
        ),
        1,
        "direct-eval authority must have one construction route"
    );
}

#[test]
fn erased_direct_eval_identity_is_captured_before_arguments_and_consumed_after() {
    let authority = normalized(bounded(
        SOURCE,
        "/// A possible intrinsic direct-eval target captured before argument",
        "pub(super) fn dynamic_source_kind_for_function_id(",
    ));
    assert!(authority.contains("#[must_use="));
    assert!(authority.contains("pub(super)structErasedDirectEvalCall{"));
    assert!(authority.contains("call_site:DirectEvalCallSite"));
    assert!(!authority.contains("function_id:FunctionId"));
    assert!(!authority.contains("context:BuiltinCallContext"));
    assert!(!authority.contains("Clone"));
    assert!(!authority.contains("Copy"));

    let generic_call = normalized(bounded(
        NON_PROPERTY_CALL_SOURCE,
        "        let lower_generic_indirect_call =",
        "        if callee.kind != ValueKind::Function {",
    ));
    assert!(generic_call.contains("function_targets.exact_targets().is_none()"));
    let capture = generic_call
        .find("capture_erased_direct_eval_call(source_callee,&callee)")
        .expect("generic call must capture possible direct eval");
    let lower_args = generic_call
        .find("lower_call_args_expanding_spread(args)")
        .expect("generic call must lower arguments");
    let resolve = generic_call
        .find("erased_direct_eval.resolve(this,args,&lowered_args)")
        .expect("generic call must consume the captured identity");
    assert!(capture < lower_args);
    assert!(lower_args < resolve);
}

#[test]
fn dynamic_source_kind_consumes_the_borrowed_context_exhaustively() {
    let projection = normalized(bounded(
        SOURCE,
        "pub(super) fn dynamic_source_kind_for_function_id(",
        "const fn gap_for_source_proof(",
    ));
    assert!(projection.contains("context:&BuiltinCallContext"));
    assert_eq!(
        projection
            .matches("BuiltinCallContext::DirectEval(_)")
            .count(),
        1
    );
    for variant in ["Call", "Construct", "RegExpLiteral"] {
        assert!(
            projection.contains(&format!("BuiltinCallContext::{variant}")),
            "missing exhaustive `{variant}` projection"
        );
    }
    assert!(!projection.contains("_=>"));
    assert!(!projection.contains("context.clone()"));
    assert!(!projection.contains("context=="));
}

#[test]
fn exact_function_target_authority_is_independent_from_heap_shape() {
    let target_domain = normalized(bounded(
        IR_SOURCE,
        "pub enum FunctionTargetKnowledge {",
        "impl FunctionTargetKnowledge {",
    ));
    assert_eq!(
        target_domain,
        "Exact(BTreeSet<FunctionId>),Open(BTreeSet<FunctionId>),}"
    );
    let normalized_ir = normalized(IR_SOURCE);
    for compatibility_escape_hatch in [
        "implDerefforFunctionTargetKnowledge",
        "implIntoIteratorforFunctionTargetKnowledge",
        "implAsRefforFunctionTargetKnowledge",
    ] {
        assert!(!normalized_ir.contains(compatibility_escape_hatch));
    }

    let merge = normalized(bounded(
        LOWERING_SOURCE,
        "    fn merge_value_infos(",
        "    fn record_return_expression(",
    ));
    let target_join = merge
        .find("left.function_targets.join(right.function_targets)")
        .expect("value joins must merge target knowledge");
    let shape_join = merge
        .find("self.merge_heap_shapes")
        .expect("value joins must merge heap shapes");
    assert!(target_join < shape_join);

    let provenance = normalized(bounded(
        INVOCATION_PROVENANCE_SOURCE,
        "    fn classify(",
        "impl<'a> From<&'a ValueInfo> for InvocationTargetProvenance<'a> {",
    ));
    assert!(provenance.contains("function_targets.exact_targets()"));
    assert!(!provenance.contains("heap_shape"));

    let call_candidate_preflight = normalized(bounded(
        CALL_CANDIDATE_SOURCE,
        "    fn preflight_dynamic_source_call_candidates(",
        "    pub(super) fn preflight_function_prototype_call_dynamic_source(",
    ));
    assert!(call_candidate_preflight.contains("function_targets.known_targets()"));

    let call_candidate_analysis = normalized(bounded(
        CALL_CANDIDATE_SOURCE,
        "    pub(super) fn analyze_known_call_candidates(",
        "    pub(super) fn analyze_known_construct_candidates(",
    ));
    assert!(call_candidate_analysis.contains(
        "ifcallee.function_targets.exact_targets().is_none()||has_unaccounted_candidate{"
    ));
}

#[test]
fn construct_candidates_use_only_the_evaluated_callees_common_prototype() {
    let construct_analysis = normalized(bounded(
        CALL_CANDIDATE_SOURCE,
        "    pub(super) fn analyze_known_construct_candidates(",
        "    fn merge_call_candidate_result(",
    ));
    let prototype_read = construct_analysis
        .find("callee.heap_shape.as_deref().and_then(|shape|read_heap_shape_property(shape,\"prototype\"))")
        .expect("construct candidates must read the evaluated callee prototype");
    let prototype_install = construct_analysis
        .find("Self::with_instance_prototype(self.function_construct_instance_info(&signature),common_instance_prototype.clone(),)")
        .expect("construct candidates must replace definition-time prototype facts");
    let this_observation = construct_analysis
        .find("self.merge_function_this_info(function_id,constructed_this.clone())")
        .expect("the refreshed constructed this value must feed source-function analysis");
    assert!(prototype_read < prototype_install);
    assert!(prototype_install < this_observation);
}

#[test]
fn contract_and_t13_record_direct_eval_call_site_authority() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "private, non-`Clone`, non-`Copy` `DirectEvalCallSite`",
        "intrinsic `%eval%` identity and direct global-reference syntax",
        "caller-environment classification",
        "evaluation-order-preserving direct-eval closure",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T13 marker: {marker}");
    }
}
