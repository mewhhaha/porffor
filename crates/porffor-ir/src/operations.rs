#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Let,
    Const,
    Var,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryNumericOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseBinaryOp {
    And,
    Or,
    Xor,
    Shl,
    Shr,
    UShr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBinaryOp {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityBinaryOp {
    StrictEqual,
    StrictNotEqual,
    LooseEqual,
    LooseNotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToPrimitiveHint {
    Default,
    Number,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalBinaryOp {
    And,
    Or,
    Coalesce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericUpdateOp {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateReturnMode {
    Prefix,
    Postfix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecOperationFamily {
    TypeQuery,
    Conversion,
    Comparison,
    Object,
    Invocation,
    Iterator,
    Completion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionAbruptKind {
    Throw,
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationLoweringStatus {
    CatalogOnly,
    SharedWasmEmitter,
    TrackedGap(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecOperationIr {
    ToBoolean,
}

impl SpecOperationIr {
    pub const fn name(self) -> &'static str {
        match self {
            Self::ToBoolean => "ToBoolean",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecOperationCatalogEntry {
    pub name: &'static str,
    pub family: SpecOperationFamily,
    pub normal_result: &'static str,
    pub abrupt: &'static [CompletionAbruptKind],
    pub lowering_status: OperationLoweringStatus,
}

const NO_ABRUPT: &[CompletionAbruptKind] = &[];
const MAY_THROW: &[CompletionAbruptKind] = &[CompletionAbruptKind::Throw];
const CONTROL_COMPLETIONS: &[CompletionAbruptKind] = &[
    CompletionAbruptKind::Throw,
    CompletionAbruptKind::Return,
    CompletionAbruptKind::Break,
    CompletionAbruptKind::Continue,
];

pub const SPEC_OPERATION_CATALOG: &[SpecOperationCatalogEntry] = &[
    op("Type", SpecOperationFamily::TypeQuery, "Type", NO_ABRUPT),
    op(
        "IsCallable",
        SpecOperationFamily::TypeQuery,
        "Boolean",
        NO_ABRUPT,
    ),
    op(
        "IsConstructor",
        SpecOperationFamily::TypeQuery,
        "Boolean",
        NO_ABRUPT,
    ),
    op(
        "IsPropertyKey",
        SpecOperationFamily::TypeQuery,
        "Boolean",
        NO_ABRUPT,
    ),
    op(
        "ToPrimitive",
        SpecOperationFamily::Conversion,
        "ECMAScript language value",
        MAY_THROW,
    ),
    lowered_op(
        "ToBoolean",
        SpecOperationFamily::Conversion,
        "Boolean",
        NO_ABRUPT,
        OperationLoweringStatus::SharedWasmEmitter,
    ),
    op(
        "ToNumeric",
        SpecOperationFamily::Conversion,
        "Number or BigInt",
        MAY_THROW,
    ),
    op(
        "ToNumber",
        SpecOperationFamily::Conversion,
        "Number",
        MAY_THROW,
    ),
    op(
        "ToBigInt",
        SpecOperationFamily::Conversion,
        "BigInt",
        MAY_THROW,
    ),
    op(
        "ToString",
        SpecOperationFamily::Conversion,
        "String",
        MAY_THROW,
    ),
    op(
        "ToObject",
        SpecOperationFamily::Conversion,
        "Object",
        MAY_THROW,
    ),
    op(
        "ToPropertyKey",
        SpecOperationFamily::Conversion,
        "PropertyKey",
        MAY_THROW,
    ),
    op(
        "ToIntegerOrInfinity",
        SpecOperationFamily::Conversion,
        "Number",
        MAY_THROW,
    ),
    op(
        "ToLength",
        SpecOperationFamily::Conversion,
        "Integer",
        MAY_THROW,
    ),
    op(
        "ToIndex",
        SpecOperationFamily::Conversion,
        "Integer",
        MAY_THROW,
    ),
    op(
        "IntegerIndexedConversion",
        SpecOperationFamily::Conversion,
        "Integer",
        MAY_THROW,
    ),
    op(
        "SameValue",
        SpecOperationFamily::Comparison,
        "Boolean",
        NO_ABRUPT,
    ),
    op(
        "SameValueZero",
        SpecOperationFamily::Comparison,
        "Boolean",
        NO_ABRUPT,
    ),
    op(
        "StrictEqualityComparison",
        SpecOperationFamily::Comparison,
        "Boolean",
        NO_ABRUPT,
    ),
    op(
        "IsLooselyEqual",
        SpecOperationFamily::Comparison,
        "Boolean",
        MAY_THROW,
    ),
    op(
        "IsLessThan",
        SpecOperationFamily::Comparison,
        "Boolean or Undefined",
        MAY_THROW,
    ),
    op(
        "Get",
        SpecOperationFamily::Object,
        "ECMAScript language value",
        MAY_THROW,
    ),
    op(
        "GetV",
        SpecOperationFamily::Object,
        "ECMAScript language value",
        MAY_THROW,
    ),
    op("Set", SpecOperationFamily::Object, "Boolean", MAY_THROW),
    op(
        "HasProperty",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
    ),
    op(
        "HasOwnProperty",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
    ),
    op(
        "DeletePropertyOrThrow",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
    ),
    op(
        "CreateDataProperty",
        SpecOperationFamily::Object,
        "Boolean",
        MAY_THROW,
    ),
    op(
        "CreateDataPropertyOrThrow",
        SpecOperationFamily::Object,
        "Unused",
        MAY_THROW,
    ),
    op(
        "DefinePropertyOrThrow",
        SpecOperationFamily::Object,
        "Unused",
        MAY_THROW,
    ),
    op(
        "ToPropertyDescriptor",
        SpecOperationFamily::Object,
        "PropertyDescriptor",
        MAY_THROW,
    ),
    op(
        "FromPropertyDescriptor",
        SpecOperationFamily::Object,
        "Object or Undefined",
        MAY_THROW,
    ),
    op(
        "GetMethod",
        SpecOperationFamily::Invocation,
        "Callable or Undefined",
        MAY_THROW,
    ),
    op(
        "Call",
        SpecOperationFamily::Invocation,
        "ECMAScript language value",
        MAY_THROW,
    ),
    op(
        "Construct",
        SpecOperationFamily::Invocation,
        "Object",
        MAY_THROW,
    ),
    op(
        "OrdinaryCreateFromConstructor",
        SpecOperationFamily::Invocation,
        "Object",
        MAY_THROW,
    ),
    op(
        "SpeciesConstructor",
        SpecOperationFamily::Invocation,
        "Constructor",
        MAY_THROW,
    ),
    op(
        "ArraySpeciesCreate",
        SpecOperationFamily::Invocation,
        "Array",
        MAY_THROW,
    ),
    op(
        "GetIterator",
        SpecOperationFamily::Iterator,
        "IteratorRecord",
        MAY_THROW,
    ),
    op(
        "IteratorStep",
        SpecOperationFamily::Iterator,
        "Object or false",
        MAY_THROW,
    ),
    op(
        "IteratorValue",
        SpecOperationFamily::Iterator,
        "ECMAScript language value",
        MAY_THROW,
    ),
    op(
        "IteratorClose",
        SpecOperationFamily::Iterator,
        "Completion",
        CONTROL_COMPLETIONS,
    ),
    op(
        "AsyncIteratorClose",
        SpecOperationFamily::Iterator,
        "Completion",
        CONTROL_COMPLETIONS,
    ),
    op(
        "Completion",
        SpecOperationFamily::Completion,
        "Completion Record",
        CONTROL_COMPLETIONS,
    ),
    op(
        "UpdateEmpty",
        SpecOperationFamily::Completion,
        "Completion Record",
        CONTROL_COMPLETIONS,
    ),
];

const fn op(
    name: &'static str,
    family: SpecOperationFamily,
    normal_result: &'static str,
    abrupt: &'static [CompletionAbruptKind],
) -> SpecOperationCatalogEntry {
    SpecOperationCatalogEntry {
        name,
        family,
        normal_result,
        abrupt,
        lowering_status: OperationLoweringStatus::TrackedGap("T04"),
    }
}

const fn lowered_op(
    name: &'static str,
    family: SpecOperationFamily,
    normal_result: &'static str,
    abrupt: &'static [CompletionAbruptKind],
    lowering_status: OperationLoweringStatus,
) -> SpecOperationCatalogEntry {
    SpecOperationCatalogEntry {
        name,
        family,
        normal_result,
        abrupt,
        lowering_status,
    }
}

pub fn spec_operation_catalog() -> &'static [SpecOperationCatalogEntry] {
    SPEC_OPERATION_CATALOG
}

pub fn find_spec_operation(name: &str) -> Option<&'static SpecOperationCatalogEntry> {
    SPEC_OPERATION_CATALOG
        .iter()
        .find(|entry| entry.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const REQUIRED_T04_OPERATIONS: &[&str] = &[
        "Type",
        "IsCallable",
        "IsConstructor",
        "IsPropertyKey",
        "ToPrimitive",
        "ToBoolean",
        "ToNumeric",
        "ToNumber",
        "ToBigInt",
        "ToString",
        "ToObject",
        "ToPropertyKey",
        "ToIntegerOrInfinity",
        "ToLength",
        "ToIndex",
        "IntegerIndexedConversion",
        "SameValue",
        "SameValueZero",
        "StrictEqualityComparison",
        "IsLooselyEqual",
        "IsLessThan",
        "Get",
        "GetV",
        "Set",
        "HasProperty",
        "HasOwnProperty",
        "DeletePropertyOrThrow",
        "CreateDataProperty",
        "CreateDataPropertyOrThrow",
        "DefinePropertyOrThrow",
        "ToPropertyDescriptor",
        "FromPropertyDescriptor",
        "GetMethod",
        "Call",
        "Construct",
        "OrdinaryCreateFromConstructor",
        "SpeciesConstructor",
        "ArraySpeciesCreate",
        "GetIterator",
        "IteratorStep",
        "IteratorValue",
        "IteratorClose",
        "AsyncIteratorClose",
        "Completion",
        "UpdateEmpty",
    ];

    #[test]
    fn operations_catalog_covers_t04_required_operations() {
        for required in REQUIRED_T04_OPERATIONS {
            assert!(
                find_spec_operation(required).is_some(),
                "missing T04 operation catalog entry: {required}"
            );
        }
    }

    #[test]
    fn operations_catalog_names_are_unique() {
        let mut names = BTreeSet::new();
        for entry in SPEC_OPERATION_CATALOG {
            assert!(
                names.insert(entry.name),
                "duplicate operation {}",
                entry.name
            );
        }
    }

    #[test]
    fn operations_catalog_tracks_every_gap_or_shared_lowering() {
        let mut lowered = BTreeSet::new();
        for entry in SPEC_OPERATION_CATALOG {
            assert!(!entry.normal_result.is_empty(), "{} result", entry.name);
            match entry.lowering_status {
                OperationLoweringStatus::SharedWasmEmitter => {
                    lowered.insert(entry.name);
                }
                OperationLoweringStatus::TrackedGap(task) => {
                    assert_eq!(task, "T04", "{} owner", entry.name);
                }
                OperationLoweringStatus::CatalogOnly => {
                    panic!(
                        "{} must be explicitly tracked before implementation",
                        entry.name
                    );
                }
            }
        }
        assert!(lowered.contains("ToBoolean"));
    }

    #[test]
    fn operations_catalog_marks_abrupt_capable_operations() {
        for name in ["ToPrimitive", "Get", "Call", "IteratorClose", "Completion"] {
            let entry = find_spec_operation(name).expect("operation should exist");
            assert!(
                !entry.abrupt.is_empty(),
                "{name} must document possible abrupt completions"
            );
        }
        let same_value = find_spec_operation("SameValue").expect("SameValue should exist");
        assert!(same_value.abrupt.is_empty());
    }
}
