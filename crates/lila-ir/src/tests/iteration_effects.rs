fn retained_splice_arguments<'a>(expression: &'a TypedExpr, source_key: &str) -> &'a [TypedExpr] {
    let ExprIr::MaterializeBinding { name, body, .. } = &expression.expr else {
        panic!("splice must retain one materialized receiver: {expression:?}");
    };
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(receiver),
        args,
        ..
    } = &body.expr
    else {
        panic!("splice must call its acquired property value: {body:?}");
    };
    assert!(matches!(&receiver.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(
        match &callee.expr {
            ExprIr::PropertyRead {
                target,
                key: PropertyKeyIr::StaticString(key),
            } =>
                matches!(&target.expr, ExprIr::Identifier(storage) if storage == name)
                    && key == source_key,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } =>
                operands.len() == 2
                    && matches!(&operands[0].expr, ExprIr::Identifier(storage) if storage == name)
                    && matches!(&operands[1].expr, ExprIr::String(key) if key == source_key),
            _ => false,
        },
        "the original source property and receiver must survive: {callee:?}"
    );
    assert_eq!(expression.kind, ValueKind::Dynamic);
    assert!(expression.heap_shape.is_none());
    for kind in [
        ValueKind::Array,
        ValueKind::Object,
        ValueKind::Function,
        ValueKind::Arguments,
    ] {
        assert!(expression.possible_kinds.contains(kind), "{expression:?}");
    }
    args
}

#[test]
fn splice_object_keys_operand_keeps_real_argument_spread_and_original_callee() {
    let program = lower_script(
        "function run() { return [1, 2].splice(0, 0, ...Object.keys({ x: 1 })); } run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script IR")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");
    let result = function_return(run).expect("splice result");
    let args = retained_splice_arguments(result, "splice");
    let [start, delete_count, insertion] = args else {
        panic!("the original three source arguments must survive: {args:?}");
    };
    assert!(matches!(start.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 0.0));
    assert!(matches!(delete_count.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 0.0));
    let ExprIr::SpreadArgument(spread) = &insertion.expr else {
        panic!("Object.keys is an iterated spread operand: {insertion:?}");
    };
    assert_eq!(spread.protocol, SpreadArgumentProtocol::ARGUMENT_LIST);
    let call = indirect_call_body(&spread.value).expect("Object.keys call must remain observable");
    assert!(matches!(&call.expr, ExprIr::CallIndirect { args, .. } if args.len() == 1));
}

#[test]
fn borrowed_splice_retains_source_key_receiver_and_general_argument_order() {
    let program = lower_script(
            "function run(items, more) { \
                 const target = { length: 2, 0: 'a', 1: 'b', transferred: Array.prototype.splice }; \
                 return (0, target).transferred({ valueOf() { return -1; } }, 3, ...items, 'tail', ...more); \
             } run(['x'], ['y']);",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script IR")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");
    let result = function_return(run).expect("borrowed splice result");
    let ExprIr::MaterializeBinding { value, .. } = &result.expr else {
        unreachable!()
    };
    assert!(matches!(&value.expr, ExprIr::Comma { rhs, .. }
            if matches!(&rhs.expr, ExprIr::Identifier(name) if name == "target")));
    let args = retained_splice_arguments(result, "transferred");
    let [start, delete_count, items, tail, more] = args else {
        panic!("each spread and trailing argument keeps its source position: {args:?}");
    };
    assert_eq!(start.kind, ValueKind::Object);
    assert!(matches!(delete_count.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 3.0));
    assert!(matches!(&tail.expr, ExprIr::String(value) if value == "tail"));
    for (argument, source_name) in [(items, "items"), (more, "more")] {
        let ExprIr::SpreadArgument(spread) = &argument.expr else {
            panic!("expected the original argument spread: {argument:?}");
        };
        assert_eq!(spread.protocol, SpreadArgumentProtocol::ARGUMENT_LIST);
        assert!(matches!(&spread.value.expr, ExprIr::Identifier(name) if name == source_name));
    }
}

#[test]
fn splice_custom_species_result_does_not_claim_an_array_or_element_shape() {
    let program = lower_script(
            "function run() { \
                 const values = [1, 2]; \
                 values.constructor = { [Symbol.species]: function Custom() { return { marker: true }; } }; \
                 return values.splice(0, 1, ...Object.keys({ inserted: 1 })); \
             } run();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script IR")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function");
    let result = function_return(run).expect("custom species splice result");
    let args = retained_splice_arguments(result, "splice");
    assert_eq!(args.len(), 3);
    assert!(matches!(args[1].expr, ExprIr::Number(bits) if f64::from_bits(bits) == 1.0));
    assert!(matches!(&args[2].expr, ExprIr::SpreadArgument(spread)
            if spread.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
}

#[test]
fn captured_array_splice_fast_path_invalidates_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
        "let values = [1]; function mutate() { values.splice(0, 1); } mutate(); values[0] + 1;",
    );
}

#[test]
fn forwarded_array_push_with_call_invalidates_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "let values = [1]; function mutate() { Array.prototype.push.call(values, 2); } mutate(); values[0] + 1;",
        );
}

#[test]
fn forwarded_array_pop_with_apply_invalidates_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "let values = [1]; function mutate() { Array.prototype.pop.apply(values, []); } mutate(); values[0] + 1;",
        );
}

#[test]
fn forwarded_fill_with_call_invalidates_current_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
        "const values = [1]; Array.prototype.fill.call(values, 'x'); values[0] + 1;",
    );
}

#[test]
fn forwarded_fill_with_apply_invalidates_current_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
        "const values = [1]; Array.prototype.fill.apply(values, ['x']); values[0] + 1;",
    );
}

#[test]
fn forwarded_nonmutating_callback_builtin_invalidates_later_return_shape() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "let target = { value: 1 }; function readTarget() { return target; } function callback() { delete target.value; } Array.prototype.forEach.call([0], callback); readTarget().value + 1;",
        );
}

#[test]
fn array_spread_iterator_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterable = { [Symbol.iterator]() { values = {}; return [][Symbol.iterator](); } }; let values = [1]; [...iterable]; values[0] + 1;",
        );
}

#[test]
fn for_of_iterator_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterable = { [Symbol.iterator]() { values = {}; return [][Symbol.iterator](); } }; let values = [1]; for (const value of iterable) {} values[0] + 1;",
        );
}

#[test]
fn object_spread_getter_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const source = { get value() { values = {}; return 0; } }; let values = [1]; ({ ...source }); values[0] + 1;",
        );
}

#[test]
fn array_parameter_iterator_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterable = { [Symbol.iterator]() { values = {}; return [][Symbol.iterator](); } }; function consume([value]) {} let values = [1]; consume(iterable); values[0] + 1;",
        );
}

#[test]
fn object_parameter_getter_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const source = { get value() { values = {}; return 0; } }; function consume({ value }) {} let values = [1]; consume(source); values[0] + 1;",
        );
}

#[test]
fn number_conversion_hooks_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const coercible = { valueOf() { values = {}; return 1; } }; let values = [1]; Number(coercible); values[0] + 1;",
        );
}

#[test]
fn string_conversion_hooks_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const coercible = { toString() { values = {}; return 'x'; } }; let values = [1]; String(coercible); values[0] + 1;",
        );
}

#[test]
fn symbol_description_conversion_hooks_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const coercible = { toString() { values = {}; return 'x'; } }; let values = [1]; Symbol(coercible); values[0] + 1;",
        );
}

#[test]
fn for_in_proxy_hooks_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const proxy = new Proxy({}, { ownKeys() { values = {}; return []; } }); let values = [1]; for (const key in proxy) {} values[0] + 1;",
        );
}

#[test]
fn in_operator_proxy_hooks_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const proxy = new Proxy({}, { has() { values = {}; return false; } }); let values = [1]; 'key' in proxy; values[0] + 1;",
        );
}

#[test]
fn instanceof_hooks_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const candidate = { [Symbol.hasInstance]() { values = {}; return false; } }; let values = [1]; 0 instanceof candidate; values[0] + 1;",
        );
}

#[test]
fn await_then_getter_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const thenable = { get then() { values = {}; return undefined; } }; async function observe() { await thenable; } let values = [1]; observe(); values[0] + 1;",
        );
}

#[test]
fn using_disposal_effects_invalidate_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const resource = { [Symbol.dispose]() { values = {}; } }; function release() { { using acquired = resource; } } let values = [1]; release(); values[0] + 1;",
        );
}

#[test]
fn object_from_entries_iteration_invalidates_later_element_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterable = { [Symbol.iterator]() { values = {}; return [][Symbol.iterator](); } }; let values = [1]; Object.fromEntries(iterable); values[0] + 1;",
        );
}

#[test]
fn collection_construction_iteration_invalidates_later_property_analysis() {
    for constructor in ["Map", "Set", "WeakMap", "WeakSet"] {
        let source = format!(
                "const iterable = {{ [Symbol.iterator]() {{ holder = {{}}; return [][Symbol.iterator](); }} }}; let holder = {{ value: 1 }}; new {constructor}(iterable); holder.value + 1;"
            );
        assert_caller_flow_invalidation_reaches_final_addition(&source);
    }
}

#[test]
fn empty_and_nullish_collection_construction_preserves_caller_flow_facts() {
    for construction in [
        "new Map()",
        "new Map(null)",
        "new Set()",
        "new Set(undefined)",
        "new WeakMap()",
        "new WeakMap(null)",
        "new WeakSet()",
        "new WeakSet(undefined)",
    ] {
        let source = format!("const holder = {{ value: 1 }}; {construction}; holder.value + 1;");
        assert_caller_flow_preservation_reaches_final_addition(&source);
    }
}

#[test]
fn ordinary_collection_constructor_calls_do_not_inspect_the_iterable() {
    for constructor in ["Map", "Set", "WeakMap", "WeakSet"] {
        let source = format!(
                "let holder = {{ value: 1 }}; {constructor}({{ [Symbol.iterator]() {{ holder = {{}}; return [][Symbol.iterator](); }} }}); holder.value + 1;"
            );
        assert_caller_flow_preservation_reaches_final_addition(&source);
    }
}

#[test]
fn iterator_from_next_getter_invalidates_later_property_analysis() {
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterator = { [Symbol.iterator]: null, get next() { holder = {}; return function () { return { done: true }; }; } }; let holder = { value: 1 }; Iterator.from(iterator); holder.value + 1;",
        );
}

#[test]
fn iterator_from_data_next_property_does_not_prove_the_prototype_walk_pure() {
    // An own next method says nothing about the later OrdinaryHasInstance
    // prototype walk. Native controls cover its actual traps and ordering.
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterator = { [Symbol.iterator]: null, next() { return { done: true }; } }; const holder = { value: 1 }; Iterator.from(iterator); holder.value + 1;",
        );
}

#[test]
fn iterator_from_noncallable_method_keeps_conservative_call_effects() {
    // The runtime must throw before reading next. This analysis does not
    // model that abrupt cutoff, so it must not publish an effect-free proof.
    assert_caller_flow_invalidation_reaches_final_addition(
            "const iterator = { [Symbol.iterator]: 0, get next() { holder = {}; return function () { return { done: true }; }; } }; let holder = { value: 1 }; Iterator.from(iterator); holder.value + 1;",
        );
}

#[test]
fn iterator_helper_creation_observes_the_iterator_next_property() {
    for method in ["map", "filter", "flatMap", "take", "drop"] {
        let argument = if matches!(method, "take" | "drop") {
            "1"
        } else {
            "function (value) { return value; }"
        };
        let source = format!(
                "let values = [1]; Iterator.prototype.{method}.call({{ get next() {{ values = {{}}; }} }}, {argument}); values[0] + 1;"
            );
        assert_caller_flow_invalidation_reaches_final_addition(&source);
    }
}

#[test]
fn set_algebra_observes_the_set_like_argument() {
    for method in [
        "difference",
        "intersection",
        "isDisjointFrom",
        "isSubsetOf",
        "isSupersetOf",
        "symmetricDifference",
        "union",
    ] {
        let source = format!(
                "let values = [1]; new Set().{method}({{ get size() {{ values = {{}}; return 0; }}, has() {{ return false; }}, keys() {{ return [][Symbol.iterator](); }} }}); values[0] + 1;"
            );
        assert_caller_flow_invalidation_reaches_final_addition(&source);
    }
}

#[test]
fn a_shadowed_number_callee_is_not_lowered_as_the_intrinsic() {
    let program = lower_script("function run(Number) { return Number(1); }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let run = script
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function should be lowered");
    let returned = function_return(run).expect("run should return its call");

    assert!(indirect_call_body(returned).is_some(), "{returned:?}");
}

#[test]
fn a_shadowed_parse_float_callee_is_not_lowered_as_the_intrinsic() {
    let program = lower_script("function run(parseFloat) { return parseFloat('1'); }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let run = script
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run function should be lowered");
    let returned = function_return(run).expect("run should return its call");

    assert!(indirect_call_body(returned).is_some(), "{returned:?}");
}

#[test]
fn a_symbol_factory_retains_the_actual_call_without_an_effect_free_proof() {
    let program = lower_script(
        "Symbol.prototype.q = function q() {}; function makeSymbol() { return Symbol('marker'); }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let factory = script
        .functions
        .iter()
        .find(|function| function.name == "makeSymbol")
        .expect("makeSymbol should be lowered");

    assert!(
        crate::source_call_flow_proof::prove_no_caller_flow_invalidation(
            &factory.params,
            &factory.body,
        )
        .is_none(),
        "{:?}",
        factory.body
    );
    let returned = function_return(factory).expect("factory result");
    let ExprIr::CallIndirect { callee, args, .. } = &returned.expr else {
        panic!("Symbol must retain its actual callee and arguments: {returned:?}");
    };
    assert!(matches!(&callee.expr, ExprIr::GlobalPropertyRead { name }
        | ExprIr::GlobalIdentifierRead { name } if name == "Symbol"));
    assert!(
        matches!(args.as_slice(), [TypedExpr { expr: ExprIr::String(value), .. }]
        if value == "marker")
    );
}

#[test]
fn exact_mixed_call_result_excludes_the_throwing_class_constructor_branch() {
    let source = "function returnsNumber() { return 1; } class Constructor {} let candidate = unknown ? returnsNumber : Constructor; candidate();";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let call = script
        .body
        .statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => indirect_call_body(expression),
            _ => None,
        })
        .expect("mixed target call should remain in IR");
    assert_eq!(call.possible_kinds, KindSet::from_kind(ValueKind::Number));
}

#[test]
fn exact_mixed_call_result_excludes_the_always_throwing_typed_array_intrinsic() {
    let source = "function returnsNumber() { return 1; } let TypedArray = Object.getPrototypeOf(Int8Array); let candidate = unknown ? returnsNumber : TypedArray; candidate();";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let call = script
        .body
        .statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => indirect_call_body(expression),
            _ => None,
        })
        .expect("mixed target call should remain in IR");
    assert_eq!(call.possible_kinds, KindSet::from_kind(ValueKind::Number));
}

#[test]
fn exact_mixed_construct_result_excludes_the_always_throwing_typed_array_intrinsic() {
    let source = "function Constructor() { this.value = 1; } let TypedArray = Object.getPrototypeOf(Int8Array); let candidate = unknown ? Constructor : TypedArray; new candidate();";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(construct) = script.body.statements.last().unwrap() else {
        panic!("construct should be an expression statement");
    };
    assert!(matches!(construct.expr, ExprIr::Construct { .. }));
    assert_eq!(
        construct.possible_kinds,
        KindSet::from_kind(ValueKind::Object)
    );
}

#[test]
fn spread_call_candidates_do_not_reuse_narrow_parameter_returns() {
    let source = r#"
function A(value) { return value; }
function B(value) { return value; }
A(1);
B(1);
let candidate = unknown ? A : B;
candidate(...[{ marker: 1 }]);
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let call = script
        .body
        .statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => indirect_call_body(expression),
            _ => None,
        })
        .expect("spread call should remain in IR");
    assert!(call.possible_kinds.contains(ValueKind::Object));
    for function_name in ["A", "B"] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("{function_name} should be lowered"));
        assert_eq!(function.params[0].kind, ValueKind::Dynamic);
    }
}

#[test]
fn spread_direct_call_does_not_reuse_a_narrow_parameter_return() {
    let source = r#"
function identity(value) { return value; }
identity(1);
identity(...[{ marker: 1 }]);
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let call = script
        .body
        .statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => indirect_call_body(expression),
            _ => None,
        })
        .expect("spread call should remain in IR");
    assert!(call.possible_kinds.contains(ValueKind::Object));
    let identity = script
        .functions
        .iter()
        .find(|function| function.name == "identity")
        .expect("identity should be lowered");
    assert_eq!(identity.params[0].kind, ValueKind::Dynamic);
}

#[test]
fn spread_construct_candidates_include_unknown_explicit_object_returns() {
    let source = r#"
function A(value) { return value; }
function B(value) { return value; }
A(1);
B(1);
function ReturnedFunction() {}
let Constructor = unknown ? A : B;
new Constructor(...[ReturnedFunction]);
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(construct) = script.body.statements.last().unwrap() else {
        panic!("construct should be an expression statement");
    };
    assert!(matches!(construct.expr, ExprIr::Construct { .. }));
    assert!(construct.possible_kinds.contains(ValueKind::Function));
}

#[test]
fn spread_direct_construct_includes_unknown_explicit_object_returns() {
    let source = r#"
function Constructor(value) { return value; }
Constructor(1);
function ReturnedFunction() {}
new Constructor(...[ReturnedFunction]);
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(construct) = script.body.statements.last().unwrap() else {
        panic!("construct should be an expression statement");
    };
    assert!(matches!(construct.expr, ExprIr::Construct { .. }));
    assert!(construct.possible_kinds.contains(ValueKind::Function));
}

#[test]
fn multi_target_constructor_does_not_reuse_replaced_prototype_shapes() {
    let source = r#"
function A() {}
function B() {}
A.prototype = { a: 1 };
B.prototype = { b: 2 };
let Constructor = unknown ? A : B;
new Constructor().a;
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(property) = script.body.statements.last().unwrap() else {
        panic!("property read should be an expression statement");
    };
    assert!(property.possible_kinds.contains(ValueKind::Number));
}

#[test]
fn multi_target_builtin_without_specialized_analysis_keeps_its_signature_result() {
    let source = r#"
let locale = new Intl.Locale("en");
let scriptGetter = Object.getOwnPropertyDescriptor(
    Intl.Locale.prototype,
    "script",
).get;
function returnsNumber() { return 1; }
locale.candidate = unknown ? scriptGetter : returnsNumber;
locale.candidate();
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let call = script
        .body
        .statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => indirect_call_body(expression),
            _ => None,
        })
        .expect("mixed target call should remain in IR");
    assert!(call.possible_kinds.contains(ValueKind::String));
    assert!(call.possible_kinds.contains(ValueKind::Undefined));
    assert!(call.possible_kinds.contains(ValueKind::Number));
}

#[test]
fn optional_exact_mixed_call_merges_only_the_short_circuit_branch() {
    let source = "function returnsNumber() { return 1; } let candidate = unknown ? returnsNumber : undefined; candidate?.();";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(call) = script.body.statements.last().unwrap() else {
        panic!("optional call should be an expression statement");
    };
    assert_eq!(
        call.possible_kinds,
        KindSet::from_kind(ValueKind::Number).union(KindSet::from_kind(ValueKind::Undefined))
    );
}

#[test]
fn mixed_and_open_function_constructor_candidates_keep_dynamic_source_authority() {
    for source in [
        "let Constructor = unknown ? Function : undefined; new Constructor('return 1');",
        "var Constructor = Function; globalThis.unknownHook(); new Constructor('return 1');",
    ] {
        let program = lower_script(source);
        assert!(
            program.diagnostics.is_empty(),
            "{source}: {:?}",
            program.diagnostics
        );
        assert_eq!(program.script.unwrap().prepared_dynamic_functions.len(), 1);
    }
}

#[test]
fn multi_target_constructor_retains_non_string_arguments_through_unknown_setters() {
    let source = "function Constructor(value) { this.value = value; } let Target = unknown ? Constructor : Array; new Target(7);";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.name == "Constructor")
        .expect("Constructor function should exist");
    assert_eq!(constructor.params[0].name, "value");
    let Some(StatementIr::Expression(TypedExpr {
        expr: ExprIr::Construct { callee, args, .. },
        ..
    })) = script.body.statements.last()
    else {
        panic!("multi-target construct must retain its evaluated operands");
    };
    assert!(
        matches!(args.as_slice(), [TypedExpr { expr: ExprIr::Number(bits), .. }]
        if *bits == 7.0_f64.to_bits())
    );
    assert!(callee
        .function_targets
        .known_targets()
        .contains(&constructor.id));
    assert!(callee
        .function_targets
        .known_targets()
        .contains(&StandardBuiltinId::ArrayConstructor.function_id()));
    let assignment = constructor
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::OrdinaryPropertyAssignment(assignment),
                ..
            }) => Some(assignment),
            _ => None,
        })
        .expect("constructor keeps its value write");
    assert!(matches!(assignment.base_and_receiver().expr, ExprIr::This));
    assert_eq!(
        assignment.referenced_name(),
        &PropertyKeyIr::StaticString("value".into())
    );
    assert!(matches!(&assignment.rhs().expr, ExprIr::Identifier(name) if name == "value"));
}

#[test]
fn multi_target_constructor_uses_the_reusable_exact_context_return_kind() {
    let source = r#"
const ArrayConstructor = Array;
function ReturnedFunction() {}
function C(value) { return value; }
C(ReturnedFunction);
let Target = unknown ? C : ArrayConstructor;
new Target([]);
"#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(construct) = script.body.statements.last().unwrap() else {
        panic!("construct should be an expression statement");
    };
    assert!(matches!(construct.expr, ExprIr::Construct { .. }));
    assert!(construct.possible_kinds.contains(ValueKind::Object));
    assert!(construct.possible_kinds.contains(ValueKind::Array));
    assert!(!construct.possible_kinds.contains(ValueKind::Function));
}

#[test]
fn missing_class_candidate_signatures_invalidate_static_initializer_facts() {
    for source in [
            "const holder = { f: eval }; class C { static m() {} static x = ((unknown ? C.m : Math.abs)(), holder.f); }",
            "const holder = { f: eval }; class C { static x = (new (unknown ? C : Array)(), holder.f); }",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script IR should exist");
            let initializer = script
                .functions
                .iter()
                .find(|function| function.name == "C.field.x")
                .expect("static field initializer should be lowered");
            assert!(
                initializer.return_targets.exact_targets().is_none(),
                "{source}"
            );
            assert!(
                initializer.return_targets.known_targets().is_empty(),
                "{source}"
            );
        }
}

#[test]
fn construct_results_exclude_primitive_return_branches() {
    for source in [
            "function A() { return unknown ? {} : 1; } new A();",
            "function A() { return unknown ? {} : 1; } function B() { return []; } let C = unknown ? A : B; new C();",
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script IR should exist");
            let StatementIr::Expression(construct) = script.body.statements.last().unwrap() else {
                panic!("construct should be an expression statement");
            };
            assert!(matches!(construct.expr, ExprIr::Construct { .. }));
            let object_like_kinds = KindSet::from_kind(ValueKind::Object)
                .union(KindSet::from_kind(ValueKind::Array))
                .union(KindSet::from_kind(ValueKind::Function))
                .union(KindSet::from_kind(ValueKind::Arguments));
            assert_ne!(construct.possible_kinds, KindSet::EMPTY, "{source}");
            assert!(
                construct.possible_kinds.is_subset_of(object_like_kinds),
                "{source}"
            );
            assert!(
                !construct.possible_kinds.contains(ValueKind::Number),
                "{source}"
            );
        }
}

#[test]
fn possible_proxy_constructor_candidates_seed_literal_trap_parameters() {
    let source = "function target() {} function Other() {} let C = unknown ? Proxy : Other; new C(target, { apply(target, thisArg, args) { return args.length; } }, ...[]);";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let apply = script
        .functions
        .iter()
        .find(|function| function.name == "apply")
        .expect("literal apply trap should be lowered");
    assert_eq!(apply.params[2].kind, ValueKind::Array);
}
