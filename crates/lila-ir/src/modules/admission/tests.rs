use super::*;
use lila_front::{parse, ParseOptions, ParsedSource};

fn script_sources(files: &[(&str, &str)], entry: usize) -> ModuleGraphSources {
    let modules = files
        .iter()
        .enumerate()
        .map(|(index, (name, source))| {
            let key = ModuleKey::from_host(*name);
            let url = format!("file:///{name}");
            if index == entry {
                let ParsedSource::Script(parsed) =
                    parse(*source, ParseOptions::script()).expect("Script entry parses")
                else {
                    panic!("Script parse options must retain Script syntax");
                };
                ModuleSourceIr::from_parsed_script(key, url, parsed)
            } else {
                ModuleSourceIr::new(key, (*source).into(), url)
            }
        })
        .collect::<Vec<_>>();
    let mut resolutions = Vec::new();
    for (referrer, source) in modules.iter().enumerate() {
        for request in source.module_requests().unwrap_or_default() {
            if let Some(target) = files.iter().enumerate().find_map(|(index, (name, _))| {
                (index != entry && *name == request.specifier().trim_start_matches("./"))
                    .then_some(index)
            }) {
                resolutions.push((referrer as u32, request, target as u32));
            }
        }
    }
    ModuleGraphSources {
        realm_requests: Default::default(),
        modules,
        entry: entry.try_into().expect("fixture entry fits"),
        resolutions,
    }
}

fn admitted(sources: &ModuleGraphSources) -> ModuleGraphIr {
    link_loaded_graph(sources, true)
        .unwrap_or_else(|rejection| panic!("Script graph rejected: {:?}", rejection.diagnostics))
}

fn stage(
    graph: &ModuleGraphIr,
    referrer: &str,
    specifier: &str,
    phase: ImportPhaseIr,
) -> DynamicModuleRejectionStage {
    let key = ModuleKey::from_host(referrer);
    let referrer = if graph.unit(graph.entry).record.key == key {
        graph.entry
    } else {
        graph.keys[&key]
    };
    let matches = graph
        .dynamic_rejections
        .iter()
        .filter(|rejection| {
            rejection.referrer == referrer
                && rejection.request.specifier() == specifier
                && rejection.request.phase() == phase
        })
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "one request-owned rejection");
    matches[0].stage
}

#[test]
fn script_admission_preserves_independent_valid_closures_and_rejection_stages() {
    let source = "import('./malformed.js'); import('./missing-export.js'); import('./valid.js');";
    let graph = admitted(&script_sources(
        &[
            ("entry.js", source),
            ("malformed.js", "invalid syntax!"),
            (
                "missing-export.js",
                "import { missing } from './shared.js';",
            ),
            ("valid.js", "export { value } from './shared.js';"),
            ("shared.js", "export const value = 7;"),
        ],
        0,
    ));
    assert!(graph.entry_is_script);
    assert_eq!(graph.unit(graph.entry).source_text, source);
    assert!(!graph.unit(graph.entry).record.script_entry_strict);
    assert_eq!(graph.units.len(), 3);
    assert!(graph.keys.contains_key(&ModuleKey::from_host("valid.js")));
    assert!(graph.keys.contains_key(&ModuleKey::from_host("shared.js")));
    assert!(!graph
        .keys
        .contains_key(&ModuleKey::from_host("malformed.js")));
    assert!(!graph
        .keys
        .contains_key(&ModuleKey::from_host("missing-export.js")));
    assert!(graph.link_errors.is_empty());
    assert_eq!(graph.dynamic_rejections.len(), 2);
    assert_eq!(
        stage(
            &graph,
            "entry.js",
            "./malformed.js",
            ImportPhaseIr::Evaluation
        ),
        DynamicModuleRejectionStage::ModuleLoad
    );
    assert_eq!(
        stage(
            &graph,
            "entry.js",
            "./missing-export.js",
            ImportPhaseIr::Evaluation
        ),
        DynamicModuleRejectionStage::Dependencies
    );
}

#[test]
fn script_with_only_rejected_targets_retains_its_original_entry() {
    let source = "var entryRan = true; import('./invalid.js');";
    let graph = admitted(&script_sources(
        &[("entry.js", source), ("invalid.js", "invalid syntax!")],
        0,
    ));
    assert!(graph.entry_is_script);
    assert_eq!(graph.units.len(), 1);
    assert_eq!(graph.unit(graph.entry).source_text, source);
    assert!(graph.unit(graph.entry).record.requested_modules.is_empty());
    assert!(graph.unit(graph.entry).record.import_entries.is_empty());
    assert!(graph.dynamic_components().is_empty());
    assert_eq!(graph.dynamic_rejections.len(), 1);
}

#[test]
fn projected_nonzero_script_entry_keeps_its_parse_goal_and_strictness() {
    let source = "'use strict'; import('./valid.js'); import('./invalid.js');";
    let graph = admitted(&script_sources(
        &[
            ("valid.js", "export const value = 3;"),
            ("invalid.js", "invalid syntax!"),
            ("entry.js", source),
        ],
        2,
    ));
    assert!(graph.entry_is_script);
    assert_eq!(graph.entry, 1);
    assert!(!graph.keys.contains_key(&ModuleKey::from_host("entry.js")));
    assert_eq!(graph.unit(graph.entry).source_text, source);
    assert!(graph.unit(graph.entry).record.script_entry_strict);
    assert_eq!(
        stage(
            &graph,
            "entry.js",
            "./invalid.js",
            ImportPhaseIr::Evaluation
        ),
        DynamicModuleRejectionStage::ModuleLoad
    );
}

#[test]
fn evaluation_and_defer_occurrences_keep_separate_rejection_ownership() {
    let graph = admitted(&script_sources(
        &[
            (
                "entry.js",
                "import('./invalid.js'); import.defer('./invalid.js');",
            ),
            ("invalid.js", "invalid syntax!"),
        ],
        0,
    ));
    assert_eq!(graph.dynamic_rejections.len(), 2);
    for phase in [ImportPhaseIr::Evaluation, ImportPhaseIr::Defer] {
        assert_eq!(
            stage(&graph, "entry.js", "./invalid.js", phase),
            DynamicModuleRejectionStage::ModuleLoad
        );
    }
}

#[test]
fn transitive_parse_and_deferred_link_failures_keep_the_dependency_continuation() {
    let graph = admitted(&script_sources(
        &[
            (
                "entry.js",
                "import('./transitive.js'); import.defer('./missing.js');",
            ),
            ("transitive.js", "import './invalid.js';"),
            ("invalid.js", "invalid syntax!"),
            ("missing.js", "import defer * as ns from './absent.js';"),
        ],
        0,
    ));
    assert_eq!(graph.units.len(), 1);
    assert_eq!(graph.dynamic_rejections.len(), 2);
    for (request, phase) in [
        ("./transitive.js", ImportPhaseIr::Evaluation),
        ("./missing.js", ImportPhaseIr::Defer),
    ] {
        assert_eq!(
            stage(&graph, "entry.js", request, phase),
            DynamicModuleRejectionStage::Dependencies
        );
    }
}

#[test]
fn script_partition_does_not_hide_contradictory_host_sources() {
    let mut sources = script_sources(
        &[
            ("entry.js", "import('./invalid.js'); import('./shared.js');"),
            ("invalid.js", "invalid syntax!"),
            ("shared.js", "export const value = 1;"),
        ],
        0,
    );
    sources.modules.push(ModuleSourceIr::new(
        ModuleKey::from_host("shared.js"),
        "export const value = 2;".into(),
        "file:///shared.js".into(),
    ));
    assert!(link_loaded_graph(&sources, true).is_err());
}

#[test]
fn script_partition_does_not_hide_contradictory_host_resolutions() {
    let mut sources = script_sources(
        &[
            ("entry.js", "import('./invalid.js'); import('./shared.js');"),
            ("invalid.js", "invalid syntax!"),
            ("shared.js", "export const value = 1;"),
            ("other.js", "export const value = 2;"),
        ],
        0,
    );
    sources
        .resolutions
        .push((0, ModuleRequestKeyIr::plain("./shared.js"), 3));
    assert!(link_loaded_graph(&sources, true).is_err());
}

#[test]
fn source_first_duplicate_rows_promote_all_dependency_resolutions() {
    let files = [
        (
            "entry.js",
            "import './ordinary.js'; import source artifact from './shared.js';",
        ),
        (
            "shared.js",
            "import './invalid.js'; export const value = 1;",
        ),
        (
            "shared.js",
            "import './invalid.js'; export const value = 1;",
        ),
        ("ordinary.js", "import './shared.js';"),
        ("invalid.js", "invalid syntax!"),
    ];
    let sources = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: files
            .iter()
            .map(|(name, source)| {
                ModuleSourceIr::new(
                    ModuleKey::from_host(*name),
                    (*source).into(),
                    format!("file:///{name}"),
                )
            })
            .collect(),
        entry: 0,
        resolutions: vec![
            (0, ModuleRequestKeyIr::plain("./ordinary.js"), 3),
            (0, ModuleRequestKeyIr::plain("./shared.js"), 1),
            (3, ModuleRequestKeyIr::plain("./shared.js"), 1),
            // Only the agreeing duplicate row supplies this dependency. The
            // source occurrence reaches both rows before ordinary promotion.
            (2, ModuleRequestKeyIr::plain("./invalid.js"), 4),
        ],
    };
    let rejection = link_loaded_graph(&sources, false)
        .err()
        .expect("the recursively loaded child retains its parse failure");
    assert_eq!(rejection.diagnostics.len(), 1);
    assert_eq!(
        rejection.diagnostics[0].code(),
        Some(EarlyErrorCode::ModuleSyntax)
    );
}

#[test]
fn script_source_parse_failure_belongs_to_the_import_job() {
    let sources = script_sources(
        &[
            ("entry.js", "import.source('./invalid.js');"),
            ("invalid.js", "invalid syntax!"),
        ],
        0,
    );
    let graph = admitted(&sources);
    assert_eq!(graph.units.len(), 1);
    assert_eq!(
        graph.dynamic_rejections[0].stage,
        DynamicModuleRejectionStage::ModuleLoad
    );
    assert!(!graph.dynamic_rejections[0]
        .message
        .contains("no source representation"));
}

#[test]
fn script_and_same_url_module_keep_independent_source_and_rejection_ownership() {
    let graph = admitted(&script_sources(
        &[
            (
                "entry.js",
                "var scriptOnly = true; import('./entry.js'); import('./invalid.js');",
            ),
            ("entry.js", "export const moduleOnly = true;"),
            ("invalid.js", "invalid syntax!"),
        ],
        0,
    ));
    assert_eq!(graph.units.len(), 2);
    let module = graph.keys[&ModuleKey::from_host("entry.js")];
    assert_ne!(module, graph.entry, "a Script is outside the Module map");
    assert!(graph.unit(graph.entry).record.environment.is_empty());
    assert!(!graph.unit(module).record.environment.is_empty());
    assert!(graph.unit(module).source_text.contains("moduleOnly"));
    assert_eq!(graph.dynamic_components()[0].target(), module);
    assert_eq!(graph.dynamic_rejections[0].referrer, graph.entry);
    assert_eq!(
        graph.dynamic_rejections[0].stage,
        DynamicModuleRejectionStage::ModuleLoad
    );
}

#[test]
fn script_private_identifier_collisions_cannot_be_hidden_by_dynamic_rejections() {
    for source in [
        "var $lila$module$import$0 = () => 5; import('./invalid.js');",
        "function load($lila$module$import$0) { return import('./invalid.js'); }",
        "if (true) { var $lila$module$import$0 = 1; } import('./invalid.js');",
        r"let \u0024lila$module$import$0 = 1; import('./invalid.js');",
    ] {
        let rejection = link_loaded_graph(
            &script_sources(
                &[("entry.js", source), ("invalid.js", "invalid syntax!")],
                0,
            ),
            true,
        )
        .err()
        .expect("private identifiers are explicitly unsupported");
        assert!(
            rejection
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("linker-synthesized name")),
            "{source}: {:?}",
            rejection.diagnostics
        );
    }
}

#[test]
fn private_name_property_and_string_data_do_not_reject_a_script() {
    let graph = admitted(&script_sources(
        &[
            ("entry.js", "var obj = {$lila$module$import$0: 1}; obj.$lila$module$import$0; '$lila$module$import$0'; import('./valid.js');"),
            ("valid.js", "export const value = 1;"),
        ],
        0,
    ));
    assert!(graph.dynamic_rejections.is_empty());
    assert_eq!(graph.units.len(), 2);
}

#[test]
fn module_nested_private_identifiers_share_script_admission() {
    let source = "export function load($lila$module$import$0) { return import('./valid.js'); }";
    let sources = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![ModuleSourceIr::new(
            ModuleKey::from_host("entry.js"),
            source.into(),
            "file:///entry.js".into(),
        )],
        entry: 0,
        resolutions: Vec::new(),
    };
    let rejection = link_loaded_graph(&sources, false)
        .err()
        .expect("nested Module collision is explicit");
    assert!(rejection
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("linker-synthesized name")));
}

fn complete(sources: &ModuleGraphSources) -> ModuleGraphIr {
    let entry_is_script = sources.modules[sources.entry as usize].goal() == ParseGoal::Script;
    link_complete_catalog(sources, entry_is_script)
        .unwrap_or_else(|rejection| panic!("catalog rejected: {:?}", rejection.diagnostics))
}

fn catalog(files: &[(&str, &str)], entry: usize) -> ModuleGraphSources {
    let mut sources = script_sources(files, entry);
    // These controls author the host table explicitly. Source-derived discovery
    // would omit exactly the computed and runtime-attribute cases under test.
    sources.resolutions.clear();
    sources
}

fn attributed_key(specifier: &str, attributes: &[(&str, &str)]) -> ModuleRequestKeyIr {
    ModuleRequestKeyIr::try_new(
        specifier,
        attributes.iter().map(|(key, value)| ImportAttributeIr {
            key: (*key).into(),
            value: (*value).into(),
        }),
    )
    .expect("authored attribute keys are unique")
}

fn catalog_site(source: &str) -> DynamicImportSiteIr {
    let ParsedSource::Script(parsed) = parse(source, ParseOptions::script()).expect("site parses")
    else {
        panic!("Script site product");
    };
    super::super::record::script_entry_record_for_catalog(
        &parsed,
        0,
        ModuleKey::from_host("entry.js"),
    )
    .expect("site has no linker identifier collision")
    .dynamic_import_sites
    .remove(0)
}

#[test]
fn catalog_occurrence_uses_known_operands_and_preserves_every_site_phase() {
    let empty = ModuleRequestKeyIr::plain("./a.js");
    let attributed = attributed_key("./a.js", &[("type", "data")]);
    let other_value = attributed_key("./a.js", &[("type", "other")]);
    let other_specifier = attributed_key("./b.js", &[("type", "data")]);
    for (phase, call) in [
        (ImportPhaseIr::Evaluation, "import"),
        (ImportPhaseIr::Defer, "import.defer"),
        (ImportPhaseIr::Source, "import.source"),
    ] {
        let site = catalog_site(&format!("{call}('./a.js', {{with: {{type: 'data'}}}});"));
        assert_eq!(site.catalog_occurrence(&empty), None);
        assert_eq!(site.catalog_occurrence(&other_value), None);
        assert_eq!(site.catalog_occurrence(&other_specifier), None);
        assert_eq!(
            site.catalog_occurrence(&attributed),
            Some(ModuleRequestIr::from_key(attributed.clone(), phase))
        );
        let runtime = catalog_site(&format!("{call}('./a.js', options);"));
        assert!(runtime.catalog_occurrence(&empty).is_some());
        assert!(runtime.catalog_occurrence(&attributed).is_some());
        assert_eq!(runtime.catalog_occurrence(&other_specifier), None);
        let computed = catalog_site(&format!("{call}(key, {{with: {{type: 'data'}}}});"));
        assert!(computed.catalog_occurrence(&other_specifier).is_some());
        assert_eq!(computed.catalog_occurrence(&empty), None);
    }
}

#[test]
fn computed_catalog_site_admits_declared_empty_variants_and_their_static_dependencies() {
    let mut sources = catalog(
        &[
            ("entry.js", "var key = './one.js'; import(key);"),
            ("one.js", "export { value } from './dep.js';"),
            ("two.js", "export const value = 2;"),
            ("wrong-attributes.js", "invalid syntax!"),
            ("dep.js", "export const value = 1;"),
            ("unused-invalid.js", "invalid syntax!"),
            ("unused-private.js", "let $lila$module$import$0 = 1;"),
            ("unused-body.js", "throw 'unused body';"),
        ],
        0,
    );
    sources.resolutions = vec![
        (0, ModuleRequestKeyIr::plain("./one.js"), 1),
        (0, ModuleRequestKeyIr::plain("./two.js"), 2),
        (0, attributed_key("./one.js", &[("type", "data")]), 3),
        (1, ModuleRequestKeyIr::plain("./dep.js"), 4),
    ];
    let graph = complete(&sources);
    assert!(graph.entry_is_script);
    assert_eq!(graph.units.len(), 4);
    assert!(graph.dynamic_rejections.is_empty());
    assert_eq!(graph.dynamic_components().len(), 2);
    for name in ["one.js", "two.js", "dep.js"] {
        assert!(
            graph.keys.contains_key(&ModuleKey::from_host(name)),
            "{name}"
        );
    }
    for name in [
        "wrong-attributes.js",
        "unused-invalid.js",
        "unused-private.js",
        "unused-body.js",
    ] {
        assert!(
            !graph.keys.contains_key(&ModuleKey::from_host(name)),
            "{name}"
        );
    }
    for component in graph.dynamic_components() {
        assert_eq!(component.referrer(), graph.entry);
        assert_eq!(component.request().phase(), ImportPhaseIr::Evaluation);
        assert!(component.request().attributes().is_empty());
        assert!(graph
            .resolve_dynamic_component(Some(graph.entry), component.request())
            .is_some());
        assert!(graph
            .resolve_dynamic_component(None, component.request())
            .is_none());
    }
}

#[test]
fn runtime_attributes_share_exact_component_and_rejection_projection() {
    let mut sources = catalog(
        &[
            (
                "entry.js",
                "var options = {}; import('./data.js', options);",
            ),
            ("empty.js", "export const value = 1;"),
            ("bad.js", "invalid syntax!"),
            ("other.js", "invalid syntax!"),
        ],
        0,
    );
    let attributed = attributed_key("./data.js", &[("\u{e000}", "bmp"), ("\u{10000}", "astral")]);
    assert_eq!(attributed.attributes()[0].key, "\u{10000}");
    sources.resolutions = vec![
        (0, ModuleRequestKeyIr::plain("./data.js"), 1),
        (0, attributed.clone(), 2),
        (0, ModuleRequestKeyIr::plain("./other.js"), 3),
    ];
    let graph = complete(&sources);
    assert_eq!(graph.units.len(), 2);
    assert_eq!(graph.dynamic_components().len(), 1);
    assert_eq!(graph.dynamic_rejections.len(), 1);
    let rejection = &graph.dynamic_rejections[0];
    assert_eq!(rejection.referrer, graph.entry);
    assert_eq!(rejection.request.key(), &attributed);
    assert_eq!(rejection.stage, DynamicModuleRejectionStage::ModuleLoad);
    assert_eq!(graph.dynamic_components()[0].request().attributes(), &[]);
    // Deferred rejection rows belong to the canonical module-job dispatcher,
    // rather than the retained merged-source dispatcher.
    let dispatcher = format!(
        "{} {}",
        graph.module_initialization_dispatchers(true),
        graph.module_initialization_dispatchers(false)
    );
    assert!(dispatcher.contains("astral"));
    assert!(dispatcher.contains("bmp"));
    assert!(dispatcher.contains("$lila$module$SyntaxError"));
    assert!(!graph.keys.contains_key(&ModuleKey::from_host("other.js")));
}

#[test]
fn computed_defer_admits_dependencies_and_preserves_dependency_rejection_stage() {
    let mut sources = catalog(
        &[
            ("entry.js", "var key = './valid.js'; import.defer(key);"),
            ("valid.js", "export { value } from './dep.js';"),
            ("dep.js", "export const value = 3;"),
            ("bad-link.js", "import { absent } from './dep.js';"),
            ("bad-parse.js", "invalid syntax!"),
        ],
        0,
    );
    sources.resolutions = vec![
        (0, ModuleRequestKeyIr::plain("./valid.js"), 1),
        (0, ModuleRequestKeyIr::plain("./bad-link.js"), 3),
        (0, ModuleRequestKeyIr::plain("./bad-parse.js"), 4),
        (1, ModuleRequestKeyIr::plain("./dep.js"), 2),
        (3, ModuleRequestKeyIr::plain("./dep.js"), 2),
    ];
    let graph = complete(&sources);
    assert_eq!(graph.units.len(), 3);
    assert_eq!(graph.dynamic_components().len(), 1);
    assert_eq!(
        graph.dynamic_components()[0].request().phase(),
        ImportPhaseIr::Defer
    );
    for name in ["valid.js", "dep.js"] {
        assert_eq!(
            graph.evaluation_mode(graph.keys[&ModuleKey::from_host(name)]),
            ModuleEvaluationModeIr::Deferred
        );
    }
    assert_eq!(
        stage(&graph, "entry.js", "./bad-link.js", ImportPhaseIr::Defer),
        DynamicModuleRejectionStage::Dependencies
    );
    assert_eq!(
        stage(&graph, "entry.js", "./bad-parse.js", ImportPhaseIr::Defer),
        DynamicModuleRejectionStage::ModuleLoad
    );
}

#[test]
fn computed_source_checks_only_target_parse_and_never_opens_its_requests() {
    let mut sources = catalog(
        &[
            ("entry.js", "var key = './source.js'; import.source(key);"),
            (
                "source.js",
                "import './invalid-dep.js'; import(other); export const value = 1;",
            ),
            ("invalid-dep.js", "invalid syntax!"),
            ("private-child.js", "let $lila$module$import$0 = 1;"),
            ("invalid-source.js", "invalid syntax!"),
        ],
        0,
    );
    sources.resolutions = vec![
        (0, ModuleRequestKeyIr::plain("./source.js"), 1),
        (0, ModuleRequestKeyIr::plain("./invalid-source.js"), 4),
        (1, ModuleRequestKeyIr::plain("./invalid-dep.js"), 2),
        (1, ModuleRequestKeyIr::plain("./private-child.js"), 3),
    ];
    let graph = complete(&sources);
    assert_eq!(graph.units.len(), 1);
    assert!(graph.dynamic_components().is_empty());
    assert_eq!(graph.dynamic_rejections.len(), 2);
    for rejection in &graph.dynamic_rejections {
        assert_eq!(rejection.stage, DynamicModuleRejectionStage::ModuleLoad);
        assert_eq!(rejection.request.phase(), ImportPhaseIr::Source);
        assert_eq!(rejection.referrer, graph.entry);
        assert_eq!(
            rejection.message.contains("no source representation"),
            rejection.request.specifier() == "./source.js"
        );
    }
}

#[test]
fn source_then_evaluation_promotes_the_target_and_reaches_its_computed_imports() {
    let mut sources = catalog(
        &[
            (
                "entry.js",
                "import.source('./target.js'); import('./target.js');",
            ),
            (
                "target.js",
                "var key = './bad.js'; import(key); export const value = 1;",
            ),
            ("bad.js", "invalid syntax!"),
        ],
        0,
    );
    sources.resolutions = vec![
        (0, ModuleRequestKeyIr::plain("./target.js"), 1),
        (1, ModuleRequestKeyIr::plain("./bad.js"), 2),
    ];
    let graph = complete(&sources);
    assert_eq!(graph.units.len(), 2);
    assert_eq!(graph.dynamic_rejections.len(), 2);
    assert_eq!(
        stage(&graph, "entry.js", "./target.js", ImportPhaseIr::Source),
        DynamicModuleRejectionStage::ModuleLoad
    );
    assert_eq!(
        stage(&graph, "target.js", "./bad.js", ImportPhaseIr::Evaluation),
        DynamicModuleRejectionStage::ModuleLoad
    );
    assert_eq!(
        graph
            .dynamic_components()
            .iter()
            .filter(|component| component.request().phase() == ImportPhaseIr::Evaluation)
            .count(),
        1
    );
}

#[test]
fn complete_catalog_preserves_static_rejection_and_selected_compiler_limits() {
    let sources = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![
            ModuleSourceIr::new(
                ModuleKey::from_host("entry.js"),
                "import './invalid.js';".into(),
                "lila://entry.js".into(),
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("invalid.js"),
                "invalid syntax!".into(),
                "lila://invalid.js".into(),
            ),
        ],
        entry: 0,
        resolutions: vec![(0, ModuleRequestKeyIr::plain("./invalid.js"), 1)],
    };
    let rejection = link_complete_catalog(&sources, false)
        .err()
        .expect("static parse failure rejects entry");
    assert!(rejection.diagnostics.iter().all(can_reject_import));
    let mut sources = catalog(
        &[
            ("entry.js", "import(key);"),
            ("private.js", "let $lila$module$import$0 = 1;"),
        ],
        0,
    );
    sources
        .resolutions
        .push((0, ModuleRequestKeyIr::plain("./private.js"), 1));
    let rejection = link_complete_catalog(&sources, true)
        .err()
        .expect("selected compiler limitation remains compiler failure");
    assert!(rejection
        .diagnostics
        .iter()
        .any(|diagnostic| !can_reject_import(diagnostic)));
}

#[test]
fn unused_host_contradictions_and_script_targets_remain_catalog_errors() {
    let mut sources = catalog(
        &[
            ("entry.js", "var entry = 1;"),
            ("same.js", "export const value = 1;"),
            ("same.js", "export const value = 2;"),
        ],
        0,
    );
    assert!(link_complete_catalog(&sources, true).is_err());
    sources.modules.pop();
    sources
        .resolutions
        .push((0, ModuleRequestKeyIr::plain("./script.js"), 0));
    assert!(link_complete_catalog(&sources, true).is_err());
    sources.resolutions = vec![(9, ModuleRequestKeyIr::plain("./same.js"), 1)];
    assert!(link_complete_catalog(&sources, true).is_err());
    sources.resolutions = vec![(0, ModuleRequestKeyIr::plain("./same.js"), 9)];
    assert!(link_complete_catalog(&sources, true).is_err());
}

#[test]
fn loaded_closure_keeps_its_computed_discovery_and_unreferenced_body_behavior() {
    let mut sources = catalog(
        &[
            ("entry.js", "import(key);"),
            ("target.js", "export const value = 1;"),
        ],
        0,
    );
    sources
        .resolutions
        .push((0, ModuleRequestKeyIr::plain("./target.js"), 1));
    let loaded = admitted(&sources);
    assert_eq!(loaded.units.len(), 2);
    assert!(loaded.dynamic_components().is_empty());
    assert_eq!(
        loaded.evaluation_mode(loaded.keys[&ModuleKey::from_host("target.js")]),
        ModuleEvaluationModeIr::Eager
    );
    let declared = complete(&sources);
    assert_eq!(declared.dynamic_components().len(), 1);
}

#[test]
fn complete_script_and_same_identity_module_keep_independent_referrer_tables() {
    let mut sources = catalog(
        &[
            ("entry.js", "import('./entry.js');"),
            (
                "entry.js",
                "var request = './child.js'; import(request); export const value = 1;",
            ),
            ("child.js", "export const value = 2;"),
            ("wrong-role.js", "invalid syntax!"),
        ],
        0,
    );
    sources.resolutions = vec![
        (0, ModuleRequestKeyIr::plain("./entry.js"), 1),
        (1, ModuleRequestKeyIr::plain("./child.js"), 2),
        (0, ModuleRequestKeyIr::plain("./child.js"), 3),
    ];
    let graph = complete(&sources);
    assert_eq!(graph.units.len(), 3);
    assert!(graph.dynamic_rejections.is_empty());
    let module = graph.keys[&ModuleKey::from_host("entry.js")];
    assert_ne!(module, graph.entry);
    let child_request = ModuleRequestIr::plain("./child.js");
    let child = graph
        .resolve_dynamic_component(Some(module), &child_request)
        .expect("module-owned child variant");
    assert_eq!(child.target_key(), &ModuleKey::from_host("child.js"));
    assert!(graph
        .resolve_dynamic_component(Some(graph.entry), &child_request)
        .is_none());
    assert_eq!(graph.dynamic_components().len(), 2);
}

#[test]
fn catalog_dynamic_non_scalar_literals_never_alias_display_spellings() {
    let key = ModuleRequestKeyIr::plain("\\uD800");
    assert!(catalog_site(r"import('\\uD800');")
        .catalog_occurrence(&key)
        .is_some());
    assert!(catalog_site(r"import('\uD800');")
        .catalog_occurrence(&key)
        .is_none());
    for (source, attributes) in [
        (
            r"import('./a.js', {with: {'\uD800': 'ok'}});",
            vec![("\\uD800", "ok")],
        ),
        (
            r"import('./a.js', {with: {type: '\uD800'}});",
            vec![("type", "\\uD800")],
        ),
        (
            r"import('./a.js', {with: {'\uD800': 'bad', '\\uD800': 'ok'}});",
            vec![("\\uD800", "ok")],
        ),
    ] {
        let key = attributed_key("./a.js", &attributes);
        assert!(
            catalog_site(source).catalog_occurrence(&key).is_none(),
            "{source}"
        );
    }
    let key = attributed_key("./a.js", &[("type", "ok")]);
    assert!(
        catalog_site(r"import('./a.js', {with: {type: '\uD800', type: 'ok'}});")
            .catalog_occurrence(&key)
            .is_some()
    );
    assert!(
        catalog_site(r"import('./a.js', {with: {type: 'ok', type: '\uD800'}});")
            .catalog_occurrence(&key)
            .is_none()
    );
    assert!(
        catalog_site(r"import('./a.js', {with: {type: '\uD800'}, with: {type: 'ok'}});")
            .catalog_occurrence(&key)
            .is_some()
    );
    assert!(catalog_site(r"import('./a.js', options);")
        .catalog_occurrence(&key)
        .is_some());
}

#[test]
fn complete_static_non_scalar_requests_stay_unresolved_before_display_attribute_canonicalization() {
    for source in [
        r"import '\uD800';",
        r"import './a.js' with {'\uD800': 'ok'};",
        r"import './a.js' with {type: '\uD800'};",
        r"export * from './a.js' with {'\uD800': 'bad', '\\uD800': 'ok'};",
        r"import './a.js' with {'\uD800': 'bad', '\\uD800': 'ok'};",
    ] {
        let sources = ModuleGraphSources {
            realm_requests: Default::default(),
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("entry.js"),
                    source.into(),
                    "lila://entry.js".into(),
                ),
                ModuleSourceIr::new(
                    ModuleKey::from_host("target.js"),
                    "export const value = 1;".into(),
                    "lila://target.js".into(),
                ),
            ],
            entry: 0,
            resolutions: vec![
                (0, ModuleRequestKeyIr::plain("\\uD800"), 1),
                (0, ModuleRequestKeyIr::plain("./a.js"), 1),
                (0, attributed_key("./a.js", &[("\\uD800", "ok")]), 1),
                (0, attributed_key("./a.js", &[("type", "\\uD800")]), 1),
            ],
        };
        let rejection = link_complete_catalog(&sources, false)
            .err()
            .expect("non-scalar static request has no exact host key");
        assert!(
            rejection
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code() == Some(EarlyErrorCode::ModuleUnresolved)),
            "{source}: {:?}",
            rejection.diagnostics
        );
    }
    let source = r"import '\\uD800'; import './a.js' with {'\\uD800': 'ok'};";
    let sources = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![
            ModuleSourceIr::new(
                ModuleKey::from_host("entry.js"),
                source.into(),
                "lila://entry.js".into(),
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("target.js"),
                "export const value = 1;".into(),
                "lila://target.js".into(),
            ),
        ],
        entry: 0,
        resolutions: vec![
            (0, ModuleRequestKeyIr::plain("\\uD800"), 1),
            (0, attributed_key("./a.js", &[("\\uD800", "ok")]), 1),
        ],
    };
    let graph = complete(&sources);
    assert!(graph.link_errors.is_empty());
    assert_eq!(graph.units.len(), 2);
}
