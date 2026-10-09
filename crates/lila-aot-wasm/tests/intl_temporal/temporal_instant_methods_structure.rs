const CATALOG: &str = include_str!("../../../lila-ir/src/builtins/catalog.rs");
const PLAN: &str = include_str!("../../src/planning.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect(start)
        .1
        .split_once(end)
        .expect(end)
        .0
}

#[test]
fn instant_catalog_and_planning_retain_user_code_and_duration_dependencies() {
    for name in ["Add", "Subtract", "Round", "Until", "Since"] {
        let row = bounded(
            CATALOG,
            &format!("    TemporalInstantPrototype{name} {{"),
            "\n    }",
        );
        assert!(row.contains("flags: [SYNCHRONOUS_USER_CODE]"));
        assert!(PLAN.contains(&format!(
            "StandardBuiltinId::TemporalInstantPrototype{name},"
        )));
    }
    let family = bounded(
        PLAN,
        "// The whole `Temporal.Instant` family",
        "StandardBuiltinId::TemporalZonedDateTimeConstructor",
    );
    assert!(family
        .contains("self.require_standard_builtin(StandardBuiltinId::TemporalDurationConstructor)"));
}
