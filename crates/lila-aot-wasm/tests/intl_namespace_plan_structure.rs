const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const INTL_NAMESPACE_SOURCE: &str = include_str!("../src/planning/intl_namespace.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn intl_namespace_plan_has_one_private_child_owner() {
    assert_eq!(
        PLANNING_SOURCE.matches("\nmod intl_namespace;\n").count(),
        1
    );
    assert!(!PLANNING_SOURCE.contains("\npub mod intl_namespace;\n"));
    assert!(!PLANNING_SOURCE.contains("\npub(crate) mod intl_namespace;\n"));
    assert!(!PLANNING_SOURCE.contains("\nmod intl_namespace {\n"));
    assert!(INTL_NAMESPACE_SOURCE.starts_with("use super::*;\n\n"));

    for declaration in [
        "pub(crate) struct IntlRootsSeeded(());",
        "pub(crate) enum IntlNamespacePlan {",
        "pub(crate) struct IntlNamespaceMembers {",
    ] {
        assert_eq!(
            INTL_NAMESPACE_SOURCE.matches(declaration).count(),
            1,
            "child must own exactly one `{declaration}`"
        );
        assert!(
            !PLANNING_SOURCE.contains(declaration),
            "parent must not retain `{declaration}`"
        );
    }
    assert_eq!(
        INTL_NAMESPACE_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| {
                line.starts_with("pub(crate) struct ") || line.starts_with("pub(crate) enum ")
            })
            .count(),
        3
    );
    assert!(!INTL_NAMESPACE_SOURCE.contains("pub(super) "));
    assert!(!INTL_NAMESPACE_SOURCE.contains("\npub struct "));
    assert!(!INTL_NAMESPACE_SOURCE.contains("\npub enum "));

    for definition in [
        "    pub(crate) fn rooted(",
        "    pub(crate) fn members(",
        "    pub(crate) fn in_installation_order(",
    ] {
        assert_eq!(
            INTL_NAMESPACE_SOURCE.matches(definition).count(),
            1,
            "child must own exactly one `{definition}`"
        );
        assert!(
            !PLANNING_SOURCE.contains(definition),
            "parent must not retain `{definition}`"
        );
    }
    assert_eq!(INTL_NAMESPACE_SOURCE.matches("fn ").count(), 3);
    assert_eq!(
        INTL_NAMESPACE_SOURCE
            .lines()
            .filter(|line| line.starts_with("    pub(crate) fn "))
            .count(),
        3
    );

    assert!(!INTL_NAMESPACE_SOURCE.contains("IntlRootsSeeded(pub"));
    assert_eq!(
        INTL_NAMESPACE_SOURCE
            .lines()
            .filter(|line| line.trim_start().starts_with("members: "))
            .count(),
        2,
        "one private member-list field and one private construction"
    );
    assert!(!INTL_NAMESPACE_SOURCE.contains("pub(crate) members:"));
    assert!(!INTL_NAMESPACE_SOURCE.contains("pub(super) members:"));

    let plan_domain = bounded(
        INTL_NAMESPACE_SOURCE,
        "pub(crate) enum IntlNamespacePlan {",
        "impl IntlNamespacePlan {",
    );
    let variants = plan_domain
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty() && !line.starts_with("///") && !line.starts_with("#[") && *line != "}"
        })
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "Absent,",
            "RootedWithDateTimeFormatFamily(IntlRootsSeeded),"
        ]
    );
    assert!(plan_domain.contains("#[default]\n    Absent,"));
}

#[test]
fn intl_namespace_roots_and_policy_remain_parent_owned() {
    assert_eq!(
        PLANNING_SOURCE
            .matches("const INTL_NAMESPACE_ROOTS: [StandardBuiltinId; 15] = [")
            .count(),
        1
    );
    assert!(!INTL_NAMESPACE_SOURCE.contains("const INTL_NAMESPACE_ROOTS"));
    assert_eq!(
        PLANNING_SOURCE
            .matches("pub(crate) use intl_namespace::{IntlNamespaceMembers, IntlNamespacePlan};")
            .count(),
        1
    );
    assert!(!PLANNING_SOURCE.contains("IntlRootsSeeded,"));

    let roots_and_proof = bounded(
        PLANNING_SOURCE,
        "const INTL_NAMESPACE_ROOTS: [StandardBuiltinId; 15] = [",
        "pub(crate) use intl_namespace::{IntlNamespaceMembers, IntlNamespacePlan};",
    );
    for proof in [
        "while member < INTL_NAMESPACE_CONSTRUCTORS.len()",
        "while root < INTL_NAMESPACE_ROOTS.len()",
        "if INTL_NAMESPACE_ROOTS[root] as u32 == needle",
        "assert!(\n            found,",
    ] {
        assert_eq!(
            roots_and_proof.matches(proof).count(),
            1,
            "parent must retain containment proof `{proof}`"
        );
    }

    for parent_boundary in [
        "intl: IntlNamespacePlan,",
        "self.intl.members(self.full_standard_globals)",
        "self.intl = IntlNamespacePlan::rooted(&mut self.standard_roots);",
    ] {
        assert_eq!(
            PLANNING_SOURCE.matches(parent_boundary).count(),
            1,
            "parent must retain exactly one `{parent_boundary}`"
        );
        assert!(!INTL_NAMESPACE_SOURCE.contains(parent_boundary));
    }
}

#[test]
fn intl_bootstrap_consumes_only_the_rooted_member_witness() {
    let installer = bounded(
        BOOTSTRAP_SOURCE,
        "    pub(crate) fn init_intl_object(",
        "    pub(crate) fn init_math_object(",
    );
    assert_eq!(
        installer.matches("members: IntlNamespaceMembers,").count(),
        1
    );
    assert_eq!(
        installer.matches("members.in_installation_order()").count(),
        1
    );
    assert!(!installer.contains("IntlNamespacePlan"));
    assert!(!installer.contains("IntlRootsSeeded"));
    assert!(!installer.contains(".should_initialize_standard_builtin("));

    assert_eq!(
        INTL_NAMESPACE_SOURCE
            .matches("pub(crate) fn in_installation_order(")
            .count()
            + BOOTSTRAP_SOURCE
                .matches("members.in_installation_order()")
                .count(),
        2,
        "one witness iterator definition and one bootstrap consumer close the call map"
    );
}
