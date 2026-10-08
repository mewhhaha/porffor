//! The original With head and analyzed child record share one checked owner.

use crate::{
    BindingMode, BlockIr, EvalEnvironmentRoleIr, ExprIr, LexicalEnvironmentInitializationIr,
    LexicalEnvironmentIr, OwnedEnvBindingIr, SpecOperationIr, StatementIr, TypedExpr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WithObjectEnvironmentError {
    ForeignHead,
    UnallocatedHead,
    ForeignObjectEnvironment,
}

#[must_use = "the checked With head and object record must reach their carrier"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedWithObjectEnvironmentIr {
    head_binding: OwnedEnvBindingIr,
    object_binding: OwnedEnvBindingIr,
    lexical_environment: LexicalEnvironmentIr,
}

impl CheckedWithObjectEnvironmentIr {
    pub(crate) fn new(
        head: &BlockIr,
        head_value: &TypedExpr,
        head_binding: OwnedEnvBindingIr,
        object_binding: OwnedEnvBindingIr,
        lexical_environment: LexicalEnvironmentIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, WithObjectEnvironmentError> {
        let ExprIr::Identifier(name) = &head_value.expr else {
            return Err(WithObjectEnvironmentError::ForeignHead);
        };
        let mut matches = inventory
            .iter()
            .filter(|row| row.name == head_binding.name || row.slot == head_binding.slot);
        if &head_binding.name != name
            || matches.next() != Some(&head_binding)
            || matches.next().is_some()
        {
            return Err(WithObjectEnvironmentError::UnallocatedHead);
        }
        // The retained read must name the actual completed ToObject publication.
        let Some(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: published,
            init,
        }) = head.statements.last()
        else {
            return Err(WithObjectEnvironmentError::ForeignHead);
        };
        if published != name || init.value_info() != head_value.value_info() {
            return Err(WithObjectEnvironmentError::ForeignHead);
        }
        let ExprIr::SpecOperation {
            operation: SpecOperationIr::ToObject,
            operands,
        } = &init.expr
        else {
            return Err(WithObjectEnvironmentError::ForeignHead);
        };
        if operands.len() != 1 {
            return Err(WithObjectEnvironmentError::ForeignHead);
        }
        if init.value_info() != TypedExpr::spec_to_object(operands[0].clone()).value_info() {
            return Err(WithObjectEnvironmentError::ForeignHead);
        }
        // Invocation and child slots are distinct physical domains. The actual
        // analyzed With child contains exactly its original hidden object row.
        if object_binding.name == head_binding.name
            || object_binding.slot != 0
            || lexical_environment.bindings.as_slice() != std::slice::from_ref(&object_binding)
            || lexical_environment.initialization
                != LexicalEnvironmentInitializationIr::Uninitialized
            || lexical_environment.eval_environment
                != Some(EvalEnvironmentRoleIr::WithObject {
                    object_slot: object_binding.slot,
                })
        {
            return Err(WithObjectEnvironmentError::ForeignObjectEnvironment);
        }
        Ok(Self {
            head_binding,
            object_binding,
            lexical_environment,
        })
    }

    pub(crate) fn head_binding(&self) -> &OwnedEnvBindingIr {
        &self.head_binding
    }

    pub(crate) fn object_binding(&self) -> &OwnedEnvBindingIr {
        &self.object_binding
    }

    pub(crate) fn lexical_environment(&self) -> &LexicalEnvironmentIr {
        &self.lexical_environment
    }
}
