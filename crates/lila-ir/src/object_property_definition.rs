//! One ordered property definition on an already allocated object literal.
use crate::{KindSet, ObjectPropertyIr, TypedExpr, ValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectPropertyDefinitionError {
    NonObjectTarget,
}

/// The compiler retains the literal's actual object across suspensions. Private
/// fields prevent a property definition from targeting a non-object expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPropertyDefinitionIr {
    target: TypedExpr,
    property: ObjectPropertyIr,
}

impl ObjectPropertyDefinitionIr {
    pub(crate) fn new(
        target: TypedExpr,
        property: ObjectPropertyIr,
    ) -> Result<Self, ObjectPropertyDefinitionError> {
        if target.kind != ValueKind::Object
            || target.possible_kinds != KindSet::from_kind(ValueKind::Object)
        {
            return Err(ObjectPropertyDefinitionError::NonObjectTarget);
        }
        Ok(Self { target, property })
    }

    pub fn target(&self) -> &TypedExpr {
        &self.target
    }

    pub fn property(&self) -> &ObjectPropertyIr {
        &self.property
    }
}
