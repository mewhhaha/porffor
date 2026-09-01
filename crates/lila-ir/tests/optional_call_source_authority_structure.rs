use std::fs;
use std::path::Path;

const AUTHORITY_SOURCE: &str = include_str!("../src/lowering/dynamic_source.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const CALL_EXPRESSION_SOURCE: &str =
    include_str!("../src/lowering/call_expression/non_property_call.rs");
const CALL_CANDIDATE_SOURCE: &str = include_str!("../src/lowering/call_candidate_analysis.rs");
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
fn optional_call_source_is_the_exact_private_no_capability_domain() {
    let exact_declaration = r#"    pub(super) fn into_result_info(self) -> ValueInfo {
        self.result
    }
}

pub(super) enum OptionalCallSource<'a> {
    AlreadyAccounted,
    Syntax(&'a [Expression]),
}

pub(super) fn already_accounted_optional_calls"#;
    assert_eq!(AUTHORITY_SOURCE.matches(exact_declaration).count(), 1);
    assert!(!AUTHORITY_SOURCE.contains("impl OptionalCallSource"));
    assert!(!AUTHORITY_SOURCE.contains("for OptionalCallSource"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "OptionalCallSource"),
        12,
        "the private optional-call source domain must have no duplicate transport paths"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "OptionalCallSource::AlreadyAccounted"),
        3
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "OptionalCallSource::Syntax"),
        3
    );
}

#[test]
fn optional_call_sources_have_the_exact_three_construction_routes() {
    let prefix_producer = normalized(bounded(
        AUTHORITY_SOURCE,
        "pub(super) fn already_accounted_optional_calls<'a>(",
        "impl DynamicSourceProof",
    ));
    assert_eq!(
        prefix_producer,
        normalized(
            r#"
    chain: &[OptionalChainOperationIr],
) -> Vec<OptionalCallSource<'a>> {
    chain
        .iter()
        .filter(|operation| matches!(operation, OptionalChainOperationIr::Call { .. }))
        .map(|_| OptionalCallSource::AlreadyAccounted)
        .collect()
}

"#
        )
    );

    assert_eq!(
        LOWERING_SOURCE
            .matches("&OptionalCallSource::Syntax(source_args),")
            .count(),
        1
    );
    assert_eq!(
        CALL_EXPRESSION_SOURCE
            .matches("&OptionalCallSource::Syntax(source_args),")
            .count(),
        1
    );
    assert_eq!(
        count_in_rust_sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            "&OptionalCallSource::Syntax(source_args),",
        ),
        2
    );

    let syntax_call_arm = normalized(bounded(
        LOWERING_SOURCE,
        "                OptionalOperationKind::Call { args } => {",
        "                OptionalOperationKind::PrivatePropertyAccess { field } => {",
    ));
    assert_eq!(
        syntax_call_arm,
        normalized(
            r#"
                    let source_args = args;
                    let receiver = std::mem::replace(
                        &mut first_call_receiver,
                        OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    );
                    let mut call_receiver =
                        self.take_optional_chain_call_receiver(&mut analysis, receiver);
                    let lowered_args = self.lower_call_args_expanding_spread(source_args);
                    let args = match call_receiver.as_mut() {
                        Some(receiver) => lowered_args.into_arguments_after_value(receiver),
                        None => lowered_args.into_arguments_without_predecessor(),
                    };
                    let shorted = operation.shorted();
                    let boundary_before = std::mem::take(&mut boundary_before_first_call);
                    self.analyze_optional_chain_call(
                        &mut analysis,
                        call_receiver.as_ref(),
                        &args,
                        shorted,
                        boundary_before,
                        &OptionalCallSource::Syntax(source_args),
                    );
                    chain.push(OptionalChainOperationIr::Call {
                        args,
                        receiver,
                        shorted,
                        boundary_before,
                    });
                }
"#
        )
    );

    let grouped_chain_call_arm = normalized(bounded(
        CALL_EXPRESSION_SOURCE,
        "            TypedExpr {\n                expr: ExprIr::OptionalPropertyChain { target, mut chain },\n                ..\n            } => {",
        "            callee => callee,",
    ));
    assert_eq!(
        grouped_chain_call_arm,
        normalized(
            r#"
                let source_args = args;
                let call_sources = already_accounted_optional_calls(&chain);
                let mut analysis =
                    self.analyze_optional_property_chain(target.as_ref(), &chain, &call_sources);
                let mut receiver = self.take_optional_chain_call_receiver(
                    &mut analysis,
                    OptionalChainCallReceiverIr::ReferenceOrUndefined,
                );
                let lowered_args = self.lower_call_args_expanding_spread(args);
                let args = match receiver.as_mut() {
                    Some(receiver) => lowered_args.into_arguments_after_value(receiver),
                    None => lowered_args.into_arguments_without_predecessor(),
                };
                self.analyze_optional_chain_call(
                    &mut analysis,
                    receiver.as_ref(),
                    &args,
                    false,
                    true,
                    &OptionalCallSource::Syntax(source_args),
                );
                chain.push(OptionalChainOperationIr::Call {
                    args,
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: false,
                    boundary_before: true,
                });
                let (result, effects) = self.finish_optional_chain_analysis(analysis);
                let chain =
                    TypedExpr::from_info(result, ExprIr::OptionalPropertyChain { target, chain });
                return effects.attach_to_emitted_call(chain);
            }
"#
        )
    );
}

#[test]
fn optional_chain_analysis_borrows_each_authority_exactly_once() {
    let analysis = normalized(bounded(
        LOWERING_SOURCE,
        "    fn analyze_optional_property_chain(",
        "    fn optional_chain_property_analysis(",
    ));
    for exact_transport in [
        "call_sources:&[OptionalCallSource<'_>],",
        "letmutcall_sources=call_sources.iter();",
        "letsource=call_sources.next().expect(\"missingoptionalcallsource\");",
        "self.analyze_optional_chain_call(&mutanalysis,call_receiver.as_ref(),args,*shorted,*boundary_before,source,);",
        "let(next,effects)=self.optional_call_info(&analysis.current,receiver,args,source);",
        "analysis.invocation_effects=previous.combine(effects);",
        "assert!(call_sources.next().is_none(),\"extraoptionalcallsource\");",
    ] {
        assert!(
            analysis.contains(exact_transport),
            "missing exact borrowed transport `{exact_transport}`"
        );
    }
    assert!(!analysis.contains("call_sources.iter().copied()"));
    assert!(!analysis.contains("call_sources.iter().cloned()"));
    assert_eq!(analysis.matches("call_sources.next()").count(), 2);
    assert_eq!(
        analysis
            .matches("self.analyze_optional_chain_call(")
            .count(),
        1
    );
    assert_eq!(analysis.matches("self.optional_call_info(").count(), 1);
    let iterator_creation = analysis
        .find("letmutcall_sources=call_sources.iter();")
        .expect("optional-call authority iterator");
    let per_call_authority = analysis
        .find("letsource=call_sources.next().expect(\"missingoptionalcallsource\");")
        .expect("per-call authority read");
    let consumer = analysis
        .find("self.analyze_optional_chain_call(&mutanalysis,call_receiver.as_ref(),args,*shorted,*boundary_before,source,);")
        .expect("optional-call authority consumer");
    let excess_check = analysis
        .find("assert!(call_sources.next().is_none(),\"extraoptionalcallsource\");")
        .expect("optional-call excess-authority check");
    assert!(iterator_creation < per_call_authority);
    assert!(per_call_authority < consumer);
    assert!(consumer < excess_check);
}

#[test]
fn optional_call_analysis_exhaustively_couples_source_proof_and_diagnostic_ownership() {
    let consumer = normalized(bounded(
        LOWERING_SOURCE,
        "    fn optional_call_info(",
        "    fn lower_optional_chain_property_key(",
    ));
    assert!(consumer
        .contains("OptionalCallSource::AlreadyAccounted=>CallCandidateSource::AlreadyAccounted"));
    assert!(consumer.contains(
        "OptionalCallSource::Syntax(arguments)=>CallCandidateSource::IndirectSyntax(arguments)"
    ));
    assert!(!consumer.contains("_=>"));
    assert!(consumer.contains("self.analyze_known_call_candidates(callee,receiver,args,source)"));

    let candidate_preflight = normalized(bounded(
        CALL_CANDIDATE_SOURCE,
        "    fn preflight_dynamic_source_call_candidates(",
        "    pub(super) fn preflight_function_prototype_call_dynamic_source(",
    ));
    let already_accounted = candidate_preflight
        .find("ifmatches!(source,CallCandidateSource::AlreadyAccounted)")
        .expect("already-accounted optional calls must have an explicit diagnostic branch");
    let discard = candidate_preflight[already_accounted..]
        .find("ValueInfo::undefined()")
        .expect("already-accounted unsupported calls must preserve the prior result placeholder");
    let recorder = candidate_preflight[already_accounted..]
        .find("self.record_unsupported_dynamic_source(unsupported)")
        .expect("syntax-owned unsupported calls must record their diagnostic");
    assert!(discard < recorder);

    for durable_evidence in [
        "private, capability-free `OptionalCallSource`",
        "source-proof availability and diagnostic ownership",
        "Invocation-effect tokens from every optional call",
    ] {
        let durable_evidence = normalized(durable_evidence);
        assert!(
            normalized(CONTRACT).contains(&durable_evidence)
                || normalized(TASK).contains(&durable_evidence),
            "missing durable evidence `{durable_evidence}`"
        );
    }
}
