//! Captured operands for a suspended ObjectBinding/AssignmentPattern.
use crate::{
    BindingMode, DestructuringPropertyKeyIr, DestructuringTargetIr, ExprIr, OwnedEnvBindingIr,
    StatementIr, TypedExpr, ValueInfo, ValueKind,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObjectDestructuringPreparationError {
    MissingOwnedBinding,
    AliasedOwnedBinding,
    UncapturedTarget,
}

/// The actual raw receiver and ToObject result are distinct retained cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDestructuringSourceIr {
    raw: TypedExpr,
    boxed: TypedExpr,
    raw_binding: OwnedEnvBindingIr,
    boxed_binding: OwnedEnvBindingIr,
}

impl ObjectDestructuringSourceIr {
    pub(crate) fn prepare(
        raw: TypedExpr,
        raw_binding: OwnedEnvBindingIr,
        boxed_binding: OwnedEnvBindingIr,
        owned_bindings: &[OwnedEnvBindingIr],
    ) -> Result<(Vec<StatementIr>, Self), ObjectDestructuringPreparationError> {
        validate_owned_binding(&raw_binding, owned_bindings)?;
        validate_owned_binding(&boxed_binding, owned_bindings)?;
        require_distinct(&raw_binding, &boxed_binding)?;
        let mut info = raw.value_info();
        info.heap_shape = None;
        let raw_read = TypedExpr::from_info(info, ExprIr::Identifier(raw_binding.name.clone()));
        let boxed_init = TypedExpr::spec_to_object(raw_read.clone());
        let boxed_read = TypedExpr::from_info(
            boxed_init.value_info(),
            ExprIr::Identifier(boxed_binding.name.clone()),
        );
        let statements = vec![
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: raw_binding.name.clone(),
                init: raw,
            },
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: boxed_binding.name.clone(),
                init: boxed_init,
            },
        ];
        Ok((
            statements,
            Self {
                raw: raw_read,
                boxed: boxed_read,
                raw_binding,
                boxed_binding,
            },
        ))
    }

    pub fn raw_receiver(&self) -> &TypedExpr {
        &self.raw
    }
}

/// A read whose sole producer performs the actual ToPropertyKey operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDestructuringKeyIr {
    normalized: TypedExpr,
    binding: OwnedEnvBindingIr,
}

impl ObjectDestructuringKeyIr {
    pub(crate) fn prepare(
        raw_key: TypedExpr,
        binding: OwnedEnvBindingIr,
        owned_bindings: &[OwnedEnvBindingIr],
    ) -> Result<(StatementIr, Self), ObjectDestructuringPreparationError> {
        validate_owned_binding(&binding, owned_bindings)?;
        let normalized = TypedExpr::spec_to_property_key(raw_key);
        let read = TypedExpr::from_info(
            normalized.value_info(),
            ExprIr::Identifier(binding.name.clone()),
        );
        Ok((
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: binding.name.clone(),
                init: normalized,
            },
            Self {
                normalized: read,
                binding,
            },
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObjectDestructuringOperation {
    GetV {
        raw_receiver: TypedExpr,
        boxed: TypedExpr,
        key: TypedExpr,
    },
    Rest {
        boxed: TypedExpr,
        excluded: Vec<TypedExpr>,
    },
    PutTarget {
        target: DestructuringTargetIr,
        value: TypedExpr,
    },
}

/// Read-only views retain the provenance of the private preparation owners.
pub enum ObjectDestructuringOperationView<'a> {
    GetV {
        raw_receiver: &'a TypedExpr,
        boxed: &'a TypedExpr,
        key: &'a TypedExpr,
    },
    Rest {
        boxed: &'a TypedExpr,
        excluded: &'a [TypedExpr],
    },
    PutTarget {
        target: &'a DestructuringTargetIr,
        value: &'a TypedExpr,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDestructuringOperationIr(ObjectDestructuringOperation);

impl ObjectDestructuringOperationIr {
    pub(crate) fn get_v(
        source: &ObjectDestructuringSourceIr,
        key: &ObjectDestructuringKeyIr,
    ) -> Result<Self, ObjectDestructuringPreparationError> {
        require_distinct(&source.raw_binding, &key.binding)?;
        require_distinct(&source.boxed_binding, &key.binding)?;
        Ok(Self(ObjectDestructuringOperation::GetV {
            raw_receiver: source.raw.clone(),
            boxed: source.boxed.clone(),
            key: key.normalized.clone(),
        }))
    }

    pub(crate) fn rest(
        source: &ObjectDestructuringSourceIr,
        keys: &[ObjectDestructuringKeyIr],
    ) -> Result<Self, ObjectDestructuringPreparationError> {
        for (index, key) in keys.iter().enumerate() {
            require_distinct(&source.raw_binding, &key.binding)?;
            require_distinct(&source.boxed_binding, &key.binding)?;
            for previous in &keys[..index] {
                require_distinct(&previous.binding, &key.binding)?;
            }
        }
        Ok(Self(ObjectDestructuringOperation::Rest {
            boxed: source.boxed.clone(),
            excluded: keys.iter().map(|key| key.normalized.clone()).collect(),
        }))
    }

    pub(crate) fn put_target(
        target: DestructuringTargetIr,
        value: TypedExpr,
        owned_bindings: &[OwnedEnvBindingIr],
    ) -> Result<Self, ObjectDestructuringPreparationError> {
        match &target {
            DestructuringTargetIr::Binding {
                mode: BindingMode::Let | BindingMode::Const,
                ..
            }
            | DestructuringTargetIr::NestedArray(_) => {}
            DestructuringTargetIr::AssignmentProperty { target, key, .. } => {
                let base = captured_binding(target, owned_bindings)?;
                if let DestructuringPropertyKeyIr::Computed(key) = key {
                    let key = captured_binding(key, owned_bindings)?;
                    require_distinct(base, key)?;
                }
            }
            DestructuringTargetIr::AssignmentPrivate { target, .. } => {
                captured_binding(target, owned_bindings)?;
            }
            DestructuringTargetIr::Binding {
                mode: BindingMode::Var,
                ..
            }
            | DestructuringTargetIr::ResolvedVarBinding { .. }
            | DestructuringTargetIr::AssignmentIdentifier(_)
            | DestructuringTargetIr::AssignmentSuper { .. }
            | DestructuringTargetIr::NestedObject(_) => {
                return Err(ObjectDestructuringPreparationError::UncapturedTarget);
            }
        }
        Ok(Self(ObjectDestructuringOperation::PutTarget {
            target,
            value,
        }))
    }

    pub fn use_view(&self) -> ObjectDestructuringOperationView<'_> {
        match &self.0 {
            ObjectDestructuringOperation::GetV {
                raw_receiver,
                boxed,
                key,
            } => ObjectDestructuringOperationView::GetV {
                raw_receiver,
                boxed,
                key,
            },
            ObjectDestructuringOperation::Rest { boxed, excluded } => {
                ObjectDestructuringOperationView::Rest { boxed, excluded }
            }
            ObjectDestructuringOperation::PutTarget { target, value } => {
                ObjectDestructuringOperationView::PutTarget { target, value }
            }
        }
    }

    pub fn visit_expressions(&self, visit: &mut impl FnMut(&TypedExpr)) {
        match self.use_view() {
            ObjectDestructuringOperationView::GetV {
                raw_receiver,
                boxed,
                key,
            } => {
                visit(raw_receiver);
                visit(boxed);
                visit(key);
            }
            ObjectDestructuringOperationView::Rest { boxed, excluded } => {
                visit(boxed);
                for key in excluded {
                    visit(key);
                }
            }
            ObjectDestructuringOperationView::PutTarget { target, value } => {
                visit(value);
                crate::ir::visit_destructuring_target_expressions(target, visit);
            }
        }
    }

    pub fn visit_bindings(&self, visit: &mut impl FnMut(BindingMode, &str)) {
        if let ObjectDestructuringOperationView::PutTarget { target, .. } = self.use_view() {
            crate::ir::visit_destructuring_target_bindings(target, visit);
        }
    }

    pub(crate) fn into_expr(self) -> TypedExpr {
        let mut info = match self.use_view() {
            ObjectDestructuringOperationView::GetV { .. } => ValueInfo::new(ValueKind::Dynamic),
            ObjectDestructuringOperationView::Rest { .. } => ValueInfo::new(ValueKind::Object),
            ObjectDestructuringOperationView::PutTarget { value, .. } => value.value_info(),
        };
        info.heap_shape = None;
        TypedExpr::from_info(info, ExprIr::ObjectDestructuringOperation(Box::new(self)))
    }
}

fn validate_owned_binding(
    binding: &OwnedEnvBindingIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<(), ObjectDestructuringPreparationError> {
    if binding.name.is_empty() || !inventory.contains(binding) {
        return Err(ObjectDestructuringPreparationError::MissingOwnedBinding);
    }
    if inventory
        .iter()
        .filter(|row| row.name == binding.name || row.slot == binding.slot)
        .count()
        != 1
    {
        return Err(ObjectDestructuringPreparationError::AliasedOwnedBinding);
    }
    Ok(())
}

fn require_distinct(
    left: &OwnedEnvBindingIr,
    right: &OwnedEnvBindingIr,
) -> Result<(), ObjectDestructuringPreparationError> {
    if left.name == right.name || left.slot == right.slot {
        return Err(ObjectDestructuringPreparationError::AliasedOwnedBinding);
    }
    Ok(())
}

fn captured_binding<'a>(
    value: &TypedExpr,
    inventory: &'a [OwnedEnvBindingIr],
) -> Result<&'a OwnedEnvBindingIr, ObjectDestructuringPreparationError> {
    let ExprIr::Identifier(name) = &value.expr else {
        return Err(ObjectDestructuringPreparationError::UncapturedTarget);
    };
    let binding = inventory
        .iter()
        .find(|row| &row.name == name)
        .ok_or(ObjectDestructuringPreparationError::MissingOwnedBinding)?;
    validate_owned_binding(binding, inventory)?;
    Ok(binding)
}
