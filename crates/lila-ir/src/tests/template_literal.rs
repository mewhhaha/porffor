#[test]
fn tagged_template_lowers_its_tag_and_template_site() {
    let program = lower_script("(function(template) { return template; })`a${1}b`;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(expression) = &script.body.statements[0] else {
        panic!("tagged template should lower as an expression statement");
    };
    let ExprIr::CallIndirect { callee, args, .. } = &expression.expr else {
        panic!("plain tagged template should lower as an indirect call");
    };
    assert!(matches!(callee.expr, ExprIr::FunctionValue(_)));
    let ExprIr::TemplateObject(template) = &args[0].expr else {
        panic!("first tagged-template argument should identify its template site");
    };
    assert_eq!(
        template.cooked,
        vec![Some("a".to_string()), Some("b".to_string())]
    );
    assert_eq!(template.raw, vec!["a".to_string(), "b".to_string()]);
}
