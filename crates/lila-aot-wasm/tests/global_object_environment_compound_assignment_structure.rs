use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ArithmeticBinaryOp, BitwiseBinaryOp, EnvironmentCompoundOperationIr,
    EnvironmentIdentifierOperationIr, EnvironmentIdentifierResolutionStart, ExprIr, StatementIr,
    TypedExpr,
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
const REFERENCE_SOURCE: &str = include_str!("../../lila-ir/src/reference.rs");
const FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_global_object_environment_compound_assignment.js"
);
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/global-object-environment-eager-compound-assignment-reference.md"
);

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const SELECTED_WITNESSES: [(&str, &str); 11] = [
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

const ADJACENT_PREFIX_WITNESSES: [(&str, &str); 22] = [
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v-.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--1.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--2.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--3.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--4.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--5.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--6.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--7.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--8.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--9.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--10.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--11.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--12.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--13.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--14.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--15.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--16.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--17.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--18.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--19.js"
    ),
    witness!(
        "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v--20.js"
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
fn global_eager_assignments_retain_global_resolution_in_root_and_nested_owners() {
    let program = lower_control("var value = 1; value += 2; function mutate() { value += 3; }");
    let script = program.script.as_ref().expect("script IR");
    let nested = script
        .functions
        .iter()
        .find(|function| function.name == "mutate")
        .expect("nested owner");
    for (body, expected_rhs) in [(&script.body, 2.0f64), (&nested.body, 3.0f64)] {
        let reference = body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::EnvironmentIdentifier(reference),
                    ..
                }) => Some(reference),
                _ => None,
            })
            .expect("eager assignment retains one Environment Reference");
        assert_eq!(reference.name, "value");
        assert_eq!(
            reference.resolution_start(),
            EnvironmentIdentifierResolutionStart::GlobalEnvironment
        );
        assert!(matches!(&reference.operation,
            EnvironmentIdentifierOperationIr::EagerCompound { operation: EnvironmentCompoundOperationIr::Add, rhs }
            if matches!(&rhs.expr, ExprIr::Number(value) if *value == expected_rhs.to_bits())));
    }
}

#[test]
fn shared_sealed_lifecycle_rechecks_get_and_put_before_exposing_result() {
    let objects = bounded(
        REFERENCE_SOURCE,
        "impl ObjectEnvironmentBindingObject {",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    let get = bounded(
        objects,
        "    fn get_value(self, referenced_name: &str, strictness: Strictness) -> TypedExpr {",
        "    /// SetMutableBinding on the Object Environment Record selected before RHS.",
    );
    assert!(get.contains("let recheck = self.has_property(referenced_name);"));
    assert!(get.contains("ExprIr::PropertyRead"));
    assert!(get.contains("Strictness::Sloppy => TypedExpr::undefined()"));
    assert!(get.contains("Strictness::Strict => TypedExpr::from_info("));
    assert!(get.contains("name: NativeErrorKind::ReferenceError"));
    assert_before(get, "let recheck =", "let read =");

    let put = bounded(
        objects,
        "    fn put_value(",
        "    /// GetValue, eager operation, same-base PutValue, then result.",
    );
    assert!(put.contains("let recheck = self.has_property(referenced_name);"));
    assert!(put.contains("ExprIr::PropertyWrite"));
    assert!(put.contains("Strictness::Sloppy => TypedExpr::from_info("));
    assert!(put.contains("value: Box::new(recheck)"));
    assert!(put.contains("Strictness::Strict => TypedExpr::from_info("));
    assert!(put.contains("condition: Box::new(recheck)"));
    assert!(put.contains("name: NativeErrorKind::ReferenceError"));
    assert_before(put, "let recheck =", "let write =");

    let lifecycle = bounded(
        REFERENCE_SOURCE,
        "    fn eager_compound_assignment(",
        "/// Declarative-frame depth in the function currently being lowered.",
    );
    for marker in [
        "EagerCompoundAssignmentBindings {",
        "old_value: old_value_name",
        "result: result_name",
        "write: write_name",
        "let old_value = self.clone().get_value(referenced_name, strictness);",
        "let write = self.put_value(referenced_name, strictness, result.clone());",
        "name: write_name.clone()",
        "name: result_name.clone()",
        "name: old_value_name.clone()",
    ] {
        assert!(
            lifecycle.contains(marker),
            "missing lifecycle boundary: {marker}"
        );
    }
    assert_before(lifecycle, "let old_value =", "let result_info =");
    assert_before(lifecycle, "let result_info =", "let write =");
    assert_before(lifecycle, "let write =", "let after_write =");
    assert_before(lifecycle, "let after_write =", "let after_apply =");

    let carrier = bounded(
        REFERENCE_SOURCE,
        "pub(crate) struct EagerCompoundAssignmentBindings {",
        "impl NumericUpdateBindings {",
    );
    for marker in [
        "old_value: String",
        "result: String",
        "write: String",
        "pub(crate) fn allocate(",
        "allocate(\"object.environment.compound.old.\")",
        "allocate(\"object.environment.compound.result.\")",
        "allocate(\"object.environment.compound.write.\")",
        "pub(crate) fn old_value(&self) -> TypedExpr",
        "pub(crate) fn seal(self, result: TypedExpr) -> EagerCompoundAssignment",
        "pub(crate) struct EagerCompoundAssignment {",
    ] {
        assert!(
            carrier.contains(marker),
            "missing carrier boundary: {marker}"
        );
    }
    assert!(carrier.contains(
        "#[must_use = \"a sealed eager compound assignment must consume its Reference plan\"]"
    ));
}

#[test]
fn all_eager_operators_preserve_the_rhs_and_global_reference_owner() {
    use EnvironmentCompoundOperationIr as Operation;
    for (spelling, expected) in [
        ("+=", Operation::Add),
        ("-=", Operation::Arithmetic(ArithmeticBinaryOp::Sub)),
        ("*=", Operation::Arithmetic(ArithmeticBinaryOp::Mul)),
        ("/=", Operation::Arithmetic(ArithmeticBinaryOp::Div)),
        ("%=", Operation::Arithmetic(ArithmeticBinaryOp::Mod)),
        ("**=", Operation::Arithmetic(ArithmeticBinaryOp::Exp)),
        ("&=", Operation::Bitwise(BitwiseBinaryOp::And)),
        ("|=", Operation::Bitwise(BitwiseBinaryOp::Or)),
        ("^=", Operation::Bitwise(BitwiseBinaryOp::Xor)),
        ("<<=", Operation::Bitwise(BitwiseBinaryOp::Shl)),
        (">>=", Operation::Bitwise(BitwiseBinaryOp::Shr)),
        (">>>=", Operation::Bitwise(BitwiseBinaryOp::UShr)),
    ] {
        for declaration in ["", "var value = 7;"] {
            let program = lower_control(&format!("{declaration} value {spelling} 2;"));
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
                panic!("{spelling} must keep ResolveBinding/Get/RHS/Put together");
            };
            assert_eq!(
                reference.resolution_start(),
                EnvironmentIdentifierResolutionStart::GlobalEnvironment
            );
            let EnvironmentIdentifierOperationIr::EagerCompound { operation, rhs } =
                &reference.operation
            else {
                panic!("expected eager operation for {spelling}");
            };
            assert_eq!(*operation, expected);
            assert!(matches!(&rhs.expr, ExprIr::Number(value) if *value == 2.0f64.to_bits()));
        }
    }
}

#[test]
fn consumer_and_exact_current_pin_inventory_cover_the_durable_contract() {
    for marker in [
        "globalXorValue ^= compoundRhs()",
        "globalOrValue |= compoundRhs()",
        "globalMulValue *= compoundRhs()",
        "globalDivValue /= compoundRhs()",
        "globalModValue %= compoundRhs()",
        "globalAddValue += compoundRhs()",
        "globalSubValue -= compoundRhs()",
        "globalShlValue <<= compoundRhs()",
        "globalShrValue >>= compoundRhs()",
        "globalUshrValue >>>= compoundRhs()",
        "globalAndValue &= compoundRhs()",
        "strictResult === \"not written\"",
        "initiallyAbsentGlobal += absentRhs()",
        "absentRhsCount === 0",
        "let sloppyResult = sloppyGlobalValue += sloppyRhs()",
        "sloppyTrace === \"gr\"",
        "Object.prototype.hasOwnProperty.call(globalThis, \"sloppyGlobalValue\")",
        "let inheritedResult = inheritedGlobalValue -= 2",
        "Object.prototype.hasOwnProperty.call(globalThis, \"inheritedGlobalValue\")",
    ] {
        assert!(FIXTURE.contains(marker), "missing CLI witness: {marker}");
    }

    assert_eq!(SELECTED_WITNESSES.len(), 11);
    for (path, source) in SELECTED_WITNESSES {
        assert!(path.ends_with(".js"));
        assert!(source.contains("flags: [noStrict]"), "missing flag: {path}");
        assert!(
            source.contains("Object.defineProperty(this, \"x\""),
            "{path}"
        );
        assert!(source.contains("\"use strict\""), "{path}");
        assert!(source.contains("assert.throws(ReferenceError"), "{path}");
        assert!(
            !source.contains("with (scope)"),
            "global witness used with: {path}"
        );
        assert!(
            CONTRACT.contains(path.rsplit('/').next().expect("file name")),
            "{path}"
        );
    }

    assert_eq!(ADJACENT_PREFIX_WITNESSES.len(), 22);
    for (index, (path, source)) in ADJACENT_PREFIX_WITNESSES.into_iter().enumerate() {
        assert!(source.contains("flags: [noStrict]"), "missing flag: {path}");
        if index == 0 || index % 2 == 0 {
            assert!(
                source.contains("with (scope)"),
                "with regression witness: {path}"
            );
        } else {
            assert!(
                source.contains("Object.defineProperty(this, \"x\""),
                "{path}"
            );
        }
    }

    assert!(CONTRACT.contains("The producer's closed eager domain also includes `**=`"));
    assert!(CONTRACT.contains("Logical assignments, property References, declarative bindings"));
    assert!(CONTRACT.contains("subtree and pinned matrix remain later verification checkpoints."));
}
