//! Complete checked Generator and AsyncGenerator Switch share one CaseBlock.

use super::generator_loop::{ResumableGeneratorExpression, ResumableGeneratorRegion};
use super::generator_resource_scope::CompleteMixedResourceLifetime;
use super::*;
use lila_ir::{
    AsyncGeneratorSwitchCaseIr, AsyncGeneratorSwitchIr, OrdinaryGeneratorSwitchCaseIr,
    OrdinaryGeneratorSwitchIr,
};

#[derive(Clone, Copy)]
pub(super) enum ResumableGeneratorSwitch<'a> {
    Generator(&'a OrdinaryGeneratorSwitchIr),
    AsyncGenerator(&'a AsyncGeneratorSwitchIr),
}

#[derive(Clone, Copy)]
enum ResumableGeneratorSwitchCases<'a> {
    Generator(&'a [OrdinaryGeneratorSwitchCaseIr]),
    AsyncGenerator(&'a [AsyncGeneratorSwitchCaseIr]),
}

#[derive(Clone, Copy)]
enum ResumableGeneratorSwitchCase<'a> {
    Generator(&'a OrdinaryGeneratorSwitchCaseIr),
    AsyncGenerator(&'a AsyncGeneratorSwitchCaseIr),
}

impl<'a> ResumableGeneratorSwitchCases<'a> {
    fn iter(self) -> impl Iterator<Item = ResumableGeneratorSwitchCase<'a>> {
        let (generator, mixed) = match self {
            Self::Generator(cases) => (Some(cases), None),
            Self::AsyncGenerator(cases) => (None, Some(cases)),
        };
        generator
            .into_iter()
            .flat_map(|cases| cases.iter().map(ResumableGeneratorSwitchCase::Generator))
            .chain(mixed.into_iter().flat_map(|cases| {
                cases
                    .iter()
                    .map(ResumableGeneratorSwitchCase::AsyncGenerator)
            }))
    }

    fn get(self, index: usize) -> Option<ResumableGeneratorSwitchCase<'a>> {
        match self {
            Self::Generator(cases) => cases
                .get(index)
                .map(ResumableGeneratorSwitchCase::Generator),
            Self::AsyncGenerator(cases) => cases
                .get(index)
                .map(ResumableGeneratorSwitchCase::AsyncGenerator),
        }
    }
}

impl<'a> ResumableGeneratorSwitchCase<'a> {
    fn selector(self) -> Option<ResumableGeneratorExpression<'a>> {
        match self {
            Self::Generator(case) => case.selector().map(ResumableGeneratorExpression::Generator),
            Self::AsyncGenerator(case) => {
                case.selector().map(ResumableGeneratorExpression::Complete)
            }
        }
    }

    fn body(self) -> ResumableGeneratorRegion<'a> {
        match self {
            Self::Generator(case) => ResumableGeneratorRegion::Generator(case.body()),
            Self::AsyncGenerator(case) => ResumableGeneratorRegion::Complete(case.body()),
        }
    }
}

impl<'a> ResumableGeneratorSwitch<'a> {
    pub(super) fn execution_kind(self) -> FunctionExecutionKind {
        match self {
            Self::Generator(_) => FunctionExecutionKind::Generator,
            Self::AsyncGenerator(plan) => match plan.execution() {
                lila_ir::ResumableRegionProtocolIr::Generator => FunctionExecutionKind::Generator,
                lila_ir::ResumableRegionProtocolIr::Async => FunctionExecutionKind::Async,
                lila_ir::ResumableRegionProtocolIr::AsyncGenerator => {
                    FunctionExecutionKind::AsyncGenerator
                }
            },
        }
    }

    fn discriminant(self) -> ResumableGeneratorExpression<'a> {
        match self {
            Self::Generator(plan) => ResumableGeneratorExpression::Generator(plan.discriminant()),
            Self::AsyncGenerator(plan) => {
                ResumableGeneratorExpression::Complete(plan.discriminant())
            }
        }
    }

    pub(super) fn discriminant_binding(self) -> &'a OwnedEnvBindingIr {
        match self {
            Self::Generator(plan) => plan.discriminant_binding(),
            Self::AsyncGenerator(plan) => plan.discriminant_binding(),
        }
    }

    pub(super) fn value_binding(self) -> &'a OwnedEnvBindingIr {
        match self {
            Self::Generator(plan) => plan.value_binding(),
            Self::AsyncGenerator(plan) => plan.value_binding(),
        }
    }

    fn lexical_environment(self) -> Option<&'a lila_ir::LexicalEnvironmentIr> {
        match self {
            Self::Generator(plan) => plan.lexical_environment(),
            Self::AsyncGenerator(plan) => plan.lexical_environment(),
        }
    }

    fn lexical_declarations(self) -> &'a [StatementIr] {
        match self {
            Self::Generator(plan) => plan.lexical_declarations(),
            Self::AsyncGenerator(plan) => plan.lexical_declarations(),
        }
    }

    fn cases(self) -> ResumableGeneratorSwitchCases<'a> {
        match self {
            Self::Generator(plan) => ResumableGeneratorSwitchCases::Generator(plan.cases()),
            Self::AsyncGenerator(plan) => {
                ResumableGeneratorSwitchCases::AsyncGenerator(plan.cases())
            }
        }
    }

    fn entry_state(self) -> u32 {
        match self {
            Self::Generator(plan) => plan.entry_state(),
            Self::AsyncGenerator(plan) => plan.entry_state(),
        }
    }

    fn case_block_entry_state(self) -> u32 {
        match self {
            Self::Generator(plan) => plan.case_block_entry_state(),
            Self::AsyncGenerator(plan) => plan.case_block_entry_state(),
        }
    }

    fn fallback_state(self) -> u32 {
        match self {
            Self::Generator(plan) => plan.fallback_state(),
            Self::AsyncGenerator(plan) => plan.fallback_state(),
        }
    }

    fn exit_state(self) -> u32 {
        match self {
            Self::Generator(plan) => plan.exit_state(),
            Self::AsyncGenerator(plan) => plan.exit_state(),
        }
    }
}

impl FunctionBuilder<'_> {
    fn emit_generator_switch_region_guard(
        &self,
        region: ResumableGeneratorRegion<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_resumable_state_in_range(region.entry_state(), region.end_state(), true, function)
    }

    pub(super) fn compile_ordinary_generator_switch(
        &mut self,
        plan: &OrdinaryGeneratorSwitchIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_generator_switch(
            ResumableGeneratorSwitch::Generator(plan),
            labels,
            function,
        )
    }

    pub(super) fn compile_async_generator_switch(
        &mut self,
        plan: &AsyncGeneratorSwitchIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_environment_owner;
        if plan.execution() == lila_ir::ResumableRegionProtocolIr::AsyncGenerator {
            self.checked_async_generator_environment_owner =
                Some(CheckedAsyncGeneratorEnvironmentOwner::for_switch(plan));
        }
        let result = self.compile_resumable_generator_switch(
            ResumableGeneratorSwitch::AsyncGenerator(plan),
            labels,
            function,
        );
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_resumable_generator_switch(
        &mut self,
        plan: ResumableGeneratorSwitch<'_>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        // This statement may itself be inside another suspended CaseBlock.
        self.emit_checkpoint_generator_statement_list_value(function);
        self.push_scope();
        let break_target = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_target);
        self.push_labels(labels, break_target, None);
        self.begin_resumable_switch_statement_list_value(plan, break_target)?;

        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let initial = self.runtime_schema().reserve_value_local(function);
        initial.set_undefined(function);
        self.write_generator_statement_list_binding(plan.value_binding(), &initial, function);
        initial.clear(function);
        self.emit_statement_result(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // Discriminant evaluation and its actual retained publication occur
        // before CaseBlock lexical instantiation, including on resumed entry.
        let discriminant = plan.discriminant();
        self.emit_generator_switch_region_guard(discriminant.region(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_resumable_operand_region(
            discriminant.region().block(),
            discriminant.region().entry_state(),
            function,
        )?;
        self.emit_set_resumable_resume_point(plan.case_block_entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        if let Some(environment) = plan.lexical_environment() {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.case_block_entry_state(),
                function,
            )?;
        }
        self.emit_resumable_state_equals(plan.case_block_entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        for case in plan.cases().iter() {
            self.initialize_direct_lexical_bindings(&case.body().block().statements, function);
        }
        for declaration in plan.lexical_declarations() {
            self.compile_statement(declaration, function)?;
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let resource = match plan {
            ResumableGeneratorSwitch::Generator(_) => None,
            ResumableGeneratorSwitch::AsyncGenerator(plan) => {
                CompleteMixedResourceLifetime::for_switch(plan)
            }
        };
        if let Some(resource) = resource {
            // The discriminant and CaseBlock TDZ/functions are already complete.
            // One capability now spans every lazy selector and fallthrough body.
            self.compile_complete_mixed_resource_lifetime_then(
                resource,
                function,
                |builder, disposal, function| {
                    builder.compile_resumable_generator_switch_selection_and_bodies(
                        plan, disposal, function,
                    )
                },
            )?;
            self.emit_branch_to_target(break_target, function);
        } else {
            self.compile_resumable_generator_switch_selection_and_bodies(
                plan,
                break_target,
                function,
            )?;
        }
        self.pop_labels(labels.len());
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if plan.lexical_environment().is_some() {
            // Every normal/no-match/break path used the same enclosing target
            // and already unwound this one runtime record.
            self.end_lexical_environment_scope();
        }
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_save_resumable_environment(function)?;
        self.end_generator_statement_list_value();
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
    fn compile_resumable_generator_switch_selection_and_bodies(
        &mut self,
        plan: ResumableGeneratorSwitch<'_>,
        normal_exit: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Each selector owns its whole region. A match commits a body state,
        // so following selectors are skipped without a retained Wasm local.
        for (index, case) in plan.cases().iter().enumerate() {
            let Some(selector) = case.selector() else {
                continue;
            };
            self.emit_generator_switch_region_guard(selector.region(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.compile_resumable_operand_region_then(
                selector.region().block(),
                selector.region().entry_state(),
                function,
                |builder, function| {
                    let value = builder.runtime_schema().reserve_value_local(function);
                    builder.read_generator_statement_list_binding(
                        plan.discriminant_binding(),
                        &value,
                        function,
                    );
                    let result =
                        builder.compile_switch_case_match(&value, selector.value(), function);
                    value.clear(function);
                    result
                },
            )?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_set_resumable_resume_point(case.body().entry_state(), function)?;
            function.instruction(&Instruction::Else);
            let next = plan
                .cases()
                .iter()
                .skip(index + 1)
                .find_map(|case| {
                    case.selector()
                        .map(|selector| selector.region().entry_state())
                })
                .unwrap_or(plan.fallback_state());
            self.emit_set_resumable_resume_point(next, function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }

        self.emit_resumable_state_equals(plan.fallback_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        if let Some(default) = plan.cases().iter().find(|case| case.selector().is_none()) {
            self.emit_set_resumable_resume_point(default.body().entry_state(), function)?;
        } else {
            self.emit_statement_result(function);
            self.emit_branch_to_target(normal_exit, function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        self.activate_generator_statement_list_value();
        for (index, case) in plan.cases().iter().enumerate() {
            self.emit_generator_switch_region_guard(case.body(), function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.emit_restore_generator_statement_list_value(function);
            // Every case shares the already-instantiated CaseBlock. Repeating
            // lexical initialization here would incorrectly re-enter TDZ.
            self.compile_resumable_block_contents(
                case.body().block(),
                case.body().entry_state(),
                false,
                function,
            )?;
            self.emit_checkpoint_generator_statement_list_value(function);
            if let Some(next) = plan.cases().get(index + 1) {
                self.emit_set_resumable_resume_point(next.body().entry_state(), function)?;
            } else {
                self.emit_branch_to_target(normal_exit, function);
            }
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }
}
