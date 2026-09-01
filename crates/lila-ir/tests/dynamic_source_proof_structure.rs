const SOURCE: &str = include_str!("../src/lowering/dynamic_source.rs");
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

#[test]
fn dynamic_source_proof_is_the_exact_private_no_capability_domain() {
    let declaration = normalized(bounded(
        SOURCE,
        "/// Proof that the dynamic source text is fixed by syntax before lowering.",
        "/// The exhaustive result of resolving a call to a dynamic-source identity.",
    ));
    assert_eq!(
        declaration,
        normalized(
            r#"
///
/// Its constructors are private to this module. A folded `ExprIr::String`
/// therefore cannot be promoted to AOT-known source by a downstream caller.
enum DynamicSourceProof {
    Runtime,
    AotSyntax,
}
"#
        )
    );
    assert_eq!(SOURCE.matches("DynamicSourceProof").count(), 7);
    assert!(!SOURCE.contains("impl Clone for DynamicSourceProof"));
    assert!(!SOURCE.contains("impl Copy for DynamicSourceProof"));
    assert!(!SOURCE.contains("impl Debug for DynamicSourceProof"));
    assert!(!SOURCE.contains("impl PartialEq for DynamicSourceProof"));
    assert!(!SOURCE.contains("impl Eq for DynamicSourceProof"));
}

#[test]
fn source_syntax_produces_one_proof_consumed_by_one_exhaustive_gap_projection() {
    let producer = normalized(bounded(
        SOURCE,
        "impl DynamicSourceProof {",
        "/// Recognizes only syntax whose evaluation is already the primitive string",
    ));
    assert_eq!(
        producer,
        normalized(
            r#"
    fn from_expression(expression: &Expression) -> Self {
        if has_aot_source_text_proof(expression) {
            Self::AotSyntax
        } else {
            Self::Runtime
        }
    }

    fn for_args(kind: DynamicSourceKind, args: &[Expression]) -> Self {
        match kind {
            DynamicSourceKind::DirectEval
            | DynamicSourceKind::IndirectEval
            | DynamicSourceKind::RealmEvalScript => args
                .first()
                .map_or(Self::Runtime, |source| Self::from_expression(source)),
            DynamicSourceKind::Function(
                DynamicFunctionKind::Ordinary
                | DynamicFunctionKind::Generator
                | DynamicFunctionKind::Async
                | DynamicFunctionKind::AsyncGenerator,
            ) => {
                if args.iter().all(has_aot_source_text_proof) {
                    Self::AotSyntax
                } else {
                    Self::Runtime
                }
            }
        }
    }
}
"#
        )
    );

    let consumer = normalized(bounded(
        SOURCE,
        "const fn gap_for_source_proof(",
        "impl ScriptLowerer<'_> {",
    ));
    assert_eq!(
        consumer,
        normalized(
            r#"
    kind: DynamicSourceKind,
    proof: DynamicSourceProof,
) -> DynamicSourceGap {
    match proof {
        DynamicSourceProof::Runtime => DynamicSourceGap::runtime_source(kind),
        DynamicSourceProof::AotSyntax => DynamicSourceGap::aot_known_source(kind),
    }
}
"#
        )
    );
    assert!(!consumer.contains("_=>"));

    let resolution = normalized(bounded(
        SOURCE,
        "        let proof = source_args",
        "    /// Unknown user code can erase the global `%eval%` value fact without",
    ));
    assert_eq!(
        resolution,
        normalized(
            r#"
            .map(|args| DynamicSourceProof::for_args(kind, args))
            .unwrap_or(DynamicSourceProof::Runtime);
        Some(ResolvedDynamicSourceCall::Unsupported(
            UnsupportedDynamicSourceCall {
                standard_builtin: StandardBuiltinId::from_function_id(function_id),
                gap: gap_for_source_proof(kind, proof),
            },
        ))
    }

"#
        )
    );
    assert_eq!(resolution.matches("proof").count(), 2);
    for forbidden in [
        "proof.clone()",
        "&proof",
        "matchproof",
        "matches!(proof",
        "proof==",
        "proof!=",
    ] {
        assert!(!resolution.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_t13_record_the_one_shot_source_proof() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "non-`Clone`, non-`Copy`",
        "seven lexical type mentions",
        "sole exhaustive gap projection",
        "source-equivalent capability closure",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T13 marker: {marker}");
    }
}
