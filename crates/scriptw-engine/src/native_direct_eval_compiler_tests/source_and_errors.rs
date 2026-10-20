use super::*;

#[test]
fn native_direct_syntax_uses_real_caller_grammar_and_intrinsic_error_identity() {
    let invalid = "new.target;";
    let source = format!(
        r#"
var Original=SyntaxError;SyntaxError=function(){{throw 'poisoned constructor';}};
try{{eval({text});}}catch(error){{print(error instanceof Original);print(Object.getPrototypeOf(error)===Original.prototype);}}
"#,
        text = converted_text(invalid)
    );
    assert_trace(&source, &["true", "true"], &[units(invalid)], (1, 0, 1));
}

#[test]
fn native_direct_source_keeps_exact_lone_utf16_units_and_non_string_identity() {
    let suffix = "'.charCodeAt(0);";
    let source_units = [vec![0x27, 0xd800], units(suffix)].concat();
    let expression = format!(
        "String.fromCharCode({})",
        source_units
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let source = format!(
        r#"
var token={{toString(){{throw 'not converted';}}}};
print(eval(token)===token);print(eval({expression}));
"#
    );
    assert_trace(&source, &["true", "55296"], &[source_units], (1, 1, 0));
}

#[test]
fn unused_external_import_bodies_do_not_reject_native_direct_eval() {
    let body = "function unused(specifier,options){return import(specifier,options);} 42;";
    let source = format!("print(eval({}));", converted_text(body));
    assert_trace(&source, &["42"], &[units(body)], (1, 1, 0));
}

#[test]
fn external_import_operand_throws_precede_its_runtime_capability_boundary() {
    let body = "import((function(){throw sentinel;})());";
    let source = format!(
        r#"
var sentinel={{}};
try{{eval({text});}}catch(error){{print(error===sentinel);}}
"#,
        text = converted_text(body)
    );
    assert_trace(&source, &["true"], &[units(body)], (1, 1, 0));
}

#[test]
fn executed_external_import_keeps_its_distinct_uncatchable_capability_error() {
    configure_compilation_jobs(1).unwrap();
    let source = format!("eval({});", converted_text("import('./unavailable.js');"));
    let mut engine = Engine::new(RealmBuilder::new().build());
    let error = engine
        .observe_script(
            &source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_err();
    assert_eq!(
        error.runtime_unavailable_capabilities(),
        vec![RuntimeUnavailableCapability::ExternalCallerImport]
    );
    assert!(error.runtime_dynamic_source_operations().is_empty());
    assert!(error.parse_diagnostic().is_none());
}
