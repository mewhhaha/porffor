use lila_front::{parse, ParseGoal, ParseOptions, ParsedSource};
use lila_ir::{
    lower, lower_module_graph, lower_script_graph, ExprIr, ModuleGraphSources, ModuleKey,
    ModuleNamespaceModeIr, ModuleSourceIr, ScriptIr, StatementIr, TypedExpr, ValueKind,
};

fn graph(files: &[(&str, &str)], entry_goal: ParseGoal) -> ScriptIr {
    let modules: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(index, (key, source))| {
            let key = ModuleKey::from_host(*key);
            let url = format!("file:///{}", key.as_str());
            if index == 0 && entry_goal == ParseGoal::Script {
                let ParsedSource::Script(parsed) =
                    parse(*source, ParseOptions::script()).expect("entry parses")
                else {
                    panic!("Script entry");
                };
                ModuleSourceIr::from_parsed_script(key, url, parsed)
            } else {
                ModuleSourceIr::new(key, (*source).into(), url)
            }
        })
        .collect();
    let mut resolutions = Vec::new();
    for (referrer, module) in modules.iter().enumerate() {
        for request in module.module_requests().expect("fixture parses") {
            let target = files
                .iter()
                .position(|(key, _)| *key == request.specifier().trim_start_matches("./"))
                .expect("fixture dependency");
            resolutions.push((referrer as u32, request, target as u32));
        }
    }
    let sources = ModuleGraphSources {
        modules,
        entry: 0,
        resolutions,
    };
    let program = match entry_goal {
        ParseGoal::Script => lower_script_graph(&sources),
        ParseGoal::Module => lower_module_graph(&sources),
    };
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("linked Script IR")
}

fn binding_initializers<'a>(
    statements: &'a [StatementIr],
    out: &mut Vec<(&'a str, &'a TypedExpr)>,
) {
    for statement in statements {
        match statement {
            StatementIr::Lexical { name, init, .. } => out.push((name, init)),
            StatementIr::Var(declarations) => {
                for declaration in declarations {
                    if let Some(init) = &declaration.init {
                        out.push((&declaration.name, init));
                    }
                }
            }
            StatementIr::LexicalBlock(statements) => binding_initializers(statements, out),
            StatementIr::Block(block) => {
                binding_initializers(&block.statements, out);
            }
            StatementIr::ModuleUnitOnce { block, .. } => {
                binding_initializers(&block.statements, out);
            }
            _ => {}
        }
    }
}

fn initializers(script: &ScriptIr) -> Vec<(&str, &TypedExpr)> {
    let mut out = Vec::new();
    binding_initializers(&script.body.statements, &mut out);
    for function in &script.functions {
        binding_initializers(&function.body.statements, &mut out);
    }
    out
}

fn namespace_tables(script: &ScriptIr) -> Vec<(ModuleNamespaceModeIr, &[TypedExpr])> {
    initializers(script)
        .into_iter()
        .filter_map(|(_, init)| {
            let ExprIr::ModuleNamespace { mode, exports } = &init.expr else {
                return None;
            };
            assert_eq!(init.kind, ValueKind::Object);
            assert!(
                init.heap_shape.is_none(),
                "namespace is not an ordinary shaped object"
            );
            let ExprIr::ArrayLiteral(elements) = &exports.expr else {
                panic!("private export table");
            };
            for reader in elements.iter().skip(2).step_by(2) {
                let ExprIr::FunctionValue(id) = &reader.expr else {
                    panic!("export retains a live reader: {reader:?}");
                };
                assert!(script.functions.iter().any(|function| function.id == *id));
            }
            Some((*mode, elements.as_slice()))
        })
        .collect()
}

#[test]
fn repeated_self_imports_construct_one_namespace_with_live_readers() {
    let script = graph(&[("entry.js", "import * as first from './entry.js'; import * as second from './entry.js'; export let value = 1; print(first === second, first.value); value = 2; print(second.value);")], ParseGoal::Module);
    let tables = namespace_tables(&script);
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].0, ModuleNamespaceModeIr::Eager);
    assert_eq!(tables[0].1.len(), 3);
    assert_eq!(tables[0].1[0].expr, ExprIr::Undefined);
    assert_eq!(tables[0].1[1].expr, ExprIr::String("value".into()));
}

#[test]
fn ordinary_arrays_with_namespace_spelling_keep_array_semantics() {
    let source =
        "const $m0$namespace = [void 0, 'value', () => 1]; print(Array.isArray($m0$namespace));";
    let parsed = parse(source, ParseOptions::script()).expect("source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("Script IR");
    assert!(namespace_tables(&script).is_empty());
    let ordinary = initializers(&script)
        .into_iter()
        .find(|(name, _)| *name == "$m0$namespace")
        .expect("ordinary array")
        .1;
    assert_eq!(ordinary.kind, ValueKind::Array);
    assert!(matches!(ordinary.expr, ExprIr::ArrayLiteral(_)));
    assert!(ordinary.heap_shape.is_some());

    let linked = graph(&[("entry.js", "import * as ns from './value.js'; const ordinary = [void 0, 'value', () => 1]; print(ns.value, ordinary.length);"), ("value.js", "export let value = 1;")], ParseGoal::Module);
    assert_eq!(namespace_tables(&linked).len(), 1);
    let ordinary = initializers(&linked)
        .into_iter()
        .find(|(name, _)| *name == "ordinary")
        .expect("linked ordinary array")
        .1;
    assert_eq!(ordinary.kind, ValueKind::Array);
}

#[test]
fn export_tables_keep_utf16_order_including_numeric_strings() {
    let script = graph(&[("entry.js", "import * as ns from './value.js'; print(Reflect.ownKeys(ns));"), ("value.js", "const value = 1; export { value as '2', value as '10', value as '\\uFF3A', value as '\\u{10000}' };")], ParseGoal::Module);
    let tables = namespace_tables(&script);
    assert_eq!(tables.len(), 1);
    let keys = tables[0]
        .1
        .iter()
        .skip(1)
        .step_by(2)
        .map(|key| {
            let ExprIr::String(key) = &key.expr else {
                panic!("export name")
            };
            key.as_str()
        })
        .collect::<Vec<_>>();
    assert_eq!(keys, ["10", "2", "\u{10000}", "\u{FF3A}"]);
}

#[test]
fn deferred_namespaces_retain_an_evaluator_separate_from_export_readers() {
    let script = graph(
        &[
            (
                "entry.js",
                "import defer * as ns from './value.js'; print(ns.value);",
            ),
            ("value.js", "export let value = 1;"),
        ],
        ParseGoal::Module,
    );
    let tables = namespace_tables(&script);
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].0, ModuleNamespaceModeIr::Deferred);
    let ExprIr::FunctionValue(evaluator) = &tables[0].1[0].expr else {
        panic!("deferred evaluator")
    };
    assert!(script
        .functions
        .iter()
        .any(|function| function.id == *evaluator));
    assert_ne!(tables[0].1[0].expr, tables[0].1[2].expr);
}

#[test]
fn async_and_dynamic_import_wrappers_retain_trusted_namespace_spans() {
    for (entry_goal, entry) in [
        (
            ParseGoal::Module,
            "// 🦀é\r\nimport * as ns from './value.js'; await 0; print(ns.value);",
        ),
        (
            ParseGoal::Script,
            "// 🦀é\r\nimport('./value.js').then(ns => print(ns.value));",
        ),
    ] {
        let script = graph(
            &[
                ("entry.js", entry),
                ("value.js", "// 🦀é\r\nexport let value = 1;"),
            ],
            entry_goal,
        );
        let tables = namespace_tables(&script);
        assert!(!tables.is_empty(), "{entry_goal:?}");
        for (mode, table) in tables {
            assert_eq!(mode, ModuleNamespaceModeIr::Eager);
            assert_eq!(table.len(), 3);
            assert_eq!(table[1].expr, ExprIr::String("value".into()));
        }
    }
}
