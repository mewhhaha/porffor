use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, lower_module_graph, ExprIr, FunctionProtocolIr, ModuleGraphSources, ModuleKey,
    ModuleSourceIr, ScriptIr, StatementIr,
};

fn module(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::module()).expect("module parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("module has Script IR")
}

fn graph(files: &[(&str, &str)]) -> ScriptIr {
    let modules: Vec<_> = files
        .iter()
        .map(|(key, source)| {
            ModuleSourceIr::new(
                ModuleKey::from_host(*key),
                (*source).into(),
                format!("file:///{key}"),
            )
        })
        .collect();
    let mut resolutions = Vec::new();
    for (referrer, module) in modules.iter().enumerate() {
        for request in module.module_requests().expect("module parses") {
            let target = files
                .iter()
                .position(|(key, _)| *key == request.specifier().trim_start_matches("./"))
                .expect("fixture dependency");
            resolutions.push((referrer as u32, request, target as u32));
        }
    }
    let program = lower_module_graph(&ModuleGraphSources {
        modules,
        entry: 0,
        resolutions,
    });
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("module graph has Script IR")
}

fn class_names(script: &ScriptIr) -> Vec<&str> {
    script
        .functions
        .iter()
        .filter(|function| function.protocol == FunctionProtocolIr::ClassConstructor)
        .map(|function| function.name.as_str())
        .collect()
}

#[test]
fn default_class_display_name_is_independent_of_its_merged_storage() {
    fn class_binding(statements: &[StatementIr]) -> Option<(&str, &lila_ir::ClassDefinitionIr)> {
        statements.iter().find_map(|statement| match statement {
            StatementIr::LexicalBlock(statements) => class_binding(statements),
            StatementIr::Block(block) => class_binding(&block.statements),
            StatementIr::Lexical { name, init, .. } => match &init.expr {
                ExprIr::ClassDefinition(class) => Some((name.as_str(), class.as_ref())),
                _ => None,
            },
            _ => None,
        })
    }
    let script = module("export default class { static value = this.name; }");
    assert_eq!(class_names(&script), ["default"]);
    let definition =
        class_binding(&script.body.statements).expect("default export class declaration");
    assert_eq!(definition.0, "$d0$");
    assert_eq!(definition.1.name.as_deref(), Some("default"));
    assert!(!definition.1.element_plan.static_elements.is_empty());
}

#[test]
fn anonymous_function_protocols_use_named_evaluation_before_lowering() {
    for expression in [
        "function () {}",
        "function* () {}",
        "async function () {}",
        "async function* () {}",
        "(() => 1)",
        "(async () => 1)",
        "(function () {})",
    ] {
        let script = module(&format!("export default {expression};"));
        assert_eq!(
            script
                .functions
                .iter()
                .filter(|function| function.name == "default")
                .count(),
            1,
            "{expression}"
        );
        assert!(
            !script
                .functions
                .iter()
                .any(|function| function.name == "$d0$"),
            "{expression}"
        );
    }
}

#[test]
fn explicit_and_nested_same_spelled_definitions_are_not_default_exports() {
    let script =
        module("export default class { static { let $d0$ = class {}; print($d0$.name); } }");
    let names = class_names(&script);
    assert_eq!(names.iter().filter(|name| **name == "default").count(), 1);
    assert_eq!(names.iter().filter(|name| **name == "$d0$").count(), 1);
    assert_eq!(
        class_names(&module("export default (class $d0$ {});")),
        ["$d0$"]
    );
    assert_eq!(
        class_names(&module("export default class Named {}")),
        ["Named"]
    );
    assert_eq!(
        class_names(&module("class Named {} export { Named as default };")),
        ["Named"]
    );
    assert_eq!(class_names(&module("export default (0, class {});")), [""]);
    assert_eq!(
        class_names(&module("export default (0, class { constructor() {} });")),
        [""]
    );
    assert_eq!(class_names(&module("const classes = [class {}];")), [""]);
    let function = module("export default (function $d0$() {});");
    assert!(function
        .functions
        .iter()
        .any(|function| function.name == "$d0$"));
}

#[test]
fn utf16_crlf_and_multi_unit_offsets_identify_the_actual_default_definition() {
    let script = graph(&[
        ("entry.js", "// 🦀é\r\nimport value from './value.js'; print(value.name);"),
        ("value.js", "const text = '🦀é';\r\nvoid import.meta;\r\nexport default (class { static value = this.name; });"),
    ]);
    assert_eq!(class_names(&script), ["default"]);
}

#[test]
fn async_and_deferred_wrappers_relocate_default_definition_spans() {
    for files in [
        [
            (
                "entry.js",
                "import value from './value.js'; print(value.name);",
            ),
            ("value.js", "await 0; export default class {}"),
        ],
        [
            (
                "entry.js",
                "import defer * as ns from './value.js'; print(ns.default.name);",
            ),
            ("value.js", "export default class {}"),
        ],
    ] {
        let script = graph(&files);
        // Wrapper propagation may retain multiple generated records for the
        // same constructor. Its execution identity and display name are fixed.
        let constructors = script
            .functions
            .iter()
            .filter(|function| function.protocol == FunctionProtocolIr::ClassConstructor)
            .map(|function| (function.id.as_str(), function.name.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(constructors.len(), 1);
        assert_eq!(constructors.first().expect("one constructor").1, "default");
    }
}

fn initialized_binding<'a>(
    statements: &'a [StatementIr],
    binding: &str,
) -> Vec<&'a lila_ir::TypedExpr> {
    statements
        .iter()
        .flat_map(|statement| match statement {
            StatementIr::LexicalBlock(statements) => initialized_binding(statements, binding),
            StatementIr::Block(block) => initialized_binding(&block.statements, binding),
            StatementIr::Lexical { name, init, .. } if name == binding => vec![init],
            StatementIr::Var(declarations) => declarations
                .iter()
                .filter(|declaration| declaration.name == binding)
                .filter_map(|declaration| declaration.init.as_ref())
                .collect(),
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn hoistable_default_protocols_initialize_the_storage_binding_before_evaluation() {
    for definition in [
        "function () { return 23; }",
        "function* () { yield 23; }",
        "async function () { return 23; }",
        "async function* () { yield 23; }",
    ] {
        let script = module(&format!("print('body'); export default {definition}"));
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "default")
            .expect("default callable");
        assert!(!function.is_expression, "{definition}");
        assert_eq!(function.to_string_representation.materialize(), definition);
        assert_eq!(
            script
                .global_bindings
                .get("$d0$")
                .expect("merged binding")
                .initializer,
            lila_ir::GlobalPropertyInitializerIr::SourceFunction(function.id.clone())
        );
        assert!(
            initialized_binding(&script.body.statements, "$d0$").is_empty(),
            "the source export must not create a second function: {definition}"
        );
    }
}

#[test]
fn expression_defaults_retain_lexical_initialization_at_the_export_statement() {
    for definition in [
        "function () {}",
        "function* () {}",
        "async function () {}",
        "async function* () {}",
    ] {
        let script = module(&format!("print('body'); export default ({definition});"));
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "default")
            .expect("default callable");
        assert!(function.is_expression, "{definition}");
        assert!(script
            .global_bindings
            .lexical_bindings()
            .contains_key("$d0$"));
        assert!(script.global_bindings.get("$d0$").is_none());
        let initializers = initialized_binding(&script.body.statements, "$d0$");
        assert_eq!(initializers.len(), 1, "{definition}");
        assert_eq!(
            initializers[0].expr,
            ExprIr::FunctionValue(function.id.clone())
        );
    }
}

#[test]
fn wrapper_owned_default_declarations_use_the_wrappers_function_instantiation() {
    for files in [
        [
            (
                "entry.js",
                "import value from './value.js'; print(value.name);",
            ),
            (
                "value.js",
                "await 0; export default function () { return 23; }",
            ),
        ],
        [
            (
                "entry.js",
                "import defer * as ns from './value.js'; print(ns.default.name);",
            ),
            ("value.js", "export default function () { return 23; }"),
        ],
    ] {
        let script = graph(&files);
        let default_ids = script
            .functions
            .iter()
            .filter(|function| function.name == "default")
            .map(|function| function.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(default_ids.len(), 1);
        let default_id = *default_ids.first().expect("default callable");
        assert!(script
            .global_bindings
            .get("$d1$")
            .is_none_or(|binding| !matches!(
                binding.initializer,
                lila_ir::GlobalPropertyInitializerIr::SourceFunction(_)
            )));
        let owners = script
            .functions
            .iter()
            .filter_map(|function| {
                let initializers = initialized_binding(&function.body.statements, "$d1$");
                if initializers.is_empty() {
                    return None;
                }
                assert_eq!(initializers.len(), 1, "wrapper initializes once");
                assert_eq!(
                    initializers[0].expr,
                    ExprIr::FunctionValue(default_id.into())
                );
                // Context specialization compiles another body for the same
                // lexical owner; every generated body must initialize once.
                let owner_id = function
                    .id
                    .split_once("$exact_helper_context$")
                    .map_or(function.id.as_str(), |(owner, _)| owner);
                let owner = script
                    .functions
                    .iter()
                    .find(|candidate| candidate.id == owner_id)
                    .expect("specialized wrapper retains its source owner");
                assert_eq!(
                    function.to_string_representation,
                    owner.to_string_representation
                );
                assert_eq!(function.protocol, owner.protocol);
                Some(owner_id)
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            owners.len(),
            1,
            "only the existing wrapper owns instantiation"
        );
        assert!(!owners.contains(default_id));
    }
}
