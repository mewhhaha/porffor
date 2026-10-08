use lila_ir::{
    lower_module_graph, lower_script_graph, EarlyErrorCode, ExportName, ExprIr, JsonModuleValueIr,
    JsonValue, ModuleGraphSources, ModuleKey, ModuleKindIr, ModuleSourceIr, StatementIr, TypedExpr,
};

fn sources(entry: &str, data: &str, script: bool) -> ModuleGraphSources {
    let key = ModuleKey::from_host("entry.js");
    let entry = if script {
        let lila_front::ParsedSource::Script(parsed) =
            lila_front::parse(entry, lila_front::ParseOptions::script()).unwrap()
        else {
            panic!("Script")
        };
        ModuleSourceIr::from_parsed_script(key, "file:///entry.js".into(), parsed)
    } else {
        ModuleSourceIr::new(key, entry.into(), "file:///entry.js".into())
    };
    let requests = entry.module_requests().unwrap();
    ModuleGraphSources {
        realm_requests: Default::default(),
        modules: vec![
            entry,
            ModuleSourceIr::json(
                ModuleKey::from_host("data.json"),
                data.into(),
                "file:///data.json".into(),
            ),
        ],
        entry: 0,
        resolutions: requests
            .into_iter()
            .map(|request| (0, request, 1))
            .collect(),
    }
}

fn value_in_expression(expr: &TypedExpr) -> Option<&JsonModuleValueIr> {
    match &expr.expr {
        ExprIr::JsonModuleValue(value) => Some(value),
        ExprIr::AssignIdentifier { value, .. } => value_in_expression(value),
        ExprIr::EnvironmentIdentifier(identifier) => identifier
            .operation
            .operands()
            .find_map(value_in_expression),
        _ => None,
    }
}
fn value_in_statement(statement: &StatementIr) -> Option<&JsonModuleValueIr> {
    match statement {
        StatementIr::Expression(expr) | StatementIr::Lexical { init: expr, .. } => {
            value_in_expression(expr)
        }
        StatementIr::Block(block) => block.statements.iter().find_map(value_in_statement),
        StatementIr::LexicalBlock(items) => items.iter().find_map(value_in_statement),
        StatementIr::EmptyStatementCompletion(owner) => value_in_statement(owner.statement()),
        _ => None,
    }
}

#[test]
fn genuine_json_record_has_one_initialized_default_and_native_data_evaluation() {
    let sources = sources(
        "import value from './data.json' with { type: 'json' }; value;",
        r#"{"__proto__":1,"__proto__":2,"units":"\uD800","zero":-0}"#,
        false,
    );
    let program = lower_module_graph(&sources);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let graph = program.modules.as_ref().unwrap();
    let record = &graph.unit(1).record;
    assert_eq!(record.kind(), ModuleKindIr::Json);
    assert_eq!(graph.exported_names(1), vec![ExportName::new("default")]);
    assert!(record.requested_modules.is_empty());
    assert_eq!(record.environment.len(), 1);
    assert!(record.environment[0].initialized_before_evaluation);
    assert!(!record.environment[0].in_tdz_until_evaluated);
    let native = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find_map(|function| function.body.statements.iter().find_map(value_in_statement))
        .expect("actual JSON evaluation reaches the checked native operation");
    assert_eq!(native.module(), 1);
    let JsonValue::Object(entries) = native.value() else {
        panic!("JSON object")
    };
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].0, entries[1].0);
    assert!(matches!(&entries[2].1, JsonValue::String(units) if units == &[0xd800]));
    assert!(matches!(entries[3].1, JsonValue::Number(bits) if bits == (-0.0f64).to_bits()));
}

#[test]
fn json_named_import_type_mismatch_and_static_malformed_data_reject() {
    for entry in [
        "import { a } from './data.json' with { type: 'json' };",
        "import value from './data.json';",
    ] {
        assert!(!lower_module_graph(&sources(entry, "{\"a\":1}", false)).is_wasm_supported());
    }
    let malformed = lower_module_graph(&sources(
        "import value from './data.json' with { type: 'json' };",
        "[1,]",
        false,
    ));
    assert!(malformed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code() == Some(EarlyErrorCode::ModuleSyntax)));
}

#[test]
fn dynamic_malformed_json_and_source_phase_stay_import_job_rejections() {
    let dynamic = lower_script_graph(&sources(
        "import('./data.json', { with: { type: 'json' } }).catch(print);",
        "[1,]",
        true,
    ));
    assert!(dynamic.is_wasm_supported(), "{:?}", dynamic.diagnostics);
    let source = lower_module_graph(&sources(
        "import source value from './data.json' with { type: 'json' };",
        "null",
        false,
    ));
    assert!(source
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code() == Some(EarlyErrorCode::ModuleSourceUnavailable)));
}

#[test]
fn identical_bytes_do_not_merge_contradictory_record_kinds() {
    let mut graph = sources(
        "import value from './data.json' with { type: 'json' };",
        "0",
        false,
    );
    graph.modules.push(ModuleSourceIr::new(
        ModuleKey::from_host("data.json"),
        "0".into(),
        "file:///data.json".into(),
    ));
    assert!(!lower_module_graph(&graph).is_wasm_supported());
}
