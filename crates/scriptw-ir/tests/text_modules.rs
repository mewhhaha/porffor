use scriptw_ir::{
    lower_module_graph, ExportName, ExprIr, ModuleGraphSources, ModuleKey, ModuleKindIr,
    ModuleRecordKey, ModuleSourceIr, StatementIr, TypedExpr,
};

fn sources(entry: &str, text: &str) -> ModuleGraphSources {
    let entry = ModuleSourceIr::new(
        ModuleKey::from_host("entry.js"),
        entry.into(),
        "scriptw://entry".into(),
    );
    let requests = entry.module_requests().unwrap();
    ModuleGraphSources {
        realm_requests: Default::default(),
        entry: 0,
        modules: vec![
            entry,
            ModuleSourceIr::text(
                ModuleKey::from_host("raw.js"),
                text.into(),
                "scriptw://raw".into(),
            ),
        ],
        resolutions: requests
            .into_iter()
            .map(|request| (0, request, 1))
            .collect(),
    }
}

fn contains_string(expr: &TypedExpr, expected: &str) -> bool {
    match &expr.expr {
        ExprIr::String(text) => text == expected,
        ExprIr::AssignIdentifier { value, .. } => contains_string(value, expected),
        ExprIr::EnvironmentIdentifier(identifier) => identifier
            .operation
            .operands()
            .any(|operand| contains_string(operand, expected)),
        _ => false,
    }
}
fn contains_statement(statement: &StatementIr, expected: &str) -> bool {
    match statement {
        StatementIr::Expression(expr) | StatementIr::Lexical { init: expr, .. } => {
            contains_string(expr, expected)
        }
        StatementIr::Block(block) => block
            .statements
            .iter()
            .any(|statement| contains_statement(statement, expected)),
        StatementIr::LexicalBlock(items) => items
            .iter()
            .any(|statement| contains_statement(statement, expected)),
        StatementIr::EmptyStatementCompletion(owner) => {
            contains_statement(owner.statement(), expected)
        }
        _ => false,
    }
}

#[test]
fn text_defaults_retain_exact_strings_without_javascript_or_json_parsing() {
    for text in [
        "",
        "invalid syntax! import './absent.js';",
        "{not JSON}",
        "\u{feff}Ā\0𝄞\n",
    ] {
        let input = sources("import raw from './raw.js' with {type:'text'}; raw;", text);
        assert_eq!(input.modules[1].payload_bytes(), text.as_bytes());
        assert_eq!(input.modules[1].module_requests(), Some(Vec::new()));
        let program = lower_module_graph(&input);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let graph = program.modules.as_ref().unwrap();
        let record = &graph.unit(1).record;
        assert_eq!(record.kind(), ModuleKindIr::Text);
        assert_eq!(graph.exported_names(1), vec![ExportName::new("default")]);
        assert_eq!(record.environment.len(), 1);
        assert!(record.environment[0].initialized_before_evaluation);
        assert!(!record.environment[0].in_tdz_until_evaluated);
        assert!(record.requested_modules.is_empty());
        assert!(
            program
                .script
                .as_ref()
                .unwrap()
                .functions
                .iter()
                .any(|function| {
                    function
                        .body
                        .statements
                        .iter()
                        .any(|statement| contains_statement(statement, text))
                }),
            "the synthetic evaluation must emit the original String"
        );
    }
}

#[test]
fn text_records_are_distinct_at_one_locator_and_require_exact_attributes() {
    let mut input = sources(
        "import raw from './raw.js' with {type:'text'};",
        "export default 42;",
    );
    input.modules.push(input.modules[1].clone());
    input.modules.push(ModuleSourceIr::new(
        ModuleKey::from_host("raw.js"),
        "export default 42;".into(),
        "scriptw://raw".into(),
    ));
    let program = lower_module_graph(&input);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.unwrap();
    assert_eq!(graph.units.len(), 3);
    let key = ModuleKey::from_host("raw.js");
    assert_ne!(
        graph.keys[&ModuleRecordKey::new(key.clone(), ModuleKindIr::Text)],
        graph.keys[&ModuleRecordKey::new(key, ModuleKindIr::SourceText)]
    );
    for entry in [
        "import raw from './raw.js';",
        "import raw from './raw.js' with {type:'json'};",
        "import raw from './raw.js' with {type:'bytes'};",
        "import raw from './raw.js' with {type:'text',extra:'no'};",
        "import {missing} from './raw.js' with {type:'text'};",
    ] {
        assert!(
            !lower_module_graph(&sources(entry, "invalid syntax!")).is_wasm_supported(),
            "{entry}"
        );
    }
}
