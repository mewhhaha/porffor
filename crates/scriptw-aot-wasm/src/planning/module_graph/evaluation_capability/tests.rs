use super::*;
use scriptw_ir::{ModuleGraphSources, ModuleKey, ModuleRequestKeyIr, ModuleSourceIr};

fn plan(entry: &str, wrapper: &str) -> ModuleGraphPlan {
    let sources = ModuleGraphSources {
        entry: 0,
        modules: vec![
            ModuleSourceIr::new(
                ModuleKey::from_host("entry"),
                entry.into(),
                "scriptw://entry".into(),
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("wrapper"),
                wrapper.into(),
                "scriptw://wrapper".into(),
            ),
            ModuleSourceIr::module_source(
                ModuleKey::from_host("host"),
                "ModuleSource".into(),
                "scriptw://host".into(),
            ),
        ],
        resolutions: vec![
            (0, ModuleRequestKeyIr::plain("wrapper"), 1),
            (0, ModuleRequestKeyIr::plain("host"), 2),
            (1, ModuleRequestKeyIr::plain("entry"), 0),
            (1, ModuleRequestKeyIr::plain("host"), 2),
        ],
        realm_requests: Default::default(),
    };
    let program = scriptw_ir::lower_module_graph(&sources);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    ModuleGraphPlan::from_script(&program.script.unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn source_phase_never_grants_host_evaluation_authority() {
    let plan = plan("import source source from 'host'; source;", "");
    assert_eq!(plan.unavailable_evaluation_capability(0), None);
    assert_eq!(
        plan.unavailable_evaluation_capability(2),
        Some(RuntimeUnavailableCapability::HostModuleEvaluation)
    );
}

#[test]
fn evaluation_closure_preserves_transitive_host_capability_requirements() {
    let plan = plan("import 'wrapper';", "import * as source from 'host';");
    for module in [0, 1, 2] {
        assert_eq!(
            plan.unavailable_evaluation_capability(module),
            Some(RuntimeUnavailableCapability::HostModuleEvaluation)
        );
    }
}

#[test]
fn dynamic_and_defer_edges_do_not_become_eager_evaluation_requirements() {
    for entry in [
        "if (false) import('host');",
        "import defer * as source from 'host';",
    ] {
        let plan = plan(entry, "");
        assert_eq!(plan.unavailable_evaluation_capability(0), None);
    }
}

#[test]
fn executable_cycles_terminate_without_opening_source_edges() {
    let plan = plan(
        "import 'wrapper';",
        "import 'entry'; import source source from 'host';",
    );
    assert_eq!(plan.unavailable_evaluation_capability(0), None);
    assert_eq!(plan.unavailable_evaluation_capability(1), None);
}
