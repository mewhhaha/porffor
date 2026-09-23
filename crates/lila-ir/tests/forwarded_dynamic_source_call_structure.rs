const CANDIDATE_SOURCE: &str = include_str!("../src/lowering/call_candidate_analysis.rs");
const CALL_SOURCE: &str = include_str!("../src/lowering/call_expression.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const INTRINSIC_METHOD_SOURCE: &str = include_str!("../src/lowering/intrinsic_method.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/dynamic-source-capability.md");
const TASK: &str = include_str!("../../../tasks/13-dynamic-source-evaluation.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn dynamic_source_preflight_has_one_closed_admission_token() {
    let declaration = normalized(bounded(
        CANDIDATE_SOURCE,
        "#[must_use = \"dynamic-source candidate preflight must admit analysis or reject emission\"]",
        "impl<'a> ScriptLowerer<'a> {",
    ));

    assert!(declaration.contains("enumDynamicSourceCallAdmission{"));
    assert!(declaration.contains("Admitted(AdmittedDynamicSourceCall)"));
    assert!(declaration.contains("Rejected"));
    assert!(declaration.contains("structAdmittedDynamicSourceCall{"));
    assert!(declaration.contains("function_ids:Vec<FunctionId>"));
    assert!(declaration.contains("pass_through_results:BTreeMap<FunctionId,ValueInfo>"));
    for escape_hatch in [
        "Clone",
        "Copy",
        "Default",
        "pubfunction_ids",
        "pubpass_through_results",
    ] {
        assert!(!declaration.contains(escape_hatch), "{escape_hatch}");
    }
}

#[test]
fn ordinary_and_forwarded_calls_share_the_same_candidate_preflight() {
    let preflight = normalized(bounded(
        CANDIDATE_SOURCE,
        "    fn preflight_dynamic_source_call_candidates(",
        "    pub(super) fn preflight_function_prototype_call_dynamic_source(",
    ));
    assert!(preflight.contains("callee.function_targets.known_targets()"));
    assert_eq!(
        preflight
            .matches("self.resolve_dynamic_source_call(function_id,source.arguments(),arguments)")
            .count(),
        1,
    );
    assert!(!preflight.contains("DirectEval"));
    assert!(preflight.contains("self.record_unsupported_dynamic_source(unsupported)"));
    assert!(preflight.contains("DynamicSourceCallAdmission::Rejected"));
    assert!(preflight.contains(
        "DynamicSourceCallAdmission::Admitted(AdmittedDynamicSourceCall{function_ids,pass_through_results,})"
    ));

    let ordinary = normalized(bounded(
        CANDIDATE_SOURCE,
        "    pub(super) fn analyze_known_call_candidates(",
        "    pub(super) fn analyze_known_construct_candidates(",
    ));
    assert_eq!(
        ordinary
            .matches("preflight_dynamic_source_call_candidates")
            .count(),
        1
    );
    assert!(ordinary.contains("DynamicSourceCallAdmission::Admitted("));
    assert!(ordinary.contains("DynamicSourceCallAdmission::Rejected=>{"));
    assert!(!ordinary.contains("_=>"));
    assert!(ordinary.contains("callee.function_targets.exact_targets().is_none()"));
}

#[test]
fn forwarding_shifts_this_arg_and_cannot_manufacture_direct_eval_authority() {
    let forwarding = normalized(bounded(
        CANDIDATE_SOURCE,
        "    pub(super) fn preflight_function_prototype_call_dynamic_source(",
        "    pub(super) fn consume_forwarded_dynamic_source_admission(",
    ));
    assert!(forwarding.contains("debug_assert!(!Self::call_args_have_spread(arguments))"));
    assert!(forwarding.contains("matches!(argument,Expression::Spread(_))"));
    assert!(forwarding.contains("arguments.get(1..).unwrap_or_default()"));
    assert!(forwarding.contains(
        "CallCandidateSource::IndirectSyntax(source_arguments.get(1..).unwrap_or_default())"
    ));
    assert_eq!(
        forwarding
            .matches("preflight_dynamic_source_call_candidates")
            .count(),
        1
    );
    for deferred_route in [
        "FunctionPrototypeApply",
        "ReflectApply",
        "ReflectConstruct",
        "BoundFunction",
        "Proxy",
    ] {
        assert!(!forwarding.contains(deferred_route), "{deferred_route}");
    }
}

#[test]
fn forwarding_requires_current_intrinsic_call_property_authority() {
    // `Function.prototype.call` is one row of the shared intrinsic-method
    // catalogue, and only the live-prototype proof turns a row into a callee.
    let catalogue = normalized(bounded(
        INTRINSIC_METHOD_SOURCE,
        "    pub(super) fn catalogued_method(self, name: &str) -> Option<StandardBuiltinId> {",
        "            _ => return None,",
    ));
    assert!(catalogue.contains("(Self::Function,\"call\")=>B::FunctionPrototypeCall"));
    assert!(INTRINSIC_METHOD_SOURCE
        .contains("Self::Function => StandardBuiltinId::FunctionConstructor,"));

    let live_prototype = normalized(bounded(
        INTRINSIC_METHOD_SOURCE,
        "    pub(super) fn live_intrinsic_prototype(",
        "    fn intrinsic_prototype_still_holds(",
    ));
    assert!(live_prototype.contains(".get(constructor.global_name()?)"));
    assert!(live_prototype.contains(".filter(|property|property.proven_present)?"));
    assert!(live_prototype.contains("!=Some(&constructor.function_id())"));
    assert!(live_prototype.contains("constructor_shape.properties.get(\"prototype\")?"));
    assert!(live_prototype.contains("ObjectShapeProperty::Accessor{..}=>None"));

    let authority = normalized(bounded(
        INTRINSIC_METHOD_SOURCE,
        "    fn intrinsic_prototype_still_holds(",
        "\nfn own_shape_property<",
    ));
    assert!(authority.contains("StandardBuiltinId::FunctionPrototype.function_id()"));
    assert!(authority.contains("own_shape_property(live_shape,name)"));
    assert!(authority
        .contains("method.function_targets.exact_single_target()==Some(&builtin.function_id())"));
    assert!(authority.contains("Some(ObjectShapeProperty::Accessor{..})=>false"));
    // Absence is intrinsic only where the fresh-realm shape omits the name.
    assert!(authority.contains("own_shape_property(shape,name).is_none()"));
    // The proof token has exactly one constructor: the lookup above.
    assert_eq!(
        INTRINSIC_METHOD_SOURCE
            .matches("IntrinsicMethod { builtin }")
            .count(),
        1
    );

    // Property lowering claims `call` only for a receiver whose own properties
    // are known, and call lowering has no name-based route of its own left.
    let resolution = normalized(bounded(
        LOWERING_SOURCE,
        "            let target_is_function_prototype =",
        "            if name == \"of\" && self.is_builtin_reference_expr(&target, ARRAY_NAME) {",
    ));
    assert!(resolution.contains("self.intrinsic_method(IntrinsicPrototype::Function,name)"));
    assert!(resolution.contains(".filter(|_|target.heap_shape.is_some())"));
    assert!(resolution.contains("lookup.unclaimed()"));
    assert_eq!(
        LOWERING_SOURCE
            .matches("self.intrinsic_method(IntrinsicPrototype::Function, name)")
            .count(),
        1
    );
    assert!(!CALL_SOURCE.contains("if receiver.possible_kinds.contains(ValueKind::Function) =>"));
    assert!(!CALL_SOURCE.contains("\"call\""));
}

#[test]
fn rejected_forwarding_returns_before_target_observation_or_emission() {
    let call = normalized(bounded(
        CALL_SOURCE,
        "                        let forwarded_dynamic_source_result =",
        "                    if string_from_code_point_apply_call",
    ));
    assert!(call.contains("Some(StandardBuiltinId::FunctionPrototypeCall)"));
    assert!(call.contains("&&!Self::call_args_have_spread(&args"));
    assert!(call.contains("DynamicSourceCallAdmission::Admitted(admission)=>"));
    assert!(call.contains("DynamicSourceCallAdmission::Rejected=>{returnTypedExpr::undefined();}"));
    let admission_match_end = call
        .find("}else{None};")
        .expect("forwarding admission result");
    assert!(!call[..admission_match_end].contains("_=>"));

    let rejection = call
        .find("DynamicSourceCallAdmission::Rejected")
        .expect("closed rejection branch");
    let this_observation = call
        .find("self.merge_function_this_info")
        .expect("forwarded this observation");
    let flow_observation = call
        .find("self.consume_forwarded_call_flow_effects")
        .expect("forwarded caller-flow observation");
    assert!(rejection < this_observation);
    assert!(rejection < flow_observation);
    assert!(call.contains("&&forwarded_dynamic_source_result.is_none()"));

    let full_call = normalized(CALL_SOURCE);
    let source_capture = full_call
        .find("letsource_arguments=args;")
        .expect("source argument capture");
    let argument_lowering = full_call[source_capture..]
        .find("self.lower_call_args(")
        .expect("argument lowering after source capture")
        + source_capture;
    let preflight = full_call[argument_lowering..]
        .find("self.preflight_function_prototype_call_dynamic_source(")
        .expect("forwarded preflight after argument lowering")
        + argument_lowering;
    let emission = full_call[preflight..]
        .find("self.lower_indirect_method_call(")
        .expect("method-call emission after preflight")
        + preflight;
    assert!(source_capture < argument_lowering);
    assert!(argument_lowering < preflight);
    assert!(preflight < emission);
    // Mirrors the raw-line budget in scripts/check-module-boundaries.sh.
    assert!(CALL_SOURCE.lines().count() <= 3_112);
}

#[test]
fn contract_and_t13_keep_the_forwarding_slice_and_remaining_debt_explicit() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "spread-free intrinsic `Function.prototype.call` forwarding",
        "closed, must-use `DynamicSourceCallAdmission`",
        "`Function.prototype.call` acquisition remains proven intrinsic",
        "Finite candidate discovery preserves runtime callable identity and exact source equality",
        "it does not imply unrestricted forwarding support",
    ] {
        assert!(contract_words.contains(marker), "contract: {marker}");
        assert!(task_words.contains(marker), "T13: {marker}");
    }
}
