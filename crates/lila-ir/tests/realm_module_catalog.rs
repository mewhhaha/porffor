use std::collections::BTreeMap;

use lila_front::{parse, ParseOptions, ParsedSource};
use lila_ir::{
    lower_complete_module_catalog, lower_script_graph, ExprIr, HostSurfacePolicy,
    ModuleGraphSources, ModuleKey, ModuleRequestKeyIr, ModuleSourceIr, RealmModuleResolutionIr,
    StatementIr,
};

fn sources(entry: &str, modules: &[(&str, &str)]) -> ModuleGraphSources {
    let ParsedSource::Script(parsed) = parse(entry, ParseOptions::script()).unwrap() else {
        unreachable!()
    };
    let mut sources = vec![ModuleSourceIr::from_parsed_script(
        ModuleKey::from_host("entry.js"),
        "lila://entry.js".into(),
        parsed,
    )];
    sources.extend(modules.iter().map(|(name, source)| {
        ModuleSourceIr::new(
            ModuleKey::from_host(*name),
            (*source).into(),
            format!("lila://{name}"),
        )
    }));
    ModuleGraphSources {
        modules: sources,
        entry: 0,
        resolutions: Vec::new(),
        realm_requests: BTreeMap::new(),
    }
}

#[test]
fn realm_catalog_has_a_reusable_initializer_and_a_distinct_source_resolution_domain() {
    let mut sources = sources(
        "import('same');",
        &[
            ("realm.js", "export const x = 1;"),
            ("script.js", "export const x = 2;"),
        ],
    );
    sources
        .resolutions
        .push((0, ModuleRequestKeyIr::plain("same"), 2));
    sources.realm_requests.insert(
        ModuleRequestKeyIr::plain("same"),
        RealmModuleResolutionIr::Loaded(1),
    );
    let program = lower_complete_module_catalog(&sources, HostSurfacePolicy::Product, None);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.as_ref().unwrap();
    let RealmModuleResolutionIr::Loaded(realm_target) =
        graph.realm_requests[&ModuleRequestKeyIr::plain("same")]
    else {
        panic!("Realm target survives admission");
    };
    assert_ne!(graph.dynamic_components()[0].target(), realm_target);
    let script = program.script.unwrap();
    let plan = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(value) => match &value.expr {
                ExprIr::ModuleExecutionGraph(graph) => Some(graph),
                _ => None,
            },
            _ => None,
        })
        .expect("startup uses the reusable graph operation");
    let initializer = script
        .functions
        .iter()
        .find(|function| Some(&function.id) == plan.initializer())
        .unwrap();
    assert!(initializer.body.statements.iter().any(|statement| matches!(statement,
        StatementIr::Expression(value) if matches!(&value.expr, ExprIr::ModuleExecutionGraph(graph) if graph.initializer().is_none())
    )));
    assert!(script
        .functions
        .iter()
        .any(|function| Some(&function.id) == plan.realm_import_dispatcher()));
    assert!(!script
        .global_bindings
        .iter()
        .any(|binding| binding.name.contains("$lila$module$")));
}

#[test]
fn realm_only_parse_and_link_failures_stay_import_rejections() {
    let mut sources = sources(
        "0;",
        &[
            ("syntax.js", "export const = ;"),
            ("link.js", "import { missing } from './ok.js';"),
            ("ok.js", "export const ok = 1;"),
        ],
    );
    sources
        .resolutions
        .push((2, ModuleRequestKeyIr::plain("./ok.js"), 3));
    for (specifier, target) in [("syntax", 1), ("link", 2)] {
        sources.realm_requests.insert(
            ModuleRequestKeyIr::plain(specifier),
            RealmModuleResolutionIr::Loaded(target),
        );
    }
    sources.realm_requests.insert(
        ModuleRequestKeyIr::plain("host"),
        RealmModuleResolutionIr::Rejected("host refused exact request".into()),
    );
    let program = lower_script_graph(&sources);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.unwrap();
    assert_eq!(graph.units.len(), 1);
    for key in ["syntax", "link", "host"] {
        assert!(
            matches!(&graph.realm_requests[&ModuleRequestKeyIr::plain(key)], RealmModuleResolutionIr::Rejected(message) if !message.is_empty())
        );
    }
    assert_eq!(
        graph.realm_requests[&ModuleRequestKeyIr::plain("host")],
        RealmModuleResolutionIr::Rejected("host refused exact request".into())
    );
}

#[test]
fn literal_discovery_collects_candidates_without_claiming_the_method_identity() {
    let ParsedSource::Script(parsed) = parse(
        "const value = { importValue(x) { return x; } }; value['importValue']('./local.js'); value.importValue('./other.js'); value.importValue(name);",
        ParseOptions::script(),
    ).unwrap() else { unreachable!() };
    assert_eq!(
        lila_ir::scan_script_realm_module_requests(&parsed),
        vec![
            ModuleRequestKeyIr::plain("./local.js"),
            ModuleRequestKeyIr::plain("./other.js")
        ]
    );
    let program = lila_ir::lower(&ParsedSource::Script(parsed));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(
        program.modules.is_none(),
        "a discovery candidate is not an intrinsic call or a graph by itself"
    );
}
