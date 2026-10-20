//! A suspended With retains the original analyzed Object Environment Record.

use super::*;
use crate::generator_loop_control::GeneratorLoopSourceRange;

impl ScriptLowerer<'_> {
    pub(super) fn lower_ordinary_generator_with(
        &mut self,
        with: &boa_ast::statement::With,
    ) -> (StatementIr, ValueKind) {
        let lowered = self.lower_checked_generator_with(with);
        match lowered {
            Some((plan, kind)) => (StatementIr::OrdinaryGeneratorWith(Box::new(plan)), kind),
            None => {
                self.unsupported(
                    "ordinary generator With differs from its complete source and environment plan",
                );
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn lower_checked_generator_with(
        &mut self,
        with: &boa_ast::statement::With,
    ) -> Option<(OrdinaryGeneratorWithIr, ValueKind)> {
        let source = GeneratorWithSource::new(with)?;
        let states = source.states(self.plain_generator_entry_state()?)?;
        let (mut prefix, raw) =
            self.lower_staged_generator_expression(source.source().expression())?;
        // ToObject occurs once outside the new With environment, after the
        // complete head. Its exact boxed result is retained across body resumes.
        let object = TypedExpr::spec_to_object(raw);
        let object_info = object.value_info();
        let head_name =
            self.alloc_suspension_owned_binding("generator.with.head.", object_info.clone());
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
        let head = GeneratorLoopExpressionIr::new(
            self.finish_generator_with_region(prefix, ValueKind::Undefined, states.head())?,
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
        self.current_generator_resume_state = Some(states.body().entry);
        self.with_environment_chain.enter_current(
            with_object,
            CurrentScopeDepth::at_with_entry(self.scopes.len()),
        );
        self.ordinary_generator_region_depth += 1;
        let (statement, kind) = self.lower_statement(source.source().statement());
        self.ordinary_generator_region_depth -= 1;
        self.with_environment_chain.leave_current();
        // The carrier alone owns the With record. A genuine nested source Block
        // keeps its own separate lexical environment inside this region.
        let body = self.finish_generator_with_region(vec![statement], kind, states.body())?;
        let plan = OrdinaryGeneratorWithIr::new(
            states,
            head,
            head_binding,
            object_binding,
            lexical_environment,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        self.current_generator_resume_state = Some(plan.exit_state());
        Some((plan, kind))
    }

    fn finish_generator_with_region(
        &self,
        statements: Vec<StatementIr>,
        kind: ValueKind,
        range: GeneratorLoopSourceRange,
    ) -> Option<GeneratorLoopRegionIr> {
        if self.plain_generator_entry_state()? != range.end {
            return None;
        }
        GeneratorLoopRegionIr::new(
            BlockIr {
                statements,
                result_kind: kind,
                lexical_environment: None,
            },
            range,
        )
        .ok()
    }
}
