use lila_ir::{FunctionFlavor, FunctionIr, StatementIr};

use crate::{EmitError, MappedSlot};

/// Whether this invocation owns an arguments object, and the only legal
/// construction protocol when it does.
///
/// See `docs/rust-rewrite/contracts/arguments-object-construction-protocol.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionArgumentsProtocol(FunctionArgumentsKind);

#[derive(Debug, Clone, PartialEq, Eq)]
enum FunctionArgumentsKind {
    Absent,
    Present(PresentArgumentsObjectProtocol),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PresentArgumentsObjectProtocol {
    Unmapped(UnmappedArgumentsPlan),
    Mapped(MappedArgumentsPlan),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnmappedArgumentsPlan {
    _private: (),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        Self(FunctionArgumentsKind::Absent)
    }

    pub(crate) const fn strict_internal_callable() -> Self {
        Self(FunctionArgumentsKind::Present(
            PresentArgumentsObjectProtocol::Unmapped(UnmappedArgumentsPlan { _private: () }),
        ))
    }

    pub(crate) fn for_user_function(function: &FunctionIr) -> Result<Self, EmitError> {
        match function.protocol.flavor() {
            FunctionFlavor::Arrow => Ok(Self(FunctionArgumentsKind::Absent)),
            FunctionFlavor::Ordinary if function.strict || !has_simple_parameter_list(function) => {
                Ok(Self(FunctionArgumentsKind::Present(
                    PresentArgumentsObjectProtocol::Unmapped(UnmappedArgumentsPlan {
                        _private: (),
                    }),
                )))
            }
            FunctionFlavor::Ordinary => Ok(Self(FunctionArgumentsKind::Present(
                PresentArgumentsObjectProtocol::Mapped(MappedArgumentsPlan::for_function(
                    function,
                )?),
            ))),
        }
    }

    pub(crate) fn present(&self) -> Option<&PresentArgumentsObjectProtocol> {
        match &self.0 {
            FunctionArgumentsKind::Absent => None,
            FunctionArgumentsKind::Present(protocol) => Some(protocol),
        }
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

    fn mapped_entries(protocol: &FunctionArgumentsProtocol) -> Vec<(u32, u32)> {
        let Some(PresentArgumentsObjectProtocol::Mapped(plan)) = protocol.present() else {
            panic!("expected a mapped arguments protocol");
        };
        plan.entries
            .iter()
            .map(|entry| (entry.argument.0, entry.environment.0))
            .collect()
    }

    #[test]
    fn function_arguments_protocol_distinguishes_absent_unmapped_and_mapped_empty() {
        assert!(protocol_for("const f = (value) => value;")
            .present()
            .is_none());

        for source in [
            "function f(value) { 'use strict'; }",
            "function f(value = 1) {}",
            "function f(...values) {}",
            "function f({ value }) {}",
        ] {
            assert!(matches!(
                protocol_for(source).present(),
                Some(PresentArgumentsObjectProtocol::Unmapped(_))
            ));
        }

        let empty = protocol_for("function f() {}");
        assert!(matches!(
            empty.present(),
            Some(PresentArgumentsObjectProtocol::Mapped(_))
        ));
        assert!(mapped_entries(&empty).is_empty());
    }

    #[test]
    fn function_arguments_mapped_plan_uses_validated_slots_and_last_duplicate() {
        assert_eq!(
            mapped_entries(&protocol_for("function f(first, second) {}")),
            vec![(0, 0), (1, 1)]
        );
        assert_eq!(
            mapped_entries(&protocol_for("function f(value, other, value) {}")),
            vec![(1, 1), (2, 0)]
        );
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
