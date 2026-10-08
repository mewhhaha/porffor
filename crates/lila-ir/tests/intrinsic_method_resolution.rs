//! Static method resolution against intrinsic prototypes is a proof, not a
//! spelling. An unmodified program keeps the exact builtin callee (and with it
//! the fast path and the builtin's result kind); once the program replaces the
//! method, the call is an ordinary property read and call of whatever is
//! installed, with no claimed result kind.
use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, PropertyKeyIr, ScriptIr, StandardBuiltinId, StatementIr, ValueKind};

const INTRINSIC_METHOD_SOURCE: &str = include_str!("../src/lowering/intrinsic_method.rs");
const CALL_SOURCE: &str = include_str!("../src/lowering/call_expression.rs");
const PROPERTY_ACCESS_SOURCE: &str = include_str!("../src/lowering/property_access.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

fn lower_script(source: &'static str) -> ScriptIr {
    std::thread::Builder::new()
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            let parsed = parse(source, ParseOptions::script()).expect("script should parse");
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            program.script.expect("script IR")
        })
        .expect("worker thread should spawn")
        .join()
        .expect("lowering should not panic")
}

fn body(script: &ScriptIr) -> String {
    format!("{:?}", script.body)
}

const EXACT_APPLY_CALLEE: &str = "FunctionValue(\"$builtin.Function.prototype.apply\")";

#[test]
fn function_apply_keeps_its_acquired_callee_and_possible_builtin() {
    let script = lower_script("function f(a) { return a; } f.apply(null, [1]);");
    let body = body(&script);
    assert!(body.contains("PropertyRead"), "{body}");
    assert!(body.contains("StaticString(\"apply\")"), "{body}");
    assert!(
        body.contains("Open({\"$builtin.Function.prototype.apply\"})"),
        "{body}"
    );
    assert!(!body.contains(EXACT_APPLY_CALLEE), "{body}");
}

#[test]
fn replaced_function_prototype_apply_is_read_and_called_as_a_property() {
    let script = lower_script(
        "Function.prototype.apply = function () { return 'x'; };\n\
         function f(a) { return a; } f.apply(null, [1]);",
    );
    let body = body(&script);
    assert!(!body.contains(EXACT_APPLY_CALLEE), "{body}");
    assert!(body.contains("PropertyRead"), "{body}");
    assert_ne!(script.result_kind(), ValueKind::Number);
}

#[test]
fn unmodified_string_char_code_at_keeps_its_acquired_native_reference() {
    let script = lower_script("'abc'.charCodeAt(1);");
    let StatementIr::Expression(result) = script.body.statements.last().expect("call") else {
        panic!("expected the character-code call");
    };
    let ExprIr::MaterializeBinding { name, value, body } = &result.expr else {
        panic!("expected one receiver evaluation: {result:?}");
    };
    assert!(matches!(&value.expr, ExprIr::String(value) if value == "abc"));
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(receiver),
        args,
        ..
    } = &body.expr
    else {
        panic!("expected the acquired callee: {body:?}");
    };
    assert!(matches!(&receiver.expr, ExprIr::Identifier(storage) if storage == name));
    let ExprIr::PropertyRead {
        target,
        key: PropertyKeyIr::StaticString(key),
    } = &callee.expr
    else {
        panic!("expected the original property read: {callee:?}");
    };
    assert!(matches!(&target.expr, ExprIr::Identifier(storage) if storage == name));
    assert_eq!(key, "charCodeAt");
    assert_eq!(
        callee.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::StringPrototypeCharCodeAt.function_id())
    );
    let [index] = args.as_slice() else {
        panic!("{args:?}")
    };
    assert!(matches!(index.expr, ExprIr::Number(value) if f64::from_bits(value) == 1.0));
    assert_eq!(script.result_kind(), ValueKind::Number);
}

#[test]
fn replaced_string_char_code_at_is_called_through_the_prototype() {
    let script = lower_script(
        "String.prototype.charCodeAt = function () { return 'c'; };\n'abc'.charCodeAt(1);",
    );
    let StatementIr::Expression(result) = script.body.statements.last().expect("call") else {
        panic!("expected the replaced character-code call");
    };
    let ExprIr::MaterializeBinding { name, value, body } = &result.expr else {
        panic!("expected one receiver evaluation: {result:?}");
    };
    assert!(matches!(&value.expr, ExprIr::String(value) if value == "abc"));
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(receiver),
        args,
        ..
    } = &body.expr
    else {
        panic!("expected a call of the replaced property: {body:?}");
    };
    assert!(matches!(&receiver.expr, ExprIr::Identifier(storage) if storage == name));
    let ExprIr::PropertyRead {
        target,
        key: PropertyKeyIr::StaticString(key),
    } = &callee.expr
    else {
        panic!("expected the runtime property read: {callee:?}");
    };
    assert!(matches!(&target.expr, ExprIr::Identifier(storage) if storage == name));
    assert_eq!(key, "charCodeAt");
    assert_ne!(
        callee.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::StringPrototypeCharCodeAt.function_id())
    );
    let [index] = args.as_slice() else {
        panic!("{args:?}")
    };
    assert!(matches!(index.expr, ExprIr::Number(value) if f64::from_bits(value) == 1.0));
    assert_eq!(script.result_kind(), ValueKind::Dynamic);
}

#[test]
fn a_receiver_that_only_may_be_a_function_resolves_no_function_prototype_method() {
    // After an opaque call every binding is widened; `TypeError(...)` is then
    // an arbitrary value and its `toString` is whatever it inherits.
    let script = lower_script(
        "function add(x, y) { return x + y; }\n\
         Error.prototype.toString = function () { return 8; };\n\
         add(2, 1);\nTypeError('y').toString();",
    );
    assert_ne!(script.result_kind(), ValueKind::String);
}

#[test]
fn fresh_builtin_instances_do_not_restore_replaced_prototype_result_facts() {
    for (source, expected_kind) in [
        ("Date.prototype.getTime = function () { return 'changed'; }; new Date(0).getTime();", ValueKind::String),
        ("Map.prototype.has = function () { return 'changed'; }; new Map().has(1);", ValueKind::String),
        ("Set.prototype.has = function () { return 'changed'; }; new Set().has(1);", ValueKind::String),
        ("Array.prototype.join = function () { return 42; }; [1].join();", ValueKind::Number),
        ("Temporal.PlainDate.prototype.add = function () { return 'changed'; }; new Temporal.PlainDate(2024, 1, 1).add({ days: 1 });", ValueKind::String),
        ("Object.defineProperty(Intl.Locale.prototype, 'baseName', { get() { return 42; } }); new Intl.Locale('en').baseName;", ValueKind::Number),
        ("Object.defineProperty(Uint8Array.prototype, 'constructor', { get() { return 42; } }); new Uint8Array(1).constructor;", ValueKind::Number),
    ] {
        let script = lower_script(source);
        let StatementIr::Expression(result) = script.body.statements.last().expect("property result") else {
            panic!("expected a property read or call");
        };
        assert!(result.possible_kinds.contains(expected_kind), "{source}: {result:?}");
    }
}

#[test]
fn absent_and_hole_array_indices_keep_the_inherited_value_domain() {
    for source in [
        "Array.prototype[1] = 'inherited'; [1][1];",
        "Array.prototype[0] = 'inherited'; [,][0];",
        "Array.prototype[1] = 'inherited'; const index = 1; [1][index];",
    ] {
        let script = lower_script(source);
        let StatementIr::Expression(result) = script.body.statements.last().expect("index read")
        else {
            panic!("expected an array read");
        };
        assert!(
            result.possible_kinds.contains(ValueKind::String),
            "{source}: {result:?}"
        );
    }
}

#[test]
fn an_uncalled_primitive_method_getter_invalidates_captured_property_facts() {
    let script = lower_script(
        "var state;\n\
         Object.defineProperty(Number.prototype, 'toFixed', {\n\
           configurable: true, get() { state.value = 'changed'; return 1; }\n\
         });\n\
         state = { value: 7 };\n\
         (1).toFixed;\n\
         state.value;",
    );
    assert_eq!(script.result_kind(), ValueKind::Dynamic);
}

#[test]
fn an_uncalled_string_custom_getter_invalidates_captured_property_facts() {
    let script = lower_script(
        "var state;\n\
         Object.defineProperty(String.prototype, 'custom', {\n\
           configurable: true, get() { state.value = 'changed'; return 1; }\n\
         });\n\
         state = { value: 7 };\n\
         'abc'.custom;\n\
         state.value;",
    );
    assert_eq!(script.result_kind(), ValueKind::Dynamic);
}

#[test]
fn an_undescribed_symbol_does_not_claim_a_string_description() {
    let script = lower_script("Symbol().description;");
    let StatementIr::Expression(result) = script.body.statements.last().expect("description")
    else {
        panic!("expected the Symbol description read");
    };
    assert!(result.possible_kinds.contains(ValueKind::Undefined));
    assert_ne!(result.kind, ValueKind::String);
}

#[test]
fn a_foreign_global_constructor_cannot_prove_the_local_primitive_prototype() {
    let script = lower_script(
        "const other = $262.createRealm().global;\n\
         Number.prototype.toString = function () { return 17; };\n\
         Number = other.Number;\n\
         (1).toString();",
    );
    let StatementIr::Expression(result) = script.body.statements.last().expect("primitive call")
    else {
        panic!("expected the primitive call");
    };
    assert!(
        result.possible_kinds.contains(ValueKind::Number),
        "{result:?}"
    );
}

#[test]
fn a_replaced_symbol_description_has_the_runtime_getter_result_domain() {
    let script = lower_script(
        "const s = Symbol.iterator;\n\
         Object.defineProperty(Symbol.prototype, 'description', {\n\
           configurable: true, get() { return 23; }\n\
         });\n\
         s.description;",
    );
    let StatementIr::Expression(result) = script.body.statements.last().expect("description")
    else {
        panic!("expected the Symbol description read");
    };
    assert!(result.possible_kinds.contains(ValueKind::Number));
    assert_ne!(result.kind, ValueKind::String);
}

#[test]
fn bigint_can_call_an_inherited_user_method() {
    let script =
        lower_script("BigInt.prototype.custom = function () { return this; }; (7n).custom();");
    assert!(body(&script).contains("PropertyRead"));
}

#[test]
fn replaced_regexp_protocols_do_not_claim_native_result_types() {
    let script = lower_script(
        "RegExp.prototype[Symbol.match] = function () { return 23; };\n\
         /x/[Symbol.match]('x');",
    );
    let StatementIr::Expression(result) = script.body.statements.last().expect("match call") else {
        panic!("expected the acquired regexp protocol call");
    };
    assert!(result.possible_kinds.contains(ValueKind::Number));
}

#[test]
fn replaced_array_iterators_do_not_claim_an_iterator_result() {
    let script = lower_script(
        "Array.prototype[Symbol.iterator] = function () { return 23; };\n\
         [1][Symbol.iterator]();",
    );
    let StatementIr::Expression(result) = script.body.statements.last().expect("iterator call")
    else {
        panic!("expected the acquired array iterator call");
    };
    assert!(result.possible_kinds.contains(ValueKind::Number));
}

/// Every name-indexed resolution to one of these prototypes' builtins has to
/// go through `IntrinsicPrototype::catalogued_method`, whose only consumer is
/// the live-prototype proof. A string-literal match arm naming such a builtin
/// anywhere in the call and property-read lowering would be a second table
/// that can skip the proof.
#[test]
fn prototype_method_names_are_resolved_only_by_the_intrinsic_catalogue() {
    const GATED_PROTOTYPES: [&str; 8] = [
        "StandardBuiltinId::StringPrototype",
        "StandardBuiltinId::NumberPrototype",
        "StandardBuiltinId::BooleanPrototype",
        "StandardBuiltinId::BigIntPrototype",
        "StandardBuiltinId::SymbolPrototype",
        "StandardBuiltinId::FunctionPrototype",
        "StandardBuiltinId::IteratorPrototype",
        "StandardBuiltinId::RegExpPrototype",
    ];
    for (file, source) in [
        ("call_expression.rs", CALL_SOURCE),
        ("property_access.rs", PROPERTY_ACCESS_SOURCE),
        ("lowering.rs", LOWERING_SOURCE),
    ] {
        let lines = source.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            // A string-literal *pattern*: `"name" =>`, or `"name"` followed by
            // a guard or an or-pattern line. A string literal that is itself an
            // arm body (builtin → name tables) is not one.
            let next = lines.get(index + 1).map_or("", |line| line.trim_start());
            let is_pattern = trimmed.starts_with('"')
                && (trimmed.contains("=>") || next.starts_with("if ") || next.starts_with('|'));
            if !is_pattern {
                continue;
            }
            // The arm runs from its `=>` until the next string pattern.
            let arm = lines[index..lines.len().min(index + 5)]
                .iter()
                .enumerate()
                .take_while(|(offset, line)| *offset == 0 || !line.trim_start().starts_with('"'))
                .map(|(_, line)| *line)
                .collect::<Vec<_>>()
                .join("\n");
            let Some((_, body)) = arm.split_once("=>") else {
                continue;
            };
            for prototype in GATED_PROTOTYPES {
                assert!(
                    !body.contains(prototype),
                    "{file}:{}: name-indexed `{prototype}*` resolution outside the \
                     intrinsic-method catalogue:\n{arm}",
                    index + 1
                );
            }
        }
    }
    // The catalogue itself is the single table.
    assert_eq!(
        INTRINSIC_METHOD_SOURCE
            .matches("pub(super) fn catalogued_method(")
            .count(),
        1
    );
}

/// The proof token cannot be built outside its module: its only field is
/// private and its one construction is the successful lookup.
#[test]
fn the_intrinsic_method_proof_has_one_constructor() {
    let declaration = INTRINSIC_METHOD_SOURCE
        .split_once("pub(super) struct IntrinsicMethod {")
        .expect("proof token declaration")
        .1
        .split_once('}')
        .expect("proof token fields")
        .0;
    assert_eq!(declaration.trim(), "builtin: StandardBuiltinId,");
    assert_eq!(
        INTRINSIC_METHOD_SOURCE
            .matches("IntrinsicMethod { builtin }")
            .count(),
        1
    );
    let derives = INTRINSIC_METHOD_SOURCE
        .split_once("pub(super) struct IntrinsicMethod {")
        .expect("proof token declaration")
        .0
        .rsplit_once("#[derive(")
        .expect("proof token derives")
        .1;
    assert!(!derives.contains("Default"));
}
