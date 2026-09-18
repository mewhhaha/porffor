//! Lower exact compiler-owned module operations through the ordinary AOT pipeline.

use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_module_instantiation_boundary(
        &mut self,
        statement: &Statement,
    ) -> Option<(StatementIr, ValueKind)> {
        if !self
            .analysis
            .synchronous_modules
            .boundaries
            .contains(&(std::ptr::from_ref(statement) as usize))
        {
            return None;
        }
        let owner = self
            .current_function_id
            .as_ref()
            .and_then(|id| self.analysis.function_plans.get(id))
            .expect("an instantiation boundary belongs to a module owner");
        assert_eq!(owner.protocol, FunctionProtocolIr::ModuleActivation);
        assert!(self.current_generator_resume_state.is_none());
        assert!(self.current_async_resume_state.is_none());
        // Evaluation may run after other modules or escaped declarations
        // mutate shared state, so instantiation flow facts end here.
        self.invalidate_unknown_user_code_effects();
        let boundary = GeneratorPlanIr::MODULE_INSTANTIATION;
        Some((
            StatementIr::GeneratorYield {
                value: TypedExpr::undefined(),
                form: YieldForm::Plain,
                suspend_state: boundary.suspend_state,
                resume_state: boundary.resume_state,
                resume_mode: GeneratorResumeModeIr::Ignore,
            },
            ValueKind::Undefined,
        ))
    }

    pub(super) fn lower_synchronous_module_expression(
        &mut self,
        expression: &Expression,
    ) -> Option<TypedExpr> {
        let pointer = std::ptr::from_ref(expression) as usize;
        let operations = &self.analysis.synchronous_modules;
        let operation = if let Some(cell) = operations.reads.get(&pointer) {
            ExprIr::ModuleBindingRead(cell.clone())
        } else if let Some(evaluation) = operations.evaluations.get(&pointer) {
            ExprIr::ModuleEvaluate(evaluation.clone())
        } else if let Some(evaluation) = operations.deferred_evaluations.get(&pointer) {
            ExprIr::DeferredModuleEvaluate(evaluation.clone())
        } else if let Expression::ArrayLiteral(array) = expression {
            let graph = operations
                .graphs
                .get(&(std::ptr::from_ref(array) as usize))?;
            ExprIr::SynchronousModuleGraph(Box::new(graph.clone()))
        } else {
            return None;
        };
        self.invalidate_unknown_user_code_effects();
        let info = if matches!(operation, ExprIr::ModuleBindingRead(_)) {
            unknown_runtime_value_info()
        } else {
            ValueInfo::undefined()
        };
        Some(TypedExpr::from_info(info, operation))
    }
}
