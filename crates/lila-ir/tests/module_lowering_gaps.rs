use lila_ir::{lower_module_graph, ModuleGraphSources, ModuleKey, ModuleSourceIr, ProgramIr};

fn sources(files: &[(&str, &str)]) -> ModuleGraphSources {
    let modules: Vec<_> = files
        .iter()
        .map(|(name, source)| {
            let key = ModuleKey::from_host(*name);
            let url = format!("file:///{}", key.as_str());
            ModuleSourceIr::new(key, (*source).into(), url)
        })
        .collect();
    let mut resolutions = Vec::new();
    for (referrer, module) in modules.iter().enumerate() {
        for request in module.module_requests().unwrap_or_default() {
            let target = files
                .iter()
                .position(|(name, _)| *name == request.specifier().trim_start_matches("./"))
                .expect("fixture target exists");
            resolutions.push((referrer as u32, request, target as u32));
        }
    }
    ModuleGraphSources {
        realm_requests: Default::default(),
        modules,
        entry: 0,
        resolutions,
    }
}

fn lower(files: &[(&str, &str)]) -> ProgramIr {
    lower_module_graph(&sources(files))
}

fn assert_lowers(files: &[(&str, &str)]) -> ProgramIr {
    let program = lower(files);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
}

#[test]
fn string_named_module_syntax_links() {
    assert_lowers(&[
        (
            "a.js",
            "import * as Scouts from \"./a.js\"; export * as \"All\" from \"./f.js\"; Scouts.All;",
        ),
        ("f.js", "export var x = 1;"),
    ]);
    assert_lowers(&[
        (
            "a.js",
            "import { \"a-b\" as ab } from \"./f.js\"; export { ab as \"x y\" };",
        ),
        ("f.js", "var v = 1; export { v as \"a-b\" };"),
    ]);
}

#[test]
fn empty_attribute_lists_link() {
    assert_lowers(&[
        (
            "a.js",
            "import x from './f1.js' with {}; import './f2.js' with {}; export * from './f3.js' with {};",
        ),
        ("f1.js", "export default 1;"),
        ("f2.js", ""),
        ("f3.js", "export var y = 2;"),
    ]);
}

#[test]
fn programs_without_a_script_always_block() {
    let program = lower(&[
        ("a.js", "import { missing } from \"./f.js\";"),
        ("f.js", ""),
    ]);
    assert!(program.script.is_none());
    assert!(program.wasm_blocking_diagnostic().is_some());
}

#[test]
fn awaited_exported_var_pattern_declares_its_names() {
    let program = assert_lowers(&[(
        "a.js",
        "export var name1 = await []; export var { x = await [] } = {};",
    )]);
    let script = program.script.as_ref().expect("script IR");
    let dump = format!("{:?}", script.functions);
    assert!(
        dump.contains("Var([VarDeclaratorIr { name: \"x\", init: None }])"),
        "x must be declared as a var before the awaited pattern"
    );
}
