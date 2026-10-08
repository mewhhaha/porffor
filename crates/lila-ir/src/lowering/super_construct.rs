//! Retain real derived-constructor preparation before suspended argument evaluation.
use super::suspended_call::InvocationSuspension;
use super::*;
use crate::prepared_super_construct::SuperConstructCapturePlan;

impl ScriptLowerer<'_> {
    pub(super) fn owns_derived_super_call(&self) -> bool {
        self.class_context
            .as_ref()
            .is_some_and(|context| context.is_derived_constructor)
            || self.direct_eval_invocation()
                == Some(lila_front::EvalInvocationContext::DerivedConstructor)
    }

    pub(super) fn lower_super_call(&mut self, call: &SuperCall) -> TypedExpr {
        if !self.owns_derived_super_call() {
            return self.unsupported_expr("unsupported expression form: super call");
        }
        let args = self
            .lower_call_args_expanding_spread(call.arguments())
            .into_arguments_without_predecessor();
        let info = self.super_construct_result_info();
        TypedExpr::from_info(info, ExprIr::SuperConstruct { args })
    }

    pub(super) fn lower_suspended_super_call(&mut self, call: &SuperCall) -> Option<TypedExpr> {
        self.plain_async_entry_state()?;
        if !self.owns_derived_super_call() {
            return Some(self.unsupported_expr(
                "suspended super requires its lexical derived constructor owner",
            ));
        }
        let constructor =
            self.alloc_suspension_owned_binding("super.constructor.", unknown_runtime_value_info());
        let new_target =
            self.alloc_suspension_owned_binding("super.newTarget.", unknown_runtime_value_info());
        let binding = |name: &str| {
            self.generated_owned_env_bindings
                .iter()
                .find(|binding| binding.name == name)
                .cloned()
        };
        let (Some(constructor), Some(new_target)) = (binding(&constructor), binding(&new_target))
        else {
            return Some(self.unsupported_expr(
                "suspended super preparation requires allocated invocation cells",
            ));
        };
        let Some(plan) = SuperConstructCapturePlan::new(
            constructor,
            new_target,
            &self.generated_owned_env_bindings,
        ) else {
            return Some(
                self.unsupported_expr("suspended super preparation cannot alias invocation cells"),
            );
        };
        let (prefix, prepared) = plan.into_prefix();
        self.async_expression_prefix
            .as_mut()
            .expect("suspended SuperCall owns an argument prefix")
            .extend(prefix);
        // GetSuperConstructor can run a Proxy [[GetPrototypeOf]] trap before
        // ArgumentListEvaluation. Preserve original source-effect invalidation.
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        let Some(arguments) =
            self.capture_suspended_arguments(call.arguments(), InvocationSuspension::Await)
        else {
            return Some(
                self.unsupported_expr("suspended super requires its complete argument list owner"),
            );
        };
        self.super_construct_result_info();
        // Before BindThisValue an async arrow's lexical-this planning fact may
        // be Undefined. Construct nevertheless returns its actual object.
        Some(TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::PreparedSuperConstruct(Box::new(prepared.finish(arguments))),
        ))
    }

    fn super_construct_result_info(&mut self) -> ValueInfo {
        let super_constructor_target = self
            .class_context
            .as_ref()
            .and_then(|context| context.super_constructor_target.clone());
        let requires_unknown_property_hook_observation = super_constructor_target
            .as_ref()
            .is_none_or(Self::invocation_target_requires_unknown_property_hook_observation);
        let super_constructor_may_run_source = super_constructor_target
            .as_ref()
            .is_none_or(|function_id| self.function_may_run_user_code_synchronously(function_id));
        if requires_unknown_property_hook_observation {
            self.observe_all_planned_source_as_unknown_property_hooks();
        }
        if requires_unknown_property_hook_observation || super_constructor_may_run_source {
            self.invalidate_unknown_user_code_effects();
        }
        self.current_construct_this_info
            .clone()
            .unwrap_or_else(|| self.current_this_info())
    }
}
