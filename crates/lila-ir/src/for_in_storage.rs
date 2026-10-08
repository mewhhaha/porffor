//! One consumed original storage and source-bound per-key initialization proof.

use crate::lowering::CheckedAsyncGeneratorForInInitializer;
use crate::lowering_helpers::GeneratorForInHeadProof;
use crate::{
    BindingMode, BlockIr, ExprIr, ForInOfEnvironmentIr, OwnedEnvBindingIr, StatementIr, TypedExpr,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ForInStorageError {
    InvalidHeadEnvironment,
    UnallocatedBinding,
    AliasedBindings,
    MissingHeadPublication,
    ForeignSourceHead,
    InvalidInitialization,
    ForeignLexicalEnvironment,
}

#[must_use = "the checked storage and initializer must reach the actual ForIn carrier"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedForInStorageIr {
    head_binding: OwnedEnvBindingIr,
    initialization: CheckedAsyncGeneratorForInInitializer,
    lexical_environment: Option<ForInOfEnvironmentIr>,
    enumerator_binding: OwnedEnvBindingIr,
    key_binding: OwnedEnvBindingIr,
    value_binding: OwnedEnvBindingIr,
}

impl CheckedForInStorageIr {
    pub(crate) fn new(
        head_proof: &GeneratorForInHeadProof,
        head: &BlockIr,
        head_value: &TypedExpr,
        head_binding: OwnedEnvBindingIr,
        initialization: CheckedAsyncGeneratorForInInitializer,
        lexical_environment: Option<ForInOfEnvironmentIr>,
        enumerator_binding: OwnedEnvBindingIr,
        key_binding: OwnedEnvBindingIr,
        value_binding: OwnedEnvBindingIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, ForInStorageError> {
        if head.lexical_environment.is_some() {
            return Err(ForInStorageError::InvalidHeadEnvironment);
        }
        let bindings = [
            &head_binding,
            &enumerator_binding,
            &key_binding,
            &value_binding,
        ];
        let names: BTreeSet<_> = bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect();
        let slots: BTreeSet<_> = bindings.iter().map(|binding| binding.slot).collect();
        if names.len() != bindings.len() || slots.len() != bindings.len() {
            return Err(ForInStorageError::AliasedBindings);
        }
        for binding in bindings {
            let mut rows = inventory
                .iter()
                .filter(|row| row.name == binding.name || row.slot == binding.slot);
            if rows.next() != Some(binding) || rows.next().is_some() {
                return Err(ForInStorageError::UnallocatedBinding);
            }
        }
        let ExprIr::Identifier(read) = &head_value.expr else {
            return Err(ForInStorageError::MissingHeadPublication);
        };
        let Some(StatementIr::Lexical {
            mode: BindingMode::Let,
            name,
            init,
        }) = head.statements.last()
        else {
            return Err(ForInStorageError::MissingHeadPublication);
        };
        if read != &head_binding.name
            || name != read
            || init.value_info() != head_value.value_info()
            || head
                .statements
                .iter()
                .filter(|statement| {
                    matches!(statement,
                StatementIr::Lexical { name, .. } if name == &head_binding.name)
                })
                .count()
                != 1
        {
            return Err(ForInStorageError::MissingHeadPublication);
        }
        if initialization.key_binding() != &key_binding
            || initialization.mode() != head_proof.mode()
        {
            return Err(ForInStorageError::InvalidInitialization);
        }
        if initialization.environment() != lexical_environment.as_ref() {
            return Err(ForInStorageError::ForeignLexicalEnvironment);
        }
        crate::for_in_initialization::validate_original_head_environment(
            head_proof,
            initialization.region().block(),
            lexical_environment.as_ref(),
            &names,
        )
        .map_err(initialization_error)?;
        Ok(Self {
            head_binding,
            initialization,
            lexical_environment,
            enumerator_binding,
            key_binding,
            value_binding,
        })
    }

    pub(crate) fn head_binding(&self) -> &OwnedEnvBindingIr {
        &self.head_binding
    }
    pub(crate) fn initialization(&self) -> &CheckedAsyncGeneratorForInInitializer {
        &self.initialization
    }
    pub(crate) fn lexical_environment(&self) -> Option<&ForInOfEnvironmentIr> {
        self.lexical_environment.as_ref()
    }
    pub(crate) fn enumerator_binding(&self) -> &OwnedEnvBindingIr {
        &self.enumerator_binding
    }
    pub(crate) fn key_binding(&self) -> &OwnedEnvBindingIr {
        &self.key_binding
    }
    pub(crate) fn value_binding(&self) -> &OwnedEnvBindingIr {
        &self.value_binding
    }
}

fn initialization_error(
    error: crate::for_in_initialization::ForInInitializationError,
) -> ForInStorageError {
    use crate::for_in_initialization::ForInInitializationError as Error;
    match error {
        Error::ForeignSourceHead => ForInStorageError::ForeignSourceHead,
        Error::InvalidInitialization => ForInStorageError::InvalidInitialization,
        Error::ForeignLexicalEnvironment => ForInStorageError::ForeignLexicalEnvironment,
    }
}
