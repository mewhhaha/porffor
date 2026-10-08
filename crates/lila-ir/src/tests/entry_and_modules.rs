#[test]
fn lowers_simple_script_ir() {
    let program = lower_script("let x = 40; const y = 2; x + y;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.body.statements.len(), 3);
    assert_eq!(script.result_kind(), ValueKind::Number);
}

#[test]
fn lowers_no_import_module_export_declaration() {
    let program = lower_module("export const value = 1; value;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.result_kind(), ValueKind::Undefined);
}

#[test]
fn module_root_this_is_statically_undefined_through_arrow_chains() {
    let program = lower_module("const direct = () => this; const nested = () => () => this; this;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.protocol == FunctionProtocolIr::ModuleActivation)
        .expect("private Module lexical owner");
    let StatementIr::Expression(root_this) = owner
        .body
        .statements
        .iter()
        .rev()
        .find(|statement| !matches!(statement, StatementIr::Empty))
        .expect("module root this should remain an expression")
    else {
        panic!("expected module root this expression");
    };
    assert!(matches!(root_this.expr, ExprIr::Undefined));
    assert_eq!(script.top_level_this_uses, 0);

    let arrows = script
        .functions
        .iter()
        .filter(|function| {
            function.is_nested
                && function.protocol.flavor() == FunctionFlavor::Arrow
                && matches!(
                    &function.to_string_representation,
                    CallableToStringRepresentation::ExactSource(source)
                        if matches!(source.as_str(), "() => this" | "() => () => this")
                )
        })
        .collect::<Vec<_>>();
    assert_eq!(arrows.len(), 3);
    assert_eq!(
        arrows
            .iter()
            .filter(|function| {
                function_return(function)
                    .is_some_and(|value| matches!(value.expr, ExprIr::Undefined))
            })
            .count(),
        2,
        "direct and innermost root arrows should return undefined"
    );
    assert!(arrows
        .iter()
        .all(|function| !function.captures_lexical_this));
}

#[test]
fn module_function_activations_keep_runtime_this() {
    let program =
        lower_module("function ordinary() { return this; } function make() { return () => this; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let ordinary = script
        .functions
        .iter()
        .find(|function| function.name == "ordinary")
        .expect("ordinary function should be lowered");
    assert!(matches!(
        function_return(ordinary).map(|value| &value.expr),
        Some(ExprIr::This)
    ));
    let arrow = script
        .functions
        .iter()
        .find(|function| {
            function.protocol.flavor() == FunctionFlavor::Arrow && function.captures_lexical_this
        })
        .expect("lexical arrow should be lowered");
    assert!(arrow.captures_lexical_this);
    assert!(matches!(
        function_return(arrow).map(|value| &value.expr),
        Some(ExprIr::This)
    ));
}

#[test]
fn script_root_this_stays_global_through_arrow_chains() {
    let program = lower_script("this; const direct = () => this; const nested = () => () => this;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(root_this) = &script.body.statements[0] else {
        panic!("expected Script root this expression");
    };
    assert!(matches!(root_this.expr, ExprIr::This));
    assert_eq!(script.top_level_this_uses, 3);
    assert_eq!(
        script
            .functions
            .iter()
            .filter(|function| {
                function.protocol.flavor() == FunctionFlavor::Arrow
                    && function_return(function)
                        .is_some_and(|value| matches!(value.expr, ExprIr::This))
            })
            .count(),
        2
    );
}

/// A single-source `lower` has no host loader, so it resolves no specifier.
/// The honest report is that the *request* did not resolve, not that imports
/// are unsupported — `modules::link` links them once a loader supplies the
/// dependency. The assertion previously looked for the string
/// "module imports", which no diagnostic in the crate has ever produced.
#[test]
fn rejects_an_import_whose_specifier_no_loader_resolved() {
    let program = lower_module("import value from './dep.js'; value;");
    assert!(!program.is_wasm_supported());
    assert!(
        program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unresolved module request")
            && diagnostic.message.contains("./dep.js")),
        "got {:?}",
        program.diagnostics
    );
}

#[test]
fn allows_non_prototype_proto_property_forms() {
    let program = lower_script(r#"({ __proto__() { return 1; }, ["__proto__"]: 2 });"#);
    assert!(
        program
            .diagnostics
            .iter()
            .all(|diagnostic| { diagnostic.code() != Some(EarlyErrorCode::ObjectDuplicateProto) }),
        "diagnostics: {:?}",
        program.diagnostics
    );
}

#[test]
fn lowers_assignment_and_if_ir() {
    let program = lower_script("let x = 0; if (!x) { x = 5; } x;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.result_kind(), ValueKind::Number);
    assert!(matches!(script.body.statements[1], StatementIr::If { .. }));
    assert!(program.ir_summary().contains("ifs=1"));
    assert!(program.ir_summary().contains("assigns=1"));
}
