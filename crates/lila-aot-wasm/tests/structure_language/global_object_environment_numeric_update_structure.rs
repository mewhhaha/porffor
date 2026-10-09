use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, EnvironmentIdentifierResolutionStart, ExprIr,
    NumericUpdateOp, StatementIr, TypedExpr, UpdateReturnMode,
};

fn lower_control(source: &str) -> lila_ir::ProgramIr {
    let parsed = parse(source, ParseOptions::script()).expect("Reference control parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
}
const REFERENCE_SOURCE: &str = include_str!("../../../lila-ir/src/reference.rs");
const FIXTURE: &str = include_str!(
    "../../../lila-cli/tests/fixtures/wasm_global_object_environment_numeric_update.js"
);
const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/global-object-environment-numeric-update-reference.md"
);

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 4] = [
    witness!(
        "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue--1.js"
    ),
    witness!(
        "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue--1.js"
    ),
    witness!(
        "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue--1.js"
    ),
    witness!(
        "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue--1.js"
    ),
];

const WITH_REGRESSION_WITNESSES: [(&str, &str); 4] = [
    witness!(
        "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue-.js"
    ),
    witness!(
        "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue-.js"
    ),
    witness!(
        "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue-.js"
    ),
    witness!(
        "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue-.js"
    ),
];

const GLOBAL_COMPOUND_REGRESSION_WITNESSES: [(&str, &str); 11] = [
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--1.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--3.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--5.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--7.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--9.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--11.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--13.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--15.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--17.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--19.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--21.js"
    ),
];

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
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "`{earlier}` must precede `{later}`");
}

#[test]
fn one_private_fixed_role_carrier_drives_the_shared_numeric_lifecycle() {
    let carrier = bounded(
        REFERENCE_SOURCE,
        "#[derive(Debug)]\npub(crate) struct NumericUpdateBindings {",
        "/// Compiler-private bindings for one eager Object Environment compound",
    );
    for marker in ["old_value: String", "result: String", "write: String"] {
        assert!(carrier.contains(marker), "missing fixed role: {marker}");
    }
    assert!(!carrier.contains("pub(crate) old_value"));
    assert!(!carrier.contains("Clone"));
    assert!(!carrier.contains("Copy"));

    let allocator = bounded(
        REFERENCE_SOURCE,
        "impl NumericUpdateBindings {",
        "impl WithEnvironmentReferencePlan {",
    );
    assert_before(
        allocator,
        "allocate(\"object.environment.update.old.\")",
        "allocate(\"object.environment.update.result.\")",
    );
    assert_before(
        allocator,
        "allocate(\"object.environment.update.result.\")",
        "allocate(\"object.environment.update.write.\")",
    );

    let objects = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let lifecycle = bounded(
        objects,
        "    fn numeric_update(",
        "    /// GetValue, eager operation, same-base PutValue, then result.",
    );
    for marker in [
        "let NumericUpdateBindings {",
        "let old_value = self.clone().get_value(referenced_name, strictness);",
        "ExprIr::UpdateIdentifier {",
        "value_kind: NumericUpdateValueKind::Dynamic",
        "let write = self.put_value(referenced_name, strictness, updated_value);",
        "name: write_name.clone()",
        "name: result_name.clone()",
        "name: old_value_name.clone()",
    ] {
        assert!(
            lifecycle.contains(marker),
            "missing shared lifecycle marker: {marker}"
        );
    }
    assert_before(lifecycle, "let old_value =", "let update =");
    assert_before(lifecycle, "let update =", "let updated_value =");
    assert_before(lifecycle, "let updated_value =", "let write =");
    assert_before(lifecycle, "let write =", "let result =");
    assert_before(lifecycle, "let result =", "let after_write =");
    assert_before(lifecycle, "let after_write =", "let after_update =");
}

#[test]
fn global_update_keeps_the_record_while_parameter_and_capture_updates_keep_storage() {
    let program = lower_control(
        "var value = 1; value++; function local(value) { value++; } function outer() { let captured = 1; return function inner() { captured++; }; }",
    );
    let script = program.script.as_ref().expect("script IR");
    let reference = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(reference),
                ..
            }) => Some(reference),
            _ => None,
        })
        .expect("a global var declaration does not grant local source storage");
    assert_eq!(reference.name, "value");
    assert_eq!(
        reference.resolution_start(),
        EnvironmentIdentifierResolutionStart::GlobalEnvironment
    );
    for (owner, binding) in [("local", "value"), ("inner", "captured")] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == owner)
            .expect("local/captured owner");
        assert!(function.body.statements.iter().any(|statement| matches!(statement,
            StatementIr::Expression(TypedExpr { expr: ExprIr::UpdateIdentifier { name, .. }, .. }) if name == binding)), "{function:?}");
    }
}

#[test]
fn all_numeric_modes_retain_global_resolution_for_declared_and_unknown_names() {
    for (source, expected_operation, expected_return) in [
        (
            "value++",
            NumericUpdateOp::Increment,
            UpdateReturnMode::Postfix,
        ),
        (
            "++value",
            NumericUpdateOp::Increment,
            UpdateReturnMode::Prefix,
        ),
        (
            "value--",
            NumericUpdateOp::Decrement,
            UpdateReturnMode::Postfix,
        ),
        (
            "--value",
            NumericUpdateOp::Decrement,
            UpdateReturnMode::Prefix,
        ),
    ] {
        for declaration in ["", "var value = 7;"] {
            let program = lower_control(&format!("{declaration} {source};"));
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(reference),
                ..
            }) = program
                .script
                .as_ref()
                .expect("script IR")
                .body
                .statements
                .last()
                .expect("source mutation")
            else {
                panic!("{source} must keep ResolveBinding/Get/coercion/Put together");
            };
            assert_eq!(
                reference.resolution_start(),
                EnvironmentIdentifierResolutionStart::GlobalEnvironment
            );
            assert!(matches!(&reference.operation,
                EnvironmentIdentifierOperationIr::Update { operation, return_mode }
                if *operation == expected_operation && *return_mode == expected_return));
        }
    }
}

#[test]
fn durable_consumer_and_exact_current_pin_inventories_bound_the_claim() {
    for (path, source) in SELECTED_WITNESSES {
        assert!(source.contains("flags: [noStrict]"), "{path}");
        assert!(
            source.contains("Object.defineProperty(this, \"x\""),
            "{path}"
        );
        assert!(source.contains("delete this.x;"), "{path}");
        assert!(source.contains("\"use strict\";"), "{path}");
        assert!(source.contains("assert.throws(ReferenceError"), "{path}");
        assert!(CONTRACT.contains(path), "contract omits {path}");
    }
    for (path, source) in WITH_REGRESSION_WITNESSES {
        assert!(source.contains("flags: [noStrict]"), "{path}");
        assert!(source.contains("with (scope)"), "{path}");
    }
    for (path, source) in GLOBAL_COMPOUND_REGRESSION_WITNESSES {
        assert!(source.contains("flags: [noStrict]"), "{path}");
        assert!(CONTRACT.contains("eleven global eager-compound files"));
    }

    for marker in [
        "++globalPrefixNumber",
        "globalPostfixNumber++",
        "--globalPrefixBigInt",
        "globalPostfixBigInt--",
        "++strictPrefixIncrement",
        "--strictPrefixDecrement",
        "strictPostfixIncrement++",
        "strictPostfixDecrement--",
        "sloppyPostfixNumber++",
        "--sloppyPrefixBigInt",
        "++initiallyMissingGlobalUpdate",
        "lifecycleTrace === \"h\"",
        "lifecycleTrace === \"hhgdnhs\"",
        "strictResult === \"not written\"",
    ] {
        assert!(FIXTURE.contains(marker), "fixture omits {marker}");
    }
    assert!(CONTRACT.contains("The plain assignment witness is not part of this cohort"));
    assert!(CONTRACT.contains("Logical assignments have a separate short-circuit lifecycle"));
    assert!(CONTRACT.contains("Broad language and pinned-matrix publication remain"));
}
