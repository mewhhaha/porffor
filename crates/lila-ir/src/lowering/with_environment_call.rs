//! Direct identifier calls whose Reference can select a `with` Object
//! Environment Record.
//!
//! This stays separate from the large general call lowerer because its sole
//! concern is preserving ResolveBinding's base through GetValue and Call. See
//! `docs/rust-rewrite/contracts/with-environment-identifier-call-reference.md`.

use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Intercept a direct identifier call before any name-specific builtin
    /// fold. `None` means the ordinary call lowerer retains ownership.
    pub(super) fn lower_with_environment_identifier_call(
        &mut self,
        callee: &Expression,
        source_args: &[Expression],
    ) -> Option<TypedExpr> {
        let Expression::Identifier(identifier) = callee else {
            return None;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();

        // Direct eval has a separate dynamic-source capability and Call
        // classification. This bounded Reference seam must not turn it into an
        // ordinary indirect call merely because an outer with exists.
        if name == "eval" {
            return None;
        }

        // Locate the declarative/global fallback before any observable
        // HasBinding operation. The selected chain is structurally non-empty.
        let fallback_reference = self.locate_identifier_reference(&name);
        let objects = self
            .with_environment_chain
            .select_preceding(fallback_reference.declarative_position())?;
        let strictness = self.reference_strictness();
        let plan = objects.into_identifier_call_plan(name.clone(), strictness, || {
            self.alloc_temp_binding_name("with.unscopables.")
        });

        // HasBinding and @@unscopables can invoke arbitrary user code before
        // the fallback GetValue. Construct the fallback from the prelocated
        // storage/global Reference, but deliberately discard every pre-With
        // value/target fact and widen mutable compiler metadata before
        // lowering arguments, which execute after callee GetValue.
        self.invalidate_unknown_user_code_effects();
        let fallback_callee = self.with_identifier_call_fallback(&name, fallback_reference);
        let args = self
            .lower_call_args_expanding_spread(source_args)
            .into_arguments_without_predecessor();
        let fallback = TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::CallIndirect {
                callee: Box::new(fallback_callee),
                this_arg: None,
                args: args.clone(),
                static_regexp_compilation: None,
            },
        );
        let result = plan.call(args, fallback);
        self.observe_unaccounted_invocation_effects(InvocationTargetProvenance::Erased);
        Some(result)
    }

    /// Consume the already located fallback into a fresh run-time callee read.
    /// No compiler lookup occurs after the observable with selection.
    fn with_identifier_call_fallback(
        &mut self,
        name: &str,
        reference: LocatedIdentifierReference,
    ) -> TypedExpr {
        let dynamic = unknown_runtime_value_info();

        // Script `var`/function bindings are owned by the Global Object Record
        // even when the analysis map also has a local mirror. Their runtime
        // presence and value can change during HasBinding.
        if self.is_script_global_var_name(name) && !self.has_scope_binding(name) {
            self.widen_with_identifier_call_global_fallback(name, dynamic.clone());
            return TypedExpr::from_info(
                dynamic,
                ExprIr::GlobalIdentifierRead {
                    name: name.to_string(),
                },
            );
        }

        match reference {
            LocatedIdentifierReference::Declarative { resolution, .. } => match resolution {
                BindingResolution::Uninitialized(violation) => violation.into_throw(),
                BindingResolution::Initialized(binding) => {
                    if binding.mode != BindingMode::Const {
                        self.widen_binding_for_possible_replacement(name);
                    }
                    TypedExpr::from_info(dynamic, ExprIr::Identifier(binding.storage_name))
                }
                BindingResolution::Unresolvable => {
                    unreachable!("a declarative location cannot be unresolvable")
                }
            },
            LocatedIdentifierReference::Unresolvable => {
                self.widen_with_identifier_call_global_fallback(name, dynamic.clone());
                TypedExpr::from_info(
                    dynamic,
                    ExprIr::GlobalIdentifierRead {
                        name: name.to_string(),
                    },
                )
            }
        }
    }

    fn widen_with_identifier_call_global_fallback(&mut self, name: &str, dynamic: ValueInfo) {
        if let Some(binding) = self.var_bindings.get_mut(name) {
            binding.kind = dynamic.kind;
            binding.possible_kinds = dynamic.possible_kinds;
            binding.heap_shape = None;
            binding.function_targets.widen_for_possible_replacement();
        }
        if let Some(property) = self.global_properties.get_mut(name) {
            property.value_info.widen_for_possible_replacement();
            if property.configurable {
                property.proven_present = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn with_environment_identifier_call_contract_is_owned_by_closed_plan() {
        let reference = include_str!("../reference.rs");
        let lowering = include_str!("with_environment_call.rs");

        assert!(reference.contains("struct WithEnvironmentIdentifierCallReferencePlan"));
        assert!(reference
            .contains("a with-environment identifier-call Reference must be consumed by Call"));
        assert!(reference.contains("pub(crate) fn into_identifier_call_plan("));
        assert!(reference
            .contains("pub(crate) fn call(self, args: Vec<TypedExpr>, fallback: TypedExpr)"));
        assert!(!reference.contains("impl Clone for WithEnvironmentIdentifierCallReferencePlan"));
        assert!(!reference.contains("impl Copy for WithEnvironmentIdentifierCallReferencePlan"));

        let intercept = lowering
            .find("pub(super) fn lower_with_environment_identifier_call(")
            .expect("identifier-call interception helper");
        let locate = lowering[intercept..]
            .find("let fallback_reference = self.locate_identifier_reference(&name);")
            .expect("fallback Reference must be located");
        let args = lowering[intercept..]
            .find("self.lower_call_args_expanding_spread(source_args)")
            .expect("arguments must be lowered once");
        assert!(locate < args, "callee Reference must precede arguments");
        assert!(lowering.contains("ExprIr::GlobalIdentifierRead"));
        assert!(lowering.contains("widen_for_possible_replacement()"));
        assert!(lowering.contains("this_arg: None"));
    }
}
