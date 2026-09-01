const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CALL_SOURCE: &str = include_str!("../../lila-ir/src/lowering/with_environment_call.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_with_environment_identifier_call.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/with-environment-identifier-call-reference.md"
);
const WITNESS_PATH: &str = "language/expressions/call/with-base-obj.js";
const WITNESS: &str =
    include_str!("../../../test262/vendor/test262/test/language/expressions/call/with-base-obj.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_index = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier marker: {earlier}"));
    let later_index = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later marker: {later}"));
    assert!(
        earlier_index < later_index,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn noncopy_plan_owns_the_only_callee_withbaseobject_product() {
    assert!(REFERENCE_SOURCE.contains(
        "#[derive(Debug)]\n#[must_use = \"a with-environment identifier-call Reference must be consumed by Call\"]\npub(crate) struct WithEnvironmentIdentifierCallReferencePlan {"
    ));
    assert!(!REFERENCE_SOURCE.contains(
        "#[derive(Debug, Clone)]\npub(crate) struct WithEnvironmentIdentifierCallReferencePlan"
    ));
    assert!(!REFERENCE_SOURCE.contains("impl Clone for WithEnvironmentIdentifierCallReferencePlan"));
    assert!(!REFERENCE_SOURCE.contains("impl Copy for WithEnvironmentIdentifierCallReferencePlan"));
    assert_eq!(
        REFERENCE_SOURCE
            .matches("pub(crate) fn into_identifier_call_plan(")
            .count(),
        1
    );

    let constructor = bounded(
        REFERENCE_SOURCE,
        "    pub(crate) fn into_identifier_call_plan(",
        "/// A non-empty ResolveBinding chain whose only result is an identifier call.",
    );
    assert!(constructor.contains("self.into_reference_plan("));
    assert!(constructor.contains("WithEnvironmentIdentifierCallReferencePlan {"));

    let selected = bounded(
        REFERENCE_SOURCE,
        "    fn call_or_else(",
        "    fn put_value_or_else(",
    );
    for marker in [
        "binding_object.binding_visible(referenced_name, unscopables_binding)",
        "binding_object\n            .clone()\n            .get_value(referenced_name, strictness)",
        "let receiver = binding_object.read();",
        "ExprIr::CallIndirect {",
        "callee: Box::new(callee)",
        "this_arg: Some(Box::new(receiver))",
        "args: args.to_vec()",
        "condition: Box::new(binding_visible)",
        "then_expr: Box::new(selected)",
        "else_expr: Box::new(fallback)",
    ] {
        assert!(
            selected.contains(marker),
            "missing selected call marker: {marker}"
        );
    }
    assert_before(selected, "let binding_visible =", "let callee =");
    assert_before(selected, "let callee =", "let receiver =");
    assert_before(selected, "let receiver =", "ExprIr::CallIndirect {");

    let consumer = bounded(
        REFERENCE_SOURCE,
        "impl WithEnvironmentIdentifierCallReferencePlan {",
        "/// A non-empty ResolveBinding chain for an identifier Reference inside `with`.",
    );
    assert!(consumer.contains(
        "pub(crate) fn call(self, args: Vec<TypedExpr>, fallback: TypedExpr) -> TypedExpr"
    ));
    assert!(consumer.contains("for environment in outer"));
    assert!(consumer.contains("environment.call_or_else("));
    assert!(consumer.contains("innermost.call_or_else("));
}

#[test]
fn lowerer_intercepts_before_folds_and_keeps_fallback_runtime_authoritative() {
    let call_entry = bounded(
        LOWERING_SOURCE,
        "        // Resolve a direct identifier through any preceding Object",
        "        if let Some(generator) = generator_expression_callee(callee) {",
    );
    assert!(call_entry.contains("self.lower_with_environment_identifier_call(callee, args)"));
    assert!(call_entry.contains("return call;"));

    let lower = bounded(
        CALL_SOURCE,
        "    pub(super) fn lower_with_environment_identifier_call(",
        "    /// Consume the already located fallback into a fresh run-time callee read.",
    );
    for marker in [
        "let Expression::Identifier(identifier) = callee else",
        "if name == \"eval\"",
        "let fallback_reference = self.locate_identifier_reference(&name);",
        ".select_preceding(fallback_reference.declarative_position())?",
        "objects.into_identifier_call_plan(",
        "let fallback_callee = self.with_identifier_call_fallback(&name, fallback_reference);",
        "self.lower_call_args_expanding_spread(source_args)",
        "ExprIr::CallIndirect {",
        "this_arg: None",
        "Some(plan.call(args, fallback))",
    ] {
        assert!(lower.contains(marker), "missing lowering marker: {marker}");
    }
    assert_before(
        lower,
        "locate_identifier_reference",
        "into_identifier_call_plan",
    );
    assert_before(
        lower,
        "into_identifier_call_plan",
        "with_identifier_call_fallback",
    );
    assert_before(
        lower,
        "with_identifier_call_fallback",
        "lower_call_args_expanding_spread",
    );
    assert_before(
        lower,
        "lower_call_args_expanding_spread",
        "plan.call(args, fallback)",
    );

    let fallback = bounded(
        CALL_SOURCE,
        "    fn with_identifier_call_fallback(",
        "    fn widen_with_identifier_call_global_fallback(",
    );
    assert!(fallback.contains("let dynamic = unknown_runtime_value_info();"));
    assert!(fallback.contains("ExprIr::GlobalIdentifierRead"));
    assert!(fallback.contains("self.widen_binding_for_possible_replacement(name);"));
    assert!(!fallback.contains("ExprIr::CallNamed"));
    assert!(!fallback.contains("ExprIr::FunctionValue"));

    let widen = bounded(
        CALL_SOURCE,
        "    fn widen_with_identifier_call_global_fallback(",
        "\n    }\n}",
    );
    for marker in [
        "binding.kind = dynamic.kind;",
        "binding.possible_kinds = dynamic.possible_kinds;",
        "widen_for_possible_replacement();",
        "property.value_info.widen_for_possible_replacement();",
        "property.proven_present = false;",
    ] {
        assert!(
            widen.contains(marker),
            "missing fallback widening: {marker}"
        );
    }
}

#[test]
fn aot_preserves_explicit_this_and_evaluates_arguments_after_the_callee() {
    let dispatch = bounded(
        EXPRESSIONS_SOURCE,
        "            ExprIr::CallIndirect {",
        "            ExprIr::Construct {",
    );
    assert!(dispatch.contains("this_arg.as_deref()"));
    assert!(dispatch.contains("self.emit_indirect_call("));

    let indirect = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_indirect_call(",
        "    pub(crate) fn emit_tail_indirect_call(",
    );
    for marker in [
        "self.compile_expr_to_locals(callee, callee_payload_local, callee_tag_local, function)?;",
        "if let Some(this_arg) = this_arg",
        "self.compile_expr_to_locals(this_arg, this_payload_local, this_tag_local, function)?;",
        "let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;",
        "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
    ] {
        assert!(
            indirect.contains(marker),
            "missing AOT call marker: {marker}"
        );
    }
    let generic_evaluation = bounded(
        indirect,
        "        let callee_payload_local = self.reserve_temp_local();\n        let callee_tag_local = self.reserve_temp_local();\n        let default_this_payload_local = self.reserve_temp_local();",
        "        if let Some(StaticRegExpCompilation::InvalidSyntax",
    );
    assert_before(
        generic_evaluation,
        "compile_expr_to_locals(callee",
        "if let Some(this_arg)",
    );
    assert_before(
        generic_evaluation,
        "compile_expr_to_locals(this_arg",
        "emit_call_args_vector(args",
    );
    assert_before(
        indirect,
        "emit_call_args_vector(args",
        "emit_function_or_proxy_call_with_argv_leave_throw_completion",
    );
}

#[test]
fn exact_witness_and_fixture_bound_the_claim() {
    assert_eq!(WITNESS_PATH, "language/expressions/call/with-base-obj.js");
    assert!(WITNESS.contains("flags: [noStrict]"));
    assert!(WITNESS.contains("viaCall = this;"));
    assert!(WITNESS.contains("method();"));
    assert!(WITNESS.contains("assert.sameValue(viaCall, obj, 'via CallExpression');"));
    assert!(CONTRACT.contains("reports `0/1` under Wasm AOT as `Bug/Runtime`"));
    assert!(CONTRACT.contains("This is focused current evidence"));

    for marker in [
        "trace === \"huhrgac\"",
        "selectedThis === selectedProxy",
        "selectedGetterThis === selectedProxy",
        "strictFallbackThis === undefined",
        "sloppyFallbackThis === globalObject",
        "innerBinding[Symbol.unscopables] = { nestedMethod: true }",
        "outerThis === outerBinding",
        "builtinShadowResult === \"shadow:1\"",
        "builtinFallbackResult === false",
        "fallbackMutationHasCalls === 1",
        "mutatedFallbackResult === \"new\"",
        "mutatedFallbackThis === undefined",
    ] {
        assert!(
            FIXTURE.contains(marker),
            "missing fixture contract: {marker}"
        );
    }
}
