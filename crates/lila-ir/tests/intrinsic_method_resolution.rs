//! Static method resolution against intrinsic prototypes is a proof, not a
//! spelling. An unmodified program keeps the exact builtin callee (and with it
//! the fast path and the builtin's result kind); once the program replaces the
//! method, the call is an ordinary property read and call of whatever is
//! installed, with no claimed result kind.
use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ScriptIr, ValueKind};

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
fn unmodified_function_prototype_apply_keeps_its_exact_callee() {
    let script = lower_script("function f(a) { return a; } f.apply(null, [1]);");
    assert!(body(&script).contains(EXACT_APPLY_CALLEE));
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
fn unmodified_string_char_code_at_keeps_its_inline_form() {
    let script = lower_script("'abc'.charCodeAt(1);");
    assert!(body(&script).contains("StringCharCodeAt"));
    assert_eq!(script.result_kind(), ValueKind::Number);
}

#[test]
fn replaced_string_char_code_at_is_called_through_the_prototype() {
    let script = lower_script(
        "String.prototype.charCodeAt = function () { return 'c'; };\n'abc'.charCodeAt(1);",
    );
    assert!(!body(&script).contains("StringCharCodeAt"));
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
