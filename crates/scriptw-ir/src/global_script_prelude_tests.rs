use super::*;
use scriptw_front::{parse, ParseOptions, ParsedSource};

fn prepared(source: &str, prelude: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).unwrap();
    let ParsedSource::Script(prelude) = parse(prelude, ParseOptions::script()).unwrap() else {
        unreachable!()
    };
    LoweringSession::default().lower_source_with_prelude(
        &source,
        &prelude,
        HostSurfacePolicy::default(),
    )
}

#[test]
fn prelude_and_script_entry_keep_separate_strictness_and_runtime_declarations() {
    let program = prepared(
        "'use strict'; let testLexical = 2; var entryVar = 3; entryVar;",
        "with ({}) {} var harnessVar = 1;",
    );
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let prelude = script.module_prelude.as_ref().unwrap();
    let entry = script.script_entry_after_prelude.as_ref().unwrap().unit();
    assert!(!prelude.strict);
    assert!(entry.strict);
    assert_eq!(prelude.kind, PreparedScriptKind::RealmScript);
    assert_eq!(entry.kind, PreparedScriptKind::RealmScript);
    assert!(prelude.has_global_variable_environment());
    assert!(entry.has_global_variable_environment());
    assert_ne!(prelude.id, entry.id);
    assert!(prelude
        .declarations
        .var_names
        .iter()
        .any(|name| name == "harnessVar"));
    assert!(entry
        .declarations
        .var_names
        .iter()
        .any(|name| name == "entryVar"));
    assert!(entry
        .declarations
        .lexical_names_in_source_order
        .iter()
        .any(|name| name == "testLexical"));
    assert!(script.runtime_declarations.var_names.is_empty());
    assert_eq!(script.global_bindings.lexical_names().len(), 0);
    assert!(script.body.statements.is_empty());
    assert_eq!(
        script
            .prepared_script_units()
            .filter(|unit| unit.id == entry.id || unit.id == prelude.id)
            .count(),
        2
    );
}

#[test]
fn independent_script_entry_cannot_capture_harness_lexical_slots() {
    let program = prepared(
        "'use strict'; harnessLexical + harnessVar;",
        "let harnessLexical = 4; var harnessVar = 5;",
    );
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let prelude = script.module_prelude.as_ref().unwrap();
    let entry = script.script_entry_after_prelude.as_ref().unwrap().unit();
    assert!(prelude
        .declarations
        .lexical_names_in_source_order
        .iter()
        .any(|name| name == "harnessLexical"));
    assert!(entry.declarations.lexical_names_in_source_order.is_empty());
    assert!(entry.declarations.var_names.is_empty());
    assert!(entry
        .owned_env_bindings
        .iter()
        .all(|binding| binding.name != "harnessLexical" && binding.name != "harnessVar"));
}

#[test]
fn global_prelude_diagnostic_origin_does_not_change_its_typed_payload() {
    let entry = IrDiagnostic::unsupported_dynamic_source(DynamicSourceGap::aot_known_source(
        DynamicSourceKind::ShadowRealmEvaluate,
    ));
    let prelude = entry.clone().in_global_prelude();
    assert!(!entry.is_global_prelude_failure());
    assert!(prelude.is_global_prelude_failure());
    assert_eq!(entry.kind(), prelude.kind());
    assert_eq!(entry.phase(), prelude.phase());
    assert_eq!(entry.code(), prelude.code());
    assert_eq!(entry.message, prelude.message);
}

#[test]
fn canonical_globals_bootstrap_before_independent_source_declarations() {
    let program = prepared(
        "var entryVar = 1; let entryLexical = 2; function Function() {}",
        "var harnessVar = 3;",
    );
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    for builtin in StandardBuiltinId::all_globals() {
        let Some(name) = builtin.global_name() else {
            continue;
        };
        let binding = script.global_bindings.get(name).unwrap();
        assert_eq!(
            binding.initializer,
            GlobalPropertyInitializerIr::BuiltinFunction(*builtin)
        );
        assert_eq!(binding.declarations, GlobalDeclarationSetIr::None);
    }
    for name in [
        "globalThis",
        "Infinity",
        "NaN",
        "undefined",
        "Reflect",
        "Math",
        "JSON",
        "Atomics",
        "Temporal",
        "Intl",
    ] {
        let binding = script.global_bindings.get(name).unwrap();
        assert_eq!(binding.declarations, GlobalDeclarationSetIr::None);
        assert!(!binding.initializer.enumerable());
        let mutable = !["Infinity", "NaN", "undefined"].contains(&name);
        assert_eq!(binding.initializer.writable(), mutable);
        assert_eq!(binding.initializer.configurable(), mutable);
    }
    for name in ["entryVar", "entryLexical", "harnessVar"] {
        assert!(script.global_bindings.get(name).is_none());
    }
    assert_eq!(script.global_bindings.lexical_names().len(), 0);
    let entry = script.script_entry_after_prelude.as_ref().unwrap().unit();
    assert!(matches!(
        entry.global_bindings.get("Function").unwrap().initializer,
        GlobalPropertyInitializerIr::SourceFunction(_)
    ));
    assert!(entry
        .declarations
        .var_names
        .iter()
        .any(|name| name == "entryVar"));
    assert_eq!(
        entry.declarations.lexical_names_in_source_order,
        ["entryLexical"]
    );
}
