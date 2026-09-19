use lila_ir::{
    lower_module_graph, EarlyErrorCode, IrDiagnosticPhase, ModuleGraphSources, ModuleKey,
    ModuleSourceIr, NativeErrorKind,
};

fn sources(files: &[(&str, &str)]) -> ModuleGraphSources {
    let modules = files
        .iter()
        .map(|(name, source)| {
            ModuleSourceIr::new(
                ModuleKey::from_host(*name),
                (*source).into(),
                format!("file:///{name}"),
            )
        })
        .collect::<Vec<_>>();
    let mut resolutions = Vec::new();
    for (referrer, module) in modules.iter().enumerate() {
        for request in module.module_requests().unwrap_or_default() {
            if let Some(target) = files
                .iter()
                .position(|(name, _)| *name == request.specifier().trim_start_matches("./"))
            {
                resolutions.push((referrer as u32, request, target as u32));
            }
        }
    }
    ModuleGraphSources {
        modules,
        entry: 0,
        resolutions,
    }
}

#[test]
fn dynamic_only_invalid_closures_do_not_reject_the_entry() {
    let program = lower_module_graph(&sources(&[
        ("entry.js", "import('./malformed.js'); import('./missing.js'); import('./export.js'); import('./valid.js');"),
        ("malformed.js", "invalid syntax!"),
        ("missing.js", "import defer * as ns from './absent.js';"),
        ("export.js", "import { missing } from './shared.js';"),
        ("valid.js", "export { value } from './shared.js';"),
        ("shared.js", "export const value = 7;"),
    ]));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.expect("module graph retained");
    assert_eq!(
        graph.units.len(),
        3,
        "only entry and valid static closure own activations"
    );
    assert!(graph.keys.contains_key(&ModuleKey::from_host("shared.js")));
    assert!(graph.link_errors.is_empty());
}

#[test]
fn a_static_path_to_the_same_invalid_target_rejects_before_evaluation() {
    let program = lower_module_graph(&sources(&[
        ("entry.js", "import('./invalid.js'); import defer * as ns from './invalid.js'; print('unreachable');"),
        ("invalid.js", "invalid syntax!"),
    ]));
    assert!(program.script.is_none());
    assert_eq!(program.diagnostics.len(), 1);
    let diagnostic = &program.diagnostics[0];
    assert_eq!(diagnostic.code(), Some(EarlyErrorCode::ModuleSyntax));
    assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Resolution);
    assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
}

#[test]
fn malformed_dynamic_target_does_not_admit_the_tla_driver() {
    let program = lower_module_graph(&sources(&[
        ("entry.js", "await 0; import('./invalid.js');"),
        ("invalid.js", "invalid syntax!"),
    ]));
    assert!(!program.is_wasm_supported());
    assert!(program.script.is_none());
    assert_eq!(
        program.diagnostics[0].code(),
        Some(EarlyErrorCode::ModuleSyntax)
    );
}

#[test]
fn dynamic_rejection_partition_does_not_hide_contradictory_host_sources() {
    let mut graph = sources(&[
        ("entry.js", "import('./invalid.js'); import('./shared.js');"),
        ("invalid.js", "invalid syntax!"),
        ("shared.js", "export const value = 1;"),
    ]);
    graph.modules.push(ModuleSourceIr::new(
        ModuleKey::from_host("shared.js"),
        "export const value = 2;".into(),
        "file:///shared.js".into(),
    ));
    let program = lower_module_graph(&graph);
    assert!(!program.is_wasm_supported());
    assert!(program.script.is_none());
}
