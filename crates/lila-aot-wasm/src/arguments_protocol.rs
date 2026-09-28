use lila_ir::{FunctionFlavor, FunctionIr, StatementIr};

use crate::functions::LegacyActivationMode;
use crate::{EmitError, MappedSlot};

/// Whether this invocation owns an arguments object, and the only legal
/// construction protocol when it does.
///
/// See `docs/rust-rewrite/contracts/arguments-object-construction-protocol.md`.
pub(crate) struct FunctionArgumentsProtocol(FunctionArgumentsState);

enum FunctionArgumentsState {
    Pending(FunctionArgumentsKind),
    BoundAbsent,
    BoundPresent,
}

enum FunctionArgumentsKind {
    Absent,
    Present(ArgumentsBindingAction),
}

pub(crate) enum ArgumentsBindingAction {
    Materialize(PresentArgumentsObjectProtocol),
    /// Binding is unobserved, but legacy reflection can request the object.
    DeferMapped(MappedArgumentsPlan),
    /// Source analysis proved that no binding read is reachable, and this
    /// function has no observable legacy `.arguments` data property.
    ElideUnobserved,
}

pub(crate) enum PresentArgumentsObjectProtocol {
    Unmapped(UnmappedArgumentsPlan),
    Mapped(MappedArgumentsPlan),
    /// Validated mapping words captured in the private native GC frame.
    MappedFromFrame(crate::abi::ArgVectorLocal),
}

#[must_use = "the function arguments binding protocol must be consumed"]
pub(crate) struct ArgumentsBindingProtocol(Option<ArgumentsBindingAction>);

pub(crate) struct UnmappedArgumentsPlan {
    _private: (),
}

pub(crate) struct MappedArgumentsPlan {
    entries: Box<[MappedArgumentEntry]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MappedArgumentEntry {
    argument: ArgumentIndex,
    environment: ParameterEnvironmentSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArgumentIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParameterEnvironmentSlot(u32);

impl FunctionArgumentsProtocol {
    pub(crate) const fn script_main() -> Self {
        Self(FunctionArgumentsState::Pending(
            FunctionArgumentsKind::Absent,
        ))
    }

    pub(crate) const fn strict_internal_callable() -> Self {
        Self(FunctionArgumentsState::Pending(
            FunctionArgumentsKind::Present(ArgumentsBindingAction::Materialize(
                PresentArgumentsObjectProtocol::Unmapped(UnmappedArgumentsPlan { _private: () }),
            )),
        ))
    }

    pub(crate) fn for_user_function(function: &FunctionIr) -> Result<Self, EmitError> {
        let protocol = match function.protocol.flavor() {
            FunctionFlavor::Arrow => {
                return Ok(Self(FunctionArgumentsState::Pending(
                    FunctionArgumentsKind::Absent,
                )));
            }
            FunctionFlavor::Ordinary if function.strict || !has_simple_parameter_list(function) => {
                PresentArgumentsObjectProtocol::Unmapped(UnmappedArgumentsPlan { _private: () })
            }
            FunctionFlavor::Ordinary => {
                PresentArgumentsObjectProtocol::Mapped(MappedArgumentsPlan::for_function(function)?)
            }
        };
        let action = if function.own_arguments_use.is_proven_unobserved() {
            match (
                LegacyActivationMode::for_source(function.protocol, function.strict).is_exposable(),
                protocol,
            ) {
                (false, _) => ArgumentsBindingAction::ElideUnobserved,
                (true, PresentArgumentsObjectProtocol::Mapped(plan)) => {
                    ArgumentsBindingAction::DeferMapped(plan)
                }
                (true, protocol) => ArgumentsBindingAction::Materialize(protocol),
            }
        } else {
            ArgumentsBindingAction::Materialize(protocol)
        };
        Ok(Self(FunctionArgumentsState::Pending(
            FunctionArgumentsKind::Present(action),
        )))
    }

    pub(crate) fn take_for_binding(&mut self) -> Result<ArgumentsBindingProtocol, EmitError> {
        match core::mem::replace(&mut self.0, FunctionArgumentsState::BoundAbsent) {
            FunctionArgumentsState::Pending(FunctionArgumentsKind::Absent) => {
                Ok(ArgumentsBindingProtocol(None))
            }
            FunctionArgumentsState::Pending(FunctionArgumentsKind::Present(protocol)) => {
                self.0 = FunctionArgumentsState::BoundPresent;
                Ok(ArgumentsBindingProtocol(Some(protocol)))
            }
            FunctionArgumentsState::BoundAbsent => Err(EmitError::unsupported(
                "compiler invariant violated: function arguments protocol was bound more than once",
            )),
            FunctionArgumentsState::BoundPresent => {
                self.0 = FunctionArgumentsState::BoundPresent;
                Err(EmitError::unsupported(
                    "compiler invariant violated: function arguments protocol was bound more than once",
                ))
            }
        }
    }

    pub(crate) const fn present(&self) -> Option<()> {
        match &self.0 {
            FunctionArgumentsState::Pending(FunctionArgumentsKind::Absent)
            | FunctionArgumentsState::BoundAbsent => None,
            FunctionArgumentsState::Pending(FunctionArgumentsKind::Present(_))
            | FunctionArgumentsState::BoundPresent => Some(()),
        }
    }
}

impl ArgumentsBindingProtocol {
    pub(crate) fn into_present_action(self) -> Option<ArgumentsBindingAction> {
        self.0
    }
}

impl MappedArgumentsPlan {
    fn for_function(function: &FunctionIr) -> Result<Self, EmitError> {
        let mut entries = Vec::with_capacity(function.params.len());
        for (index, parameter) in function.params.iter().enumerate() {
            if function.params[index + 1..]
                .iter()
                .any(|later| later.name == parameter.name)
            {
                continue;
            }

            let argument = u32::try_from(index).map(ArgumentIndex).map_err(|_| {
                mapped_arguments_invariant_error(
                    function,
                    format!("argument index {index} does not fit the backend index domain"),
                )
            })?;
            let mut matching_bindings = function
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name == parameter.name);
            let binding = matching_bindings.next().ok_or_else(|| {
                mapped_arguments_invariant_error(
                    function,
                    format!(
                        "parameter `{}` has no owned environment slot",
                        parameter.name
                    ),
                )
            })?;
            if matching_bindings.next().is_some() {
                return Err(mapped_arguments_invariant_error(
                    function,
                    format!(
                        "parameter `{}` has more than one owned environment slot",
                        parameter.name
                    ),
                ));
            }
            if function
                .owned_env_bindings
                .iter()
                .filter(|candidate| candidate.slot == binding.slot)
                .count()
                != 1
            {
                return Err(mapped_arguments_invariant_error(
                    function,
                    format!(
                        "parameter `{}` shares environment slot {} with another binding",
                        parameter.name, binding.slot
                    ),
                ));
            }

            entries.push(MappedArgumentEntry {
                argument,
                environment: ParameterEnvironmentSlot(binding.slot),
            });
        }
        Ok(Self {
            entries: entries.into_boxed_slice(),
        })
    }

    pub(crate) fn entries(&self) -> &[MappedArgumentEntry] {
        &self.entries
    }
}

impl MappedArgumentEntry {
    pub(crate) const fn argument_index_i64(self) -> i64 {
        self.argument.0 as i64
    }

    pub(crate) const fn mapped_slot(self) -> MappedSlot {
        MappedSlot::new(self.environment.0)
    }
}

fn has_simple_parameter_list(function: &FunctionIr) -> bool {
    function
        .params
        .iter()
        .all(|parameter| !parameter.is_rest && parameter.default_init.is_none())
        && !function
            .body
            .statements
            .iter()
            .any(|statement| matches!(statement, StatementIr::ParameterInitialization { .. }))
}

fn mapped_arguments_invariant_error(function: &FunctionIr, detail: String) -> EmitError {
    EmitError::unsupported(format!(
        "compiler invariant violated while planning mapped arguments for `{}`: {detail}",
        function.name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};
    use lila_ir::{lower, ProgramIr};

    fn lower_script(source: &str) -> ProgramIr {
        let parsed = parse(source, ParseOptions::script()).expect("script should parse");
        lower(&parsed)
    }

    fn only_function(program: &ProgramIr) -> &FunctionIr {
        let functions = &program
            .script
            .as_ref()
            .expect("script IR should exist")
            .functions;
        assert_eq!(functions.len(), 1, "fixture should lower one function");
        &functions[0]
    }

    fn protocol_for(source: &str) -> FunctionArgumentsProtocol {
        let program = lower_script(source);
        FunctionArgumentsProtocol::for_user_function(only_function(&program))
            .expect("valid lowered function should have an arguments protocol")
    }

    fn take_action(protocol: &mut FunctionArgumentsProtocol) -> Option<ArgumentsBindingAction> {
        protocol
            .take_for_binding()
            .expect("fresh protocol should be bindable")
            .into_present_action()
    }

    fn take_materialized(
        protocol: &mut FunctionArgumentsProtocol,
    ) -> Option<PresentArgumentsObjectProtocol> {
        match take_action(protocol) {
            Some(ArgumentsBindingAction::Materialize(plan)) => Some(plan),
            Some(ArgumentsBindingAction::DeferMapped(plan)) => {
                Some(PresentArgumentsObjectProtocol::Mapped(plan))
            }
            Some(ArgumentsBindingAction::ElideUnobserved) => {
                panic!("fixture requires materialized arguments")
            }
            None => None,
        }
    }

    fn mapped_entries(protocol: &mut FunctionArgumentsProtocol) -> Vec<(u32, u32)> {
        let Some(PresentArgumentsObjectProtocol::Mapped(plan)) = take_materialized(protocol) else {
            panic!("expected a mapped arguments protocol");
        };
        plan.entries
            .iter()
            .map(|entry| (entry.argument.0, entry.environment.0))
            .collect()
    }

    #[test]
    fn function_arguments_protocol_distinguishes_absent_unmapped_and_mapped_empty() {
        assert!(take_action(&mut protocol_for("const f = (value) => value;")).is_none());

        for source in [
            "function f(value) { 'use strict'; return arguments[0]; }",
            "function f(value = 1) {}",
            "function f(...values) {}",
            "function f({ value }) {}",
        ] {
            assert!(matches!(
                take_materialized(&mut protocol_for(source)),
                Some(PresentArgumentsObjectProtocol::Unmapped(_))
            ));
        }

        let mut empty = protocol_for("function f() {}");
        assert!(matches!(
            take_materialized(&mut empty),
            Some(PresentArgumentsObjectProtocol::Mapped(_))
        ));
        let mut empty = protocol_for("function f() {}");
        assert!(mapped_entries(&mut empty).is_empty());
    }

    #[test]
    fn function_arguments_mapped_plan_uses_validated_slots_and_last_duplicate() {
        assert_eq!(
            mapped_entries(&mut protocol_for("function f(first, second) {}")),
            vec![(0, 0), (1, 1)]
        );
        assert_eq!(
            mapped_entries(&mut protocol_for("function f(value, other, value) {}")),
            vec![(1, 1), (2, 0)]
        );
    }

    #[test]
    fn function_arguments_protocol_can_bind_only_once() {
        let mut protocol = protocol_for("function f(value) {}");
        assert!(take_action(&mut protocol).is_some());

        let Err(error) = protocol.take_for_binding() else {
            panic!("a consumed arguments protocol must reject a second binding");
        };
        assert!(error.to_string().contains("was bound more than once"));
    }

    #[test]
    fn unused_non_reflectable_binding_elides_only_its_object() {
        for source in [
            "function f(value) { 'use strict'; return value; }",
            "const holder = { method(value) { return value; } };",
            "const holder = { f(value) { var arguments = 1; return value; } };",
        ] {
            let mut protocol = protocol_for(source);
            assert!(protocol.present().is_some(), "{source}");
            assert!(
                matches!(
                    take_action(&mut protocol),
                    Some(ArgumentsBindingAction::ElideUnobserved)
                ),
                "{source}"
            );
            assert!(protocol.present().is_some(), "{source}");
        }
    }

    #[test]
    fn unobserved_sloppy_binding_defers_its_mapped_object() {
        assert!(matches!(
            take_action(&mut protocol_for("function f(value) { return value; }")),
            Some(ArgumentsBindingAction::DeferMapped(_))
        ));
    }

    #[test]
    fn source_reads_keep_arguments_materialized() {
        for source in [
            "function f(value) { 'use strict'; return arguments[0]; }",
            "const holder = { f(value) { arguments = 1; return value; } };",
            "const holder = { f(value) { var arguments; return arguments[0]; } };",
            "function f(value) { 'use strict'; return () => arguments[0]; }",
            "function f(value) { 'use strict'; return eval('arguments')[0]; }",
            "'use strict'; function f(value = arguments[0]) { return value; }",
        ] {
            let program = lower_script(source);
            let function = program
                .script
                .as_ref()
                .expect("script IR should exist")
                .functions
                .iter()
                .find(|function| function.name == "f")
                .expect("fixture should lower f");
            let mut protocol = FunctionArgumentsProtocol::for_user_function(function)
                .expect("valid function should have an arguments protocol");
            assert!(
                matches!(
                    take_action(&mut protocol),
                    Some(ArgumentsBindingAction::Materialize(_))
                ),
                "{source}"
            );
        }
    }

    #[test]
    fn function_emission_rejects_missing_mapped_parameter_storage() {
        let mut program = lower_script("function f(value) {}");
        let function = program
            .script
            .as_mut()
            .expect("script IR should exist")
            .functions
            .first_mut()
            .expect("fixture should lower one function");
        function
            .owned_env_bindings
            .retain(|binding| binding.name != "value");

        let error = crate::emit(&program).expect_err("malformed mapped storage must be rejected");
        assert!(
            error
                .to_string()
                .contains("parameter `value` has no owned environment slot"),
            "unexpected error: {error}"
        );
    }
}
