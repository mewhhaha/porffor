use lila_ir::{
    lower_module_graph, BytesModuleValueIr, ExportName, ExprIr, ModuleGraphSources, ModuleKey,
    ModuleKindIr, ModuleRecordKey, ModuleSourceIr, StatementIr, TypedExpr,
};

fn sources(entry: &str, bytes: &[u8]) -> ModuleGraphSources {
    let entry = ModuleSourceIr::new(
        ModuleKey::from_host("entry.js"),
        entry.into(),
        "lila://entry".into(),
    );
    let requests = entry.module_requests().unwrap();
    ModuleGraphSources {
        realm_requests: Default::default(),
        entry: 0,
        modules: vec![
            entry,
            ModuleSourceIr::bytes(
                ModuleKey::from_host("raw.js"),
                bytes.to_vec(),
                "lila://raw".into(),
            ),
        ],
        resolutions: requests
            .into_iter()
            .map(|request| (0, request, 1))
            .collect(),
    }
}

fn native_expression(expr: &TypedExpr) -> Option<&BytesModuleValueIr> {
    match &expr.expr {
        ExprIr::BytesModuleValue(value) => Some(value),
        ExprIr::AssignIdentifier { value, .. } => native_expression(value),
        ExprIr::EnvironmentIdentifier(identifier) => {
            identifier.operation.operands().find_map(native_expression)
        }
        _ => None,
    }
}
fn native_statement(statement: &StatementIr) -> Option<&BytesModuleValueIr> {
    match statement {
        StatementIr::Expression(expr) | StatementIr::Lexical { init: expr, .. } => {
            native_expression(expr)
        }
        StatementIr::Block(block) => block.statements.iter().find_map(native_statement),
        StatementIr::LexicalBlock(items) => items.iter().find_map(native_statement),
        StatementIr::EmptyStatementCompletion(owner) => native_statement(owner.statement()),
        _ => None,
    }
}

#[test]
fn raw_data_has_one_native_default_without_a_text_parse() {
    for bytes in [&[0, 255, 128, 65][..], &[][..]] {
        let input = sources(
            "import raw from './raw.js' with {type:'bytes'}; raw;",
            bytes,
        );
        assert_eq!(input.modules[1].payload_bytes(), bytes);
        assert!(input.modules[1].module_requests().unwrap().is_empty());
        let program = lower_module_graph(&input);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let graph = program.modules.as_ref().unwrap();
        assert_eq!(graph.unit(1).record.kind(), ModuleKindIr::Bytes);
        assert_eq!(graph.exported_names(1), vec![ExportName::new("default")]);
        assert!(graph.unit(1).record.requested_modules.is_empty());
        let native = program
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .find_map(|function| function.body.statements.iter().find_map(native_statement))
            .unwrap();
        assert_eq!(native.module(), 1);
        assert_eq!(native.bytes(), bytes);
    }
}

#[test]
fn bytes_records_require_the_exact_type_and_export_only_default() {
    for entry in [
        "import {missing} from './raw.js' with {type:'bytes'};",
        "import raw from './raw.js';",
        "import raw from './raw.js' with {type:'json'};",
        "import raw from './raw.js' with {type:'bytes',extra:'no'};",
    ] {
        assert!(
            !lower_module_graph(&sources(entry, &[255])).is_wasm_supported(),
            "{entry}"
        );
    }
}

#[test]
fn mixed_locator_records_are_distinct_but_repeated_bytes_are_canonical() {
    let mut input = sources(
        "import raw from './raw.js' with {type:'bytes'};",
        b"export default 1;",
    );
    input.modules.push(input.modules[1].clone());
    input.modules.push(ModuleSourceIr::new(
        ModuleKey::from_host("raw.js"),
        "export default 1;".into(),
        "lila://raw".into(),
    ));
    let program = lower_module_graph(&input);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.unwrap();
    assert_eq!(graph.units.len(), 3);
    let locator = ModuleKey::from_host("raw.js");
    assert_ne!(
        graph.keys[&ModuleRecordKey::new(locator.clone(), ModuleKindIr::Bytes)],
        graph.keys[&ModuleRecordKey::new(locator.clone(), ModuleKindIr::SourceText)]
    );
    input.modules.push(ModuleSourceIr::bytes(
        locator,
        vec![255],
        "lila://raw".into(),
    ));
    assert!(!lower_module_graph(&input).is_wasm_supported());
}
