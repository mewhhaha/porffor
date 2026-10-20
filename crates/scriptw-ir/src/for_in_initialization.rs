//! Original ForIn head Environment Record validation for the complete protocols.

use crate::generator_loop_control::visit_lexical_head_statement_bindings;
use crate::lowering_helpers::{GeneratorForInHeadKind, GeneratorForInHeadProof};
use crate::{
    BindingMode, BlockIr, ExprIr, ForInOfEnvironmentIr, LexicalEnvironmentInitializationIr,
    LexicalEnvironmentIr, StatementIr,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ForInInitializationError {
    ForeignSourceHead,
    InvalidInitialization,
    ForeignLexicalEnvironment,
}

pub(crate) fn validate_original_head_environment(
    proof: &GeneratorForInHeadProof,
    initialization: &BlockIr,
    environment: Option<&ForInOfEnvironmentIr>,
    invocation_names: &BTreeSet<&str>,
) -> Result<(), ForInInitializationError> {
    let lexical = matches!(proof.mode(), BindingMode::Let | BindingMode::Const);
    let binding_head = matches!(
        proof.kind(),
        GeneratorForInHeadKind::Binding | GeneratorForInHeadKind::BindingPattern
    );
    if !lexical {
        if environment.is_some() || !proof.lexical_storage_names().is_empty() {
            return Err(ForInInitializationError::ForeignLexicalEnvironment);
        }
        if binding_head {
            let expected: BTreeSet<_> = proof.bound_names().iter().cloned().collect();
            let mut initialized = BTreeSet::new();
            visit_lexical_head_statement_bindings(&initialization.statements, &mut |mode, name| {
                if mode == BindingMode::Var {
                    initialized.insert(name.to_string());
                }
            });
            // Direct eval writes the original variable environment rather
            // than fabricating a new local declaration in this invocation.
            for statement in &initialization.statements {
                if let StatementIr::DeclarationEvaluation(value) = statement {
                    if let ExprIr::EnvironmentIdentifier(identifier) = &value.expr {
                        if matches!(
                            identifier.operation,
                            crate::EnvironmentIdentifierOperationIr::Assign { .. }
                                | crate::EnvironmentIdentifierOperationIr::AssignWithGlobalFallback { .. }
                        ) {
                            initialized.insert(identifier.name.clone());
                        }
                    }
                }
            }
            if initialized != expected {
                return Err(ForInInitializationError::InvalidInitialization);
            }
        }
        return Ok(());
    }
    if !binding_head {
        return Err(ForInInitializationError::ForeignSourceHead);
    }
    let Some(environment) = environment else {
        return Err(ForInInitializationError::ForeignLexicalEnvironment);
    };
    let expected_tdz: BTreeSet<_> = proof
        .bound_names()
        .iter()
        .map(|name| {
            crate::binding_lifecycle::TdzPlaceholderName::for_source_name(name).into_string()
        })
        .collect();
    let actual_tdz: BTreeSet<_> = environment.tdz_binding_names.iter().cloned().collect();
    let storage: BTreeSet<_> = proof.lexical_storage_names().values().cloned().collect();
    let mut initialized = BTreeSet::new();
    visit_lexical_head_statement_bindings(&initialization.statements, &mut |mode, name| {
        if mode == proof.mode() {
            initialized.insert(name.to_string());
        }
    });
    if actual_tdz.len() != environment.tdz_binding_names.len()
        || actual_tdz != expected_tdz
        || !storage.is_subset(&initialized)
        || storage
            .iter()
            .any(|name| invocation_names.contains(name.as_str()))
    {
        return Err(ForInInitializationError::ForeignLexicalEnvironment);
    }
    for (record, expected) in [
        (environment.tdz_environment.as_ref(), &expected_tdz),
        (environment.iteration_environment.as_ref(), &storage),
    ] {
        if let Some(record) = record {
            if !valid_original_record(record, expected, invocation_names) {
                return Err(ForInInitializationError::ForeignLexicalEnvironment);
            }
        }
    }
    Ok(())
}

fn valid_original_record(
    record: &LexicalEnvironmentIr,
    expected: &BTreeSet<String>,
    invocation_names: &BTreeSet<&str>,
) -> bool {
    let names: BTreeSet<_> = record
        .bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect();
    let slots: BTreeSet<_> = record.bindings.iter().map(|binding| binding.slot).collect();
    !matches!(
        record.eval_environment,
        Some(crate::EvalEnvironmentRoleIr::WithObject { .. })
    ) && record.initialization == LexicalEnvironmentInitializationIr::Uninitialized
        && names.len() == record.bindings.len()
        && slots.len() == record.bindings.len()
        && names
            .iter()
            .all(|name| expected.contains(*name) && !invocation_names.contains(*name))
}
