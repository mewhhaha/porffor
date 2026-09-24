//! A Script that writes `import()` owns the canonical module execution graph.
//!
//! The Script stays Script code (its own strictness, global scope and root
//! `this`), its targets evaluate only from their import jobs, and a computed
//! specifier is served from the referrer's host resolution table.

use lila_front::{parse, ParseOptions, ParsedSource};
use lila_ir::{
    lower_script_graph, ExprIr, ImportPhaseIr, ModuleExecutionGraphIr, ModuleGraphSources,
    ModuleKey, ModuleRequestKeyIr, ModuleSourceIr, ProgramIr, ScriptIr, StatementIr, TypedExpr,
};

/// A Script entry followed by modules, with every literal request and every
/// `extra` row resolved by file name.
fn script_graph(files: &[(&str, &str)], extra: &[(u32, &str)]) -> (ModuleGraphSources, ProgramIr) {
    let modules: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(index, (name, source))| {
            let key = ModuleKey::from_host(*name);
            let url = format!("file:///{name}");
            if index == 0 {
                let ParsedSource::Script(parsed) =
                    parse(*source, ParseOptions::script()).expect("Script entry parses")
                else {
                    panic!("Script parse goal");
                };
                ModuleSourceIr::from_parsed_script(key, url, parsed)
            } else {
                ModuleSourceIr::new(key, (*source).into(), url)
            }
        })
        .collect();
    let target = |specifier: &str| {
        files
            .iter()
            .skip(1)
            .position(|(name, _)| *name == specifier.trim_start_matches("./"))
            .map(|position| position as u32 + 1)
    };
    let mut resolutions = Vec::new();
    for (referrer, module) in modules.iter().enumerate() {
        for request in module.module_requests().unwrap_or_default() {
            if let Some(target) = target(request.specifier()) {
                resolutions.push((referrer as u32, request, target));
            }
        }
    }
    for (referrer, specifier) in extra {
        let target = target(specifier).expect("declared spelling names a fixture");
        resolutions.push((*referrer, ModuleRequestKeyIr::plain(*specifier), target));
    }
    let sources = ModuleGraphSources {
        modules,
        entry: 0,
        resolutions,
    };
    let program = lower_script_graph(&sources);
    (sources, program)
}

fn supported(program: &ProgramIr) -> &ScriptIr {
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.as_ref().expect("lowered Script")
}

fn activation_graph(script: &ScriptIr) -> Option<&ModuleExecutionGraphIr> {
    script.body.statements.iter().find_map(|statement| {
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::ModuleExecutionGraph(graph),
            ..
        }) = statement
        else {
            return None;
        };
        Some(graph.as_ref())
    })
}

#[test]
fn a_script_with_only_computed_specifiers_still_lowers_its_import_call() {
    let (_, program) = script_graph(
        &[(
            "entry.js",
            "const obj = { get err() { throw 1; } }; try { import(obj.err); } catch (e) {}",
        )],
        &[],
    );
    let script = supported(&program);
    let graph = activation_graph(script).expect("the Script owns the canonical graph");
    assert_eq!(graph.record_count(), 1);
    assert!(graph.activations().is_empty(), "a Script is not a module");
    assert!(script.module_entry_evaluation().is_none());
    assert!(program
        .modules
        .as_ref()
        .unwrap()
        .dynamic_components()
        .is_empty());
}

#[test]
fn script_targets_have_activations_and_no_initial_evaluation() {
    let (_, program) = script_graph(
        &[
            (
                "entry.js",
                "import('./value.js').then(ns => print(ns.value));",
            ),
            ("value.js", "export const value = 7;"),
        ],
        &[],
    );
    let script = supported(&program);
    let graph = activation_graph(script).expect("canonical graph");
    assert_eq!(graph.activations().len(), 1);
    assert_eq!(graph.activations()[0].module(), 1);
    // Only the Script's own statements follow the graph: nothing evaluates
    // the target before its import job does.
    fn evaluates(statements: &[StatementIr]) -> bool {
        statements.iter().any(|statement| {
            matches!(
                statement,
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::ModuleEvaluate(_) | ExprIr::ModuleEntryEvaluation(_),
                    ..
                })
            )
        })
    }
    assert!(!evaluates(&script.body.statements));
}

#[test]
fn the_script_keeps_its_own_strictness() {
    for (source, strict) in [
        ("import('./value.js'); with ({}) {}", false),
        ("'use strict'; import('./value.js');", true),
    ] {
        let (_, program) = script_graph(
            &[
                ("entry.js", source),
                ("value.js", "export const value = 1;"),
            ],
            &[],
        );
        assert_eq!(supported(&program).strict, strict, "{source}");
    }
}

#[test]
fn module_code_reads_undefined_this_while_the_script_reads_the_global_object() {
    let (_, program) = script_graph(
        &[
            ("entry.js", "print(this); import('./this.js');"),
            (
                "this.js",
                "export const direct = this; export const arrow = (() => this)();",
            ),
        ],
        &[],
    );
    let script = supported(&program);
    fn count_this(statements: &[StatementIr]) -> usize {
        format!("{statements:?}").matches("expr: This }").count()
    }
    // The Script's root read stays the dynamic global `this`.
    assert!(count_this(&script.body.statements) >= 1);
    // Neither module read, direct or through a root arrow, is `ExprIr::This`.
    let activation = activation_graph(script).unwrap().activations()[0]
        .function()
        .clone();
    let owner = script
        .functions
        .iter()
        .find(|function| function.id == activation)
        .expect("activation owner");
    assert_eq!(count_this(&owner.body.statements), 0);
    for function in &script.functions {
        if function.id != activation && function.protocol == lila_ir::FunctionProtocolIr::Arrow {
            assert_eq!(count_this(&function.body.statements), 0, "{}", function.id);
        }
    }
}

#[test]
fn a_computed_specifier_is_served_from_the_referrers_resolution_rows() {
    let (_, program) = script_graph(
        &[
            (
                "entry.js",
                "import(['./a', '.js'].join('')); import.defer(globalThis.name);",
            ),
            ("a.js", "export const a = 1;"),
            ("b.js", "export const b = 2;"),
        ],
        &[(0, "./a.js"), (0, "./b.js")],
    );
    supported(&program);
    let graph = program.modules.as_ref().unwrap();
    let mut served: Vec<_> = graph
        .dynamic_components()
        .iter()
        .map(|component| {
            (
                component.request().specifier().to_string(),
                component.request().phase(),
                component.target(),
            )
        })
        .collect();
    served.sort();
    assert_eq!(
        served,
        [
            ("./a.js".to_string(), ImportPhaseIr::Evaluation, 1),
            ("./a.js".to_string(), ImportPhaseIr::Defer, 1),
            ("./b.js".to_string(), ImportPhaseIr::Evaluation, 2),
            ("./b.js".to_string(), ImportPhaseIr::Defer, 2),
        ]
    );
}

#[test]
fn a_script_importing_its_own_file_gets_a_separate_module_record() {
    let text = "globalThis.runs = (globalThis.runs || 0) + 1; import('./entry.js');";
    let key = ModuleKey::from_host("entry.js");
    let ParsedSource::Script(parsed) = parse(text, ParseOptions::script()).unwrap() else {
        panic!("Script parse goal");
    };
    let modules = vec![
        ModuleSourceIr::from_parsed_script(key.clone(), "file:///entry.js".into(), parsed),
        ModuleSourceIr::new(key.clone(), text.into(), "file:///entry.js".into()),
    ];
    let request = modules[0].module_requests().unwrap().remove(0);
    let sources = ModuleGraphSources {
        modules,
        entry: 0,
        resolutions: vec![(0, request, 1)],
    };
    let program = lower_script_graph(&sources);
    supported(&program);
    let graph = program.modules.as_ref().unwrap();
    assert_eq!(graph.units.len(), 2);
    // Only the Module Record is in the module map.
    assert_eq!(graph.keys.get(&key), Some(&1));
    assert_eq!(graph.dynamic_components()[0].target(), 1);
}

#[test]
fn a_dynamic_only_invalid_target_rejects_its_import_instead_of_the_script() {
    let (_, program) = script_graph(
        &[
            (
                "entry.js",
                "import('./invalid.js').catch(e => print(e.name)); import('./valid.js');",
            ),
            ("invalid.js", "invalid syntax!"),
            ("valid.js", "export const value = 1;"),
        ],
        &[],
    );
    supported(&program);
    let graph = program.modules.as_ref().unwrap();
    assert!(graph.link_errors.is_empty());
    assert!(graph
        .units
        .iter()
        .all(|unit| unit.record.key.as_str() != "invalid.js"));
}

#[test]
fn a_hashbang_script_keeps_its_comment_after_the_graph() {
    let (_, program) = script_graph(
        &[
            ("entry.js", "#!/usr/bin/env lila\nimport('./value.js');"),
            ("value.js", "export const value = 1;"),
        ],
        &[],
    );
    supported(&program);
}

#[test]
fn a_script_spelling_the_linker_prefix_is_an_explicit_gap() {
    let (_, program) = script_graph(
        &[
            (
                "entry.js",
                "var $lila$module$import$0 = 1; import('./value.js');",
            ),
            ("value.js", "export const value = 1;"),
        ],
        &[],
    );
    assert!(!program.is_wasm_supported());
    assert!(program
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("linker-reserved prefix")));
}
