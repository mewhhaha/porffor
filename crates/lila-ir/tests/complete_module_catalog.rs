use lila_front::{parse, ParseGoal, ParseOptions, ParsedSource};
use lila_ir::{
    lower_complete_module_catalog, HostSurfacePolicy, ModuleGraphSources, ModuleKey,
    ModuleRequestKeyIr, ModuleSourceIr,
};

#[test]
fn complete_lowering_uses_the_actual_script_entry_and_excludes_unused_module_bodies() {
    let source = "var request = './target.js'; import(request);";
    let ParsedSource::Script(parsed) = parse(source, ParseOptions::script()).unwrap() else {
        panic!("Script parse product");
    };
    let sources = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![
            ModuleSourceIr::new(
                ModuleKey::from_host("unused.js"),
                "invalid syntax!".into(),
                "lila://unused.js".into(),
            ),
            ModuleSourceIr::from_parsed_script(
                ModuleKey::from_host("entry.js"),
                "lila://entry.js".into(),
                parsed,
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("target.js"),
                "export const value = 5;".into(),
                "lila://independent-meta.js".into(),
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("unused-body.js"),
                "throw 'never runs';".into(),
                "lila://unused-body.js".into(),
            ),
        ],
        entry: 1,
        resolutions: vec![(1, ModuleRequestKeyIr::plain("./target.js"), 2)],
    };
    let program = lower_complete_module_catalog(&sources, HostSurfacePolicy::Product, None);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert_eq!(program.goal, ParseGoal::Script);
    assert_eq!(program.source_len, source.len());
    let graph = program
        .modules
        .as_ref()
        .expect("entry has compiled import jobs");
    assert!(graph.entry_is_script);
    assert_eq!(graph.units.len(), 2);
    assert!(!graph.unit(graph.entry).record.script_entry_strict);
    assert_eq!(graph.unit(graph.entry).source_text, source);
    assert_eq!(graph.dynamic_components().len(), 1);
    let component = &graph.dynamic_components()[0];
    assert_eq!(component.referrer(), graph.entry);
    assert_eq!(
        graph.unit(component.target()).meta_url,
        "lila://independent-meta.js"
    );
    assert_eq!(component.request().specifier(), "./target.js");
    assert!(program
        .script
        .as_ref()
        .unwrap()
        .module_entry_evaluation()
        .is_none());
}

#[test]
fn complete_module_entry_keeps_its_evaluation_owner_and_independently_parsed_prelude() {
    let source = "export const value = 5;";
    let ParsedSource::Module(parsed) = parse(source, ParseOptions::module()).unwrap() else {
        panic!("Module parse product");
    };
    let ParsedSource::Script(prelude) =
        parse("var fromPrelude = 3;", ParseOptions::script()).unwrap()
    else {
        panic!("Prelude stays Script");
    };
    let sources = ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![
            ModuleSourceIr::from_parsed(
                ModuleKey::from_host("entry.js"),
                "lila://entry.js".into(),
                parsed,
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("unused.js"),
                "invalid syntax!".into(),
                "lila://unused.js".into(),
            ),
        ],
        entry: 0,
        resolutions: Vec::new(),
    };
    let program =
        lower_complete_module_catalog(&sources, HostSurfacePolicy::Product, Some(&prelude));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert_eq!(program.goal, ParseGoal::Module);
    assert_eq!(program.modules.as_ref().unwrap().units.len(), 1);
    let script = program.script.as_ref().unwrap();
    assert!(script.module_entry_evaluation().is_some());
    let unit = script
        .module_prelude
        .as_ref()
        .expect("parsed Prelude is an owned Script unit");
    assert!(!unit.strict);
    assert_eq!(unit.declarations.var_names, vec!["fromPrelude"]);
}
