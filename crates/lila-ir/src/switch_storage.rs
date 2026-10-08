//! One checked retained-cell and CaseBlock storage owner for Switch carriers.

use crate::generator_switch::GeneratorSwitchControlError;
use crate::{
    BindingMode, BlockIr, ExprIr, LexicalEnvironmentInitializationIr, LexicalEnvironmentIr,
    OwnedEnvBindingIr, StatementIr, TypedExpr,
};

#[must_use = "the checked Switch storage must reach its actual carrier"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedSwitchStorageIr {
    discriminant_binding: OwnedEnvBindingIr,
    value_binding: OwnedEnvBindingIr,
    lexical_environment: Option<LexicalEnvironmentIr>,
}

impl CheckedSwitchStorageIr {
    pub(crate) fn new(
        head: &BlockIr,
        head_value: &TypedExpr,
        discriminant_binding: OwnedEnvBindingIr,
        value_binding: OwnedEnvBindingIr,
        lexical_environment: Option<LexicalEnvironmentIr>,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, GeneratorSwitchControlError> {
        if discriminant_binding.name == value_binding.name
            || discriminant_binding.slot == value_binding.slot
        {
            return Err(GeneratorSwitchControlError::AliasedBindings);
        }
        for binding in [&discriminant_binding, &value_binding] {
            // Name and slot both designate the same generated Environment
            // Record. An exact row plus a colliding row is also invalid.
            let mut matches = inventory
                .iter()
                .filter(|owned| owned.name == binding.name || owned.slot == binding.slot);
            if matches.next() != Some(binding) || matches.next().is_some() {
                return Err(GeneratorSwitchControlError::UnallocatedBinding);
            }
        }
        let discriminant_statements = &head.statements;
        if !matches!(
            &head_value.expr,
            ExprIr::Identifier(name) if name == &discriminant_binding.name
        ) || !matches!(
            discriminant_statements.last(),
            Some(StatementIr::Lexical { mode: BindingMode::Let, name, .. })
                if name == &discriminant_binding.name
        ) || discriminant_statements
            .iter()
            .filter(|statement| {
                matches!(
                    statement, StatementIr::Lexical { name, .. }
                        if name == &discriminant_binding.name
                )
            })
            .count()
            != 1
        {
            return Err(GeneratorSwitchControlError::MissingDiscriminantPublication);
        }
        if head.lexical_environment.is_some() {
            return Err(GeneratorSwitchControlError::InvalidCaseBlockEnvironment);
        }
        if let Some(environment) = &lexical_environment {
            let names: std::collections::BTreeSet<_> = environment
                .bindings
                .iter()
                .map(|binding| binding.name.as_str())
                .collect();
            let slots: std::collections::BTreeSet<_> = environment
                .bindings
                .iter()
                .map(|binding| binding.slot)
                .collect();
            if environment.initialization != LexicalEnvironmentInitializationIr::Uninitialized
                || names.len() != environment.bindings.len()
                || slots.len() != environment.bindings.len()
                || names.contains(discriminant_binding.name.as_str())
                || names.contains(value_binding.name.as_str())
            {
                return Err(GeneratorSwitchControlError::InvalidCaseBlockEnvironment);
            }
        }
        Ok(Self {
            discriminant_binding,
            value_binding,
            lexical_environment,
        })
    }

    pub(crate) fn discriminant_binding(&self) -> &OwnedEnvBindingIr {
        &self.discriminant_binding
    }

    pub(crate) fn value_binding(&self) -> &OwnedEnvBindingIr {
        &self.value_binding
    }

    pub(crate) fn lexical_environment(&self) -> Option<&LexicalEnvironmentIr> {
        self.lexical_environment.as_ref()
    }
}
