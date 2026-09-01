use std::fs;
use std::path::Path;

const REACHABILITY_SOURCE: &str = include_str!("../src/lowering/object_environment_logical.rs");
const ASSIGNMENT_SOURCE: &str = include_str!("../src/lowering/assignment.rs");

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
fn logical_assignment_reachability_is_the_exact_private_no_capability_domain() {
    assert_eq!(
        REACHABILITY_SOURCE
            .matches("pub(super) enum LogicalAssignmentReachability {")
            .count(),
        1
    );
    assert_eq!(
        REACHABILITY_SOURCE
            .matches(
                "}\n\npub(super) enum LogicalAssignmentReachability {\n    Definite,\n    WithEnvironmentFallback,\n}\n\nimpl<'a> ScriptLowerer<'a> {"
            )
            .count(),
        1,
        "the lowering-private domain must have exactly two rows and no attributes"
    );
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "LogicalAssignmentReachability"),
        13,
        "one declaration, one import, one typed consumer, eight exhaustive arms and two producers must own every mention"
    );
}

#[test]
fn reachability_consumer_exhaustively_owns_all_four_semantic_decisions() {
    let consumer = bounded(
        REACHABILITY_SOURCE,
        "    pub(super) fn lower_located_identifier_logical_assignment(",
        "\n    }\n}",
    );
    assert!(consumer.contains("reachability: LogicalAssignmentReachability"));
    assert_eq!(consumer.matches("match &reachability {").count(), 4);
    assert_eq!(
        consumer
            .matches("LogicalAssignmentReachability::Definite =>")
            .count(),
        4
    );
    assert_eq!(
        consumer
            .matches("LogicalAssignmentReachability::WithEnvironmentFallback =>")
            .count(),
        4
    );

    let normalized_consumer = normalized(consumer);
    for exact_decision in [
        "ifglobal_binding{match&reachability{LogicalAssignmentReachability::Definite=>{}LogicalAssignmentReachability::WithEnvironmentFallback=>{ifletSome(binding)=&binding{self.widen_binding_for_possible_replacement(&name);debug_assert_eq!(binding.mode,BindingMode::Var);}returnself.lower_global_object_environment_logical_assignment(name,op,rhs);}}}",
        "letlhs_info=match&reachability{LogicalAssignmentReachability::Definite=>ValueInfo{kind:binding.kind,possible_kinds:binding.possible_kinds,heap_shape:binding.heap_shape.clone(),function_targets:binding.function_targets.clone(),},LogicalAssignmentReachability::WithEnvironmentFallback=>{letmutvalue=ValueInfo{kind:binding.kind,possible_kinds:binding.possible_kinds,heap_shape:binding.heap_shape.clone(),function_targets:binding.function_targets.clone(),};value.widen_for_possible_replacement();value}};",
        "letresult_info=match&reachability{LogicalAssignmentReachability::Definite=>{self.merge_value_infos(lhs.value_info(),rhs.value_info())}LogicalAssignmentReachability::WithEnvironmentFallback=>{letmutvalue=self.merge_value_infos(lhs.value_info(),rhs.value_info());value.widen_for_possible_replacement();value}};",
        "letresult_info=match&reachability{LogicalAssignmentReachability::Definite=>{self.merge_value_infos(lhs.value_info(),write.value_info())}LogicalAssignmentReachability::WithEnvironmentFallback=>{letmutvalue=self.merge_value_infos(lhs.value_info(),write.value_info());value.widen_for_possible_replacement();value}};",
    ] {
        assert!(
            normalized_consumer.contains(exact_decision),
            "missing exact reachability decision `{exact_decision}`"
        );
    }
    for forbidden in [
        "reachability ==",
        "reachability !=",
        "matches!(reachability",
        "_ =>",
        "Default",
    ] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn assignment_lowering_is_the_exact_two_producer_authority() {
    let logical_arm = bounded(
        ASSIGNMENT_SOURCE,
        "            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {",
        "            AssignOp::And\n            | AssignOp::Or",
    );
    let normalized_arm = normalized(logical_arm);
    for producer in [
        "letfallback=self.lower_located_identifier_logical_assignment(name,logical_op,rhs_value.clone(),reference,LogicalAssignmentReachability::WithEnvironmentFallback,);",
        "self.lower_located_identifier_logical_assignment(name,logical_op,rhs_value,reference,LogicalAssignmentReachability::Definite,)",
    ] {
        assert_eq!(
            normalized_arm.matches(producer).count(),
            1,
            "missing exact producer `{producer}`"
        );
    }
    assert_eq!(
        ASSIGNMENT_SOURCE
            .matches("LogicalAssignmentReachability::WithEnvironmentFallback")
            .count(),
        1
    );
    assert_eq!(
        ASSIGNMENT_SOURCE
            .matches("LogicalAssignmentReachability::Definite")
            .count(),
        1
    );
    assert!(normalized_arm.contains(
        "ifletSome(objects)=selected{letplan=self.with_environment_reference_plan(name.clone(),objects);"
    ));
    assert!(
        normalized_arm.contains("returnplan.logical_assignment(logical_op,rhs_value,fallback);")
    );
    assert!(normalized_arm.contains(
        "ifreference.is_unproven_global(){returnself.lower_global_object_environment_logical_assignment(name,logical_op,rhs_value,);}"
    ));
}
