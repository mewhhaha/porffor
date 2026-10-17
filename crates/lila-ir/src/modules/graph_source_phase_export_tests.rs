use super::*;

fn source_graph(distinct: bool, second_namespace: bool) -> ModuleGraphSources {
    let mut sources = sources_of(&[
        (
            "/root/entry.js",
            "import { value } from './star.js'; value;",
        ),
        (
            "/root/star.js",
            "export * from './one.js'; export * from './two.js';",
        ),
        (
            "/root/one.js",
            "import source value from 'host-source'; export { value };",
        ),
        (
            "/root/two.js",
            if second_namespace {
                "import * as value from 'host-source'; export { value };"
            } else {
                "import source value from 'host-source'; export { value };"
            },
        ),
    ]);
    sources.modules.push(ModuleSourceIr::module_source(
        ModuleKey::from_host("native/first"),
        "源🌐".into(),
        "lila://native/first".into(),
    ));
    sources
        .resolutions
        .push((2, ModuleRequestKeyIr::plain("host-source"), 4));
    if distinct {
        sources.modules.push(ModuleSourceIr::module_source(
            ModuleKey::from_host("native/second"),
            "源🌐".into(),
            "lila://native/second".into(),
        ));
    }
    sources.resolutions.push((
        3,
        ModuleRequestKeyIr::plain("host-source"),
        if distinct { 5 } else { 4 },
    ));
    sources
}

#[test]
fn source_reexports_through_star_paths_resolve_the_actual_shared_host_record() {
    let sources = source_graph(false, false);
    let mut graph = build_graph(&sources).unwrap();
    link(&mut graph);
    assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
    let expected = ResolvedBindingIr::Resolved {
        module: 4,
        binding: ModuleBindingNameIr::ModuleSource,
    };
    assert_eq!(graph.resolve_export(1, &ExportName::new("value")), expected);
    assert_eq!(graph.units[0].resolved_imports, vec![expected]);
    assert_eq!(
        graph.evaluation_mode(4),
        ModuleEvaluationModeIr::NotEvaluated
    );
    assert!(graph.exported_names(4).is_empty());
    let program = crate::lower_module_graph(&sources);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

#[test]
fn equal_source_tags_do_not_merge_distinct_host_record_identities() {
    let mut graph = build_graph(&source_graph(true, false)).unwrap();
    link(&mut graph);
    assert_eq!(
        graph.resolve_export(1, &ExportName::new("value")),
        ResolvedBindingIr::Ambiguous
    );
    assert!(graph
        .link_errors
        .iter()
        .any(|error| matches!(error, ModuleLinkErrorIr::AmbiguousExport { .. })));
}

#[test]
fn source_and_namespace_bindings_from_one_record_remain_ambiguous() {
    let mut graph = build_graph(&source_graph(false, true)).unwrap();
    link(&mut graph);
    assert_eq!(
        graph.resolve_export(1, &ExportName::new("value")),
        ResolvedBindingIr::Ambiguous
    );
}

#[test]
fn host_source_payload_is_identity_data_and_never_javascript_syntax() {
    let mut sources = source_graph(false, false);
    let source = &sources.modules[4];
    assert_eq!(source.kind(), ModuleKindIr::ModuleSource);
    assert_eq!(source.source_text(), "");
    assert_eq!(source.payload_bytes(), "源🌐".as_bytes());
    assert!(source.module_requests().unwrap().is_empty());
    sources.modules.push(ModuleSourceIr::module_source(
        ModuleKey::from_host("native/first"),
        "changed".into(),
        "lila://native/first".into(),
    ));
    let graph = build_graph(&sources).unwrap();
    assert!(graph
        .link_errors
        .iter()
        .any(|error| matches!(error, ModuleLinkErrorIr::InconsistentLoad { .. })));
}

#[test]
fn ordinary_source_text_records_keep_their_source_unavailable_rejection() {
    let mut sources = source_graph(false, false);
    sources.modules[4] = ModuleSourceIr::new(
        ModuleKey::from_host("native/first"),
        "export default 7;".into(),
        "lila://native/first".into(),
    );
    let mut graph = build_graph(&sources).unwrap();
    link(&mut graph);
    assert!(
        graph
            .link_errors
            .iter()
            .all(|error| matches!(error, ModuleLinkErrorIr::SourceUnavailable { .. })),
        "{:?}",
        graph.link_errors
    );
    assert!(!graph.link_errors.is_empty());
}

#[test]
fn dynamic_source_jobs_retain_native_source_cells_without_evaluation() {
    let mut sources = source_graph(false, false);
    sources.modules[0] = ModuleSourceIr::new(
        ModuleKey::from_host("/root/entry.js"),
        "import.source('host-source');".into(),
        "lila://entry".into(),
    );
    sources.resolutions.retain(|(owner, _, _)| *owner != 0);
    sources
        .resolutions
        .push((0, ModuleRequestKeyIr::plain("host-source"), 4));
    let program = crate::lower_module_graph(&sources);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.as_ref().unwrap();
    let key = ModuleKey::from_host("native/first");
    let target = *graph
        .keys
        .get(&crate::ModuleRecordKey::new(
            key.clone(),
            ModuleKindIr::ModuleSource,
        ))
        .expect("the actual host source record is keyed by its ModuleSource kind");
    assert!(!graph
        .keys
        .contains_key(&crate::ModuleRecordKey::new(key, ModuleKindIr::SourceText,)));
    assert_eq!(
        graph.evaluation_mode(target),
        ModuleEvaluationModeIr::NotEvaluated
    );
    assert!(graph.dynamic_rejections.is_empty());
    let dispatcher = graph.module_initialization_dispatchers(false);
    assert!(dispatcher.contains(MergedName::minted(target, UnitCellRole::ModuleSource).as_str()));
    assert!(!dispatcher.contains("no source representation"));
}
