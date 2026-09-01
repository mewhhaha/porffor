use super::*;
use lila_front::{parse, ParseOptions};

fn lower(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).expect("script should parse");
    crate::lower(&source)
}

fn lower_test262(source: &str) -> ProgramIr {
    let source = parse(source, ParseOptions::script()).expect("script should parse");
    crate::lower_with_host_surface_policy(&source, HostSurfacePolicy::Test262)
}

fn assert_realm_eval_is_not_typed(program: &ProgramIr) {
    assert!(!program.diagnostics.iter().any(|diagnostic| {
        diagnostic.unsupported_feature()
            == Some(UnsupportedFeature::DynamicSource(
                DynamicSourceGap::aot_known_source(DynamicSourceKind::RealmEvalScript),
            ))
    }));
    assert!(!program
        .script
        .as_ref()
        .expect("script IR should exist")
        .host_builtins
        .contains(&HostBuiltinId::RealmEvalScript));
}

#[test]
fn later_descriptor_evaluation_invalidates_the_captured_target_shape() {
    let program = lower(
        "const target = { x: 1 }; Object.defineProperty(target, 'x', { value: (target.y = 0, 's') }); target.x + 1;",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected follow-up addition");
    };
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "the pre-descriptor target snapshot must not retain Number-only property typing: {:?}",
        result.expr
    );
}

#[test]
fn later_descriptor_evaluation_invalidates_an_exact_function_target_shape() {
    let program = lower(
        "function target() {} target.x = 1; const alias = target; Object.defineProperty(alias, 'x', { value: (target.y = 0, 's') }); target.x + 1;",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected follow-up addition");
    };
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "the exact function target must not retain Number-only property typing: {:?}",
        result.expr
    );
}

#[test]
fn later_descriptor_evaluation_invalidates_a_named_global_dependency() {
    let create_realm = HostBuiltinId::CreateRealm
        .global_name()
        .expect("create realm must have a harness global name");
    let source = format!(
        "let cached = {create_realm}(); \
         function makeRealm() {{ return {create_realm}(); }} \
         Object.defineProperty(globalThis, '{create_realm}', {{ \
             value: (globalThis.definePropertySideEffect = 0, function () {{ return cached; }}) \
         }}); \
         delete cached.evalScript; \
         makeRealm().evalScript('source');"
    );

    assert_realm_eval_is_not_typed(&lower_test262(&source));
}

#[test]
fn later_argument_prototype_change_invalidates_the_captured_descriptor_shape() {
    let create_realm = HostBuiltinId::CreateRealm
        .global_name()
        .expect("create realm must have a harness global name");
    let source = format!(
        "let other = {create_realm}(); \
         let descriptor = {{}}; \
         Object.defineProperty( \
             {{}}, \
             'x', \
             descriptor, \
             descriptor.__proto__ = {{ get get() {{ delete other.evalScript; }} }} \
         ); \
         other.evalScript('source');"
    );

    assert_realm_eval_is_not_typed(&lower_test262(&source));
}

#[test]
fn define_property_observes_a_descriptor_field_getter_receiver() {
    let program = lower(
        "Object.defineProperty({}, 'x', { marker: 1, get set() { return this.marker ? function selected(value) {} : undefined; } });",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let getter = script
        .functions
        .iter()
        .find(|function| function.protocol == FunctionProtocolIr::ObjectGetter)
        .expect("descriptor field getter should be lowered");
    let StatementIr::Return(result) = getter
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::Return(_)))
        .expect("descriptor field getter should return")
    else {
        unreachable!("selected statement is a return")
    };
    let ExprIr::Conditional { condition, .. } = &result.expr else {
        panic!("descriptor field getter should return its conditional: {result:?}");
    };
    assert_eq!(
        condition.possible_kinds,
        KindSet::from_kind(ValueKind::Number)
    );
}

#[test]
fn define_property_widens_planned_hooks_after_an_earlier_field_getter() {
    let program = lower(
        "function later() { 'use strict'; return typeof this; } \
         new Proxy({}, { has: later }); \
         Object.defineProperty({}, 'x', { \
             get enumerable() { return true; } \
         });",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let getter = script
        .functions
        .iter()
        .find(|function| {
            function.name == "later" && !function.id.contains("$exact_helper_context$")
        })
        .expect("planned source hook should be lowered");
    let StatementIr::Return(result) = getter
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::Return(_)))
        .expect("planned source hook should return")
    else {
        unreachable!("selected statement is a return")
    };
    let ExprIr::TypeOf { expr } = &result.expr else {
        panic!("planned source hook should return typeof this: {result:?}");
    };
    assert_eq!(
        expr.possible_kinds,
        KindSet::all_runtime_tags(),
        "an earlier descriptor field getter makes later hook provenance unknown"
    );
}

#[test]
fn define_property_observes_lost_proxy_define_property_hooks() {
    for invocation in [
        "Object.defineProperty(proxy, 'x', {})",
        "Reflect.defineProperty(proxy, 'x', {})",
        "Object.defineProperty.call(null, proxy, 'x', {})",
        "Reflect.defineProperty.call(null, proxy, 'x', {})",
        "Object.defineProperty.apply(null, [proxy, 'x', {}])",
        "Array.prototype[0] = proxy; Object.defineProperty.apply(null, [, 'x', {}])",
        "Reflect.apply(Reflect.defineProperty, null, [proxy, 'x', {}])",
        "Object.defineProperty?.(proxy, 'x', {})",
        "Reflect.defineProperty(...[proxy, 'x', {}])",
        "const forwarded = globalThis.chooseDefineProperty ? Object.defineProperty : Reflect.defineProperty; forwarded(proxy, 'x', {})",
        "const bound = Object.defineProperty.bind(null); bound(proxy, 'x', {})",
        "const erased = new Proxy(Reflect.defineProperty, {}); erased(proxy, 'x', {})",
        "function ordinary() {} function invoke(fn, target) { fn?.(target, 'x', {}); } invoke.bind(null, ordinary, {}); invoke.bind(null, new Proxy(Reflect.defineProperty, {}), proxy)",
        "function ordinary() {} function invoke(fn, target) { if (typeof fn === 'function') fn?.(target, 'x', {}); } invoke.bind(null, ordinary, {}); invoke.bind(null, new Proxy(Reflect.defineProperty, {}), proxy)",
        "function ordinary() {} function invoke(fn, target) { ({ invoke: fn }).invoke(target, 'x', {}); } invoke.bind(null, ordinary, {}); invoke.bind(null, new Proxy(Reflect.defineProperty, {}), proxy)",
        "function ordinary() {} function invoke(fn, target) { Reflect.apply(fn, null, [target, 'x', {}]); } invoke.bind(null, ordinary, {}); invoke.bind(null, new Proxy(Reflect.defineProperty, {}), proxy)",
    ] {
        let program = lower(&format!(
            "function trap() {{ 'use strict'; return this.marker + 1; }} \
             const observedHandler = {{ marker: 'observed', defineProperty: trap }}; \
             new Proxy({{}}, observedHandler); \
             const lostHandler = {{ marker: 1 }}; \
             const proxy = new Proxy({{}}, lostHandler); \
             lostHandler.defineProperty = trap; \
             {invocation};"
        ));
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let trap = script
            .functions
            .iter()
            .find(|function| {
                function.name == "trap" && !function.id.contains("$exact_helper_context$")
            })
            .expect("Proxy trap should be lowered");
        let StatementIr::Return(result) = trap
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("Proxy trap should return")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "{invocation} must widen the receiver of a possible Proxy hook: {result:?}"
        );
    }
}

#[test]
fn define_property_optional_call_with_erased_callable_provenance_has_dynamic_result() {
    let program = lower(
        "function ordinary() { return 'ordinary'; } \
         function invoke(fn, target) { return fn?.(target, 'x', {}); } \
         const proxy = new Proxy({}, {}); \
         invoke.bind(null, ordinary, {}); \
         invoke.bind(null, new Proxy(Reflect.defineProperty, {}), proxy);",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let invoke = script
        .functions
        .iter()
        .find(|function| {
            function.name == "invoke" && !function.id.contains("$exact_helper_context$")
        })
        .expect("mixed-provenance optional caller should be lowered");
    let StatementIr::Return(result) = invoke
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::Return(_)))
        .expect("mixed-provenance optional caller should return")
    else {
        unreachable!("selected statement is a return")
    };
    assert_eq!(
        result.possible_kinds,
        KindSet::all_runtime_tags(),
        "a callable Proxy can return a kind absent from retained ordinary function targets"
    );
}

#[test]
fn date_to_primitive_result_excludes_callable_values() {
    let program =
        lower("const toPrimitive = Date.prototype[Symbol.toPrimitive]; toPrimitive(\"default\");");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("Date.prototype[Symbol.toPrimitive] should be an expression statement");
    };
    assert_eq!(result.possible_kinds, KindSet::PRIMITIVE_ONLY, "{result:?}");
    assert!(result
        .function_targets
        .exact_targets()
        .is_some_and(BTreeSet::is_empty));
}

fn source_function<'a>(program: &'a ProgramIr, name: &str) -> &'a FunctionIr {
    program
        .script
        .as_ref()
        .expect("script IR should exist")
        .functions
        .iter()
        .find(|function| function.name == name && !function.id.contains("$exact_helper_context$"))
        .unwrap_or_else(|| panic!("missing source function {name}"))
}

fn returned_expression(function: &FunctionIr) -> &TypedExpr {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("function {} should return", function.name))
}

fn assert_final_addition_preserves_caller_flow(source: &str) {
    let program = lower(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected final addition: {source}");
    };
    assert!(
        matches!(
            result.expr,
            ExprIr::BinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            } | ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            }
        ),
        "{source}: {result:?}"
    );
}

#[test]
fn ordinary_promise_call_with_a_callable_executor_preserves_caller_flow() {
    let source = "let holder = { value: 1 }; function executor(resolve) { holder = {}; return resolve(0); } Promise(executor); holder.value + 1;";
    let program = lower(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected final addition");
    };
    assert!(matches!(
        result.expr,
        ExprIr::BinaryNumber {
            op: ArithmeticBinaryOp::Add,
            ..
        } | ExprIr::CoerciveBinaryNumber {
            op: ArithmeticBinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn promise_construction_with_a_callable_executor_invalidates_caller_flow() {
    let source = "let holder = { value: 1 }; function executor(resolve) { holder = {}; return resolve(0); } new Promise(executor); holder.value + 1;";
    let program = lower(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected final addition");
    };
    assert!(matches!(result.expr, ExprIr::CoerciveAdd { .. }));
}

#[test]
fn promise_construction_with_an_omitted_executor_preserves_caller_flow() {
    assert_final_addition_preserves_caller_flow(
        "const holder = { value: 1 }; new Promise(); holder.value + 1;",
    );
}

#[test]
fn promise_construction_with_a_primitive_executor_preserves_caller_flow() {
    assert_final_addition_preserves_caller_flow(
        "const holder = { value: 1 }; new Promise(0); holder.value + 1;",
    );
}

#[test]
fn promise_resolve_function_with_a_missing_resolution_preserves_caller_flow() {
    let source = "function executor(resolve) { const holder = { value: 1 }; resolve(); return holder.value + 1; } new Promise(executor);";
    let program = lower(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let result = returned_expression(source_function(&program, "executor"));
    assert!(
        matches!(
            result.expr,
            ExprIr::BinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            } | ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            }
        ),
        "{source}: {result:?}"
    );
}

#[test]
fn promise_resolve_function_with_a_primitive_resolution_preserves_caller_flow() {
    let source = "function executor(resolve) { const holder = { value: 1 }; resolve(0); return holder.value + 1; } new Promise(executor);";
    let program = lower(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let result = returned_expression(source_function(&program, "executor"));
    assert!(
        matches!(
            result.expr,
            ExprIr::BinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            } | ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Add,
                ..
            }
        ),
        "{source}: {result:?}"
    );
}

#[test]
fn promise_resolve_function_with_an_object_resolution_invalidates_caller_flow() {
    let source = "function executor(resolve) { const holder = { value: 1 }; resolve({}); return holder.value + 1; } new Promise(executor);";
    let program = lower(source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let result = returned_expression(source_function(&program, "executor"));
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "{source}: {result:?}"
    );
}
