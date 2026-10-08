//! Complete mixed With phases use the original analyzed Object Environment.

use super::*;
use crate::async_generator_source::AsyncGeneratorWithSource;

impl ScriptLowerer<'_> {
    pub(super) fn lower_async_generator_with(
        &mut self,
        source: AsyncGeneratorWithSource<'_>,
    ) -> (StatementIr, ValueKind) {
        match self.lower_checked_async_generator_with(source) {
            Some((plan, kind)) => (StatementIr::AsyncGeneratorWith(Box::new(plan)), kind),
            None => {
                self.unsupported("mixed async-generator With differs from its checked source and original environment");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn lower_checked_async_generator_with(
        &mut self,
        source: AsyncGeneratorWithSource<'_>,
    ) -> Option<(AsyncGeneratorWithIr, ValueKind)> {
        let states = source.states(self.async_generator_entry_state()?)?;
        let with = source.source();
        self.set_async_generator_phase(states.head().entry());
        let (mut prefix, raw) = self.lower_mixed_generator_value(with.expression())?;
        // Publish the sole completed ToObject result before entering this With.
        let object = TypedExpr::spec_to_object(raw);
        let object_info = object.value_info();
        let head_name =
            self.alloc_suspension_owned_binding("async.generator.with.head.", object_info.clone());
        let head_binding = self
            .generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == head_name)?
            .clone();
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: head_name.clone(),
            init: object,
        });
        let head = AsyncGeneratorLoopExpressionIr::new(
            self.finish_async_generator_region(
                BlockIr {
                    statements: prefix,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.head(),
            )
            .ok()?,
            TypedExpr::from_info(object_info.clone(), ExprIr::Identifier(head_name)),
        );
        let environment_id = *self
            .analysis
            .with_environment_ids
            .get(&(with as *const boa_ast::statement::With as usize))?;
        let lexical_environment = self.lower_runtime_lexical_environment(Some(environment_id))?;
        let binding_name = self
            .analysis
            .with_object_environment_plans
            .get(&environment_id)?
            .binding_name
            .clone();
        let object_binding = lexical_environment
            .bindings
            .iter()
            .find(|binding| binding.name == binding_name.as_str())?
            .clone();
        let with_object = ObjectEnvironmentBindingObject::materialized(&binding_name, object_info);
        self.set_async_generator_phase(states.body().entry());
        self.with_environment_chain.enter_current(
            with_object,
            CurrentScopeDepth::at_with_entry(self.scopes.len()),
        );
        self.mixed_async_generator_region_depth += 1;
        let (statement, kind) = self.lower_statement(with.statement());
        self.mixed_async_generator_region_depth -= 1;
        self.with_environment_chain.leave_current();
        // The With carrier owns this record; the original source Block, when
        // present, remains a separate nested lexical environment.
        let body = self
            .finish_async_generator_region(
                BlockIr {
                    statements: vec![statement],
                    result_kind: kind,
                    lexical_environment: None,
                },
                states.body(),
            )
            .ok()?;
        let plan = AsyncGeneratorWithIr::new(
            states,
            head,
            head_binding,
            object_binding,
            lexical_environment,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        self.set_async_generator_phase(plan.exit_state());
        Some((plan, kind))
    }
}
