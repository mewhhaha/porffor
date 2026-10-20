//! Empty prepared eval still carries intrinsic lookup facts and its hidden caller capture.
use crate::{
    EnvironmentBindingMutabilityIr, EvalDeclarativeEnvironmentKindIr, EvalEnvironmentRoleIr,
    GlobalPropertyInitializerIr, PreparedScriptUnit, DIRECT_EVAL_EXECUTION_CONTEXT_NAME,
};

pub(super) fn is_inert(unit: &PreparedScriptUnit) -> bool {
    let capture = match unit.owned_env_bindings.as_slice() {
        [] => true,
        [binding] => {
            binding.name == DIRECT_EVAL_EXECUTION_CONTEXT_NAME
                && binding.slot == 0
                && binding.mutability == EnvironmentBindingMutabilityIr::Immutable { strict: true }
        }
        _ => false,
    };
    let environment = match &unit.eval_environment {
        None => true,
        Some(EvalEnvironmentRoleIr::Declarative { kind, bindings }) => {
            bindings.is_empty()
                && matches!(
                    (unit.strict, *kind),
                    (false, EvalDeclarativeEnvironmentKindIr::Lexical)
                        | (true, EvalDeclarativeEnvironmentKindIr::Variable)
                )
        }
        Some(EvalEnvironmentRoleIr::WithObject { .. }) => false,
    };
    capture
        && environment
        && unit.global_bindings.lexical_names().next().is_none()
        && unit.global_bindings.iter().all(|binding| {
            !binding.declarations.is_declared()
                && match &binding.initializer {
                    GlobalPropertyInitializerIr::FreshUndefined
                    | GlobalPropertyInitializerIr::SourceFunction(_) => false,
                    GlobalPropertyInitializerIr::Intrinsic
                    | GlobalPropertyInitializerIr::Infinity
                    | GlobalPropertyInitializerIr::NaN
                    | GlobalPropertyInitializerIr::Undefined
                    | GlobalPropertyInitializerIr::ReflectObject
                    | GlobalPropertyInitializerIr::MathObject
                    | GlobalPropertyInitializerIr::JsonObject
                    | GlobalPropertyInitializerIr::AtomicsObject
                    | GlobalPropertyInitializerIr::TemporalObject
                    | GlobalPropertyInitializerIr::IntlObject
                    | GlobalPropertyInitializerIr::BuiltinFunction(_)
                    | GlobalPropertyInitializerIr::HostFunction(_) => true,
                }
        })
}
