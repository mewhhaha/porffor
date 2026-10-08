//! Checked Generator and AsyncGenerator phases share their physical loop owner.

use super::generator_resource_scope::CompleteMixedResourceLifetime;
use super::*;
use lila_ir::{
    AsyncGeneratorIfIr, AsyncGeneratorLoopExpressionIr, AsyncGeneratorLoopIr,
    AsyncGeneratorLoopRegionIr, ForLexicalEnvironmentIr, GeneratorLoopExpressionIr,
    GeneratorLoopKindIr, GeneratorLoopRegionIr, OrdinaryGeneratorIfIr, OrdinaryGeneratorLoopIr,
    ResumableExpressionIr, ResumableRegionIr, ResumableRegionProtocolIr,
};

/// Both variants carry a complete checked source plan. The old linear async
/// loop cannot enter this physical phase lifecycle.
#[derive(Clone, Copy)]
pub(super) enum ResumableGeneratorLoop<'a> {
    Generator(&'a OrdinaryGeneratorLoopIr),
    AsyncGenerator(&'a AsyncGeneratorLoopIr),
}

/// A complete region permits fresh/resumed records. An enclosing source scope
/// restores only the source-certified enclosing records at their resume states.
#[derive(Clone, Copy)]
pub(crate) struct CheckedAsyncGeneratorEnvironmentOwner(AsyncGeneratorEnvironmentOrigin);

#[derive(Clone, Copy)]
enum AsyncGeneratorEnvironmentOrigin {
    Region,
    FunctionSource,
}

impl CheckedAsyncGeneratorEnvironmentOwner {
    fn for_loop(_plan: &AsyncGeneratorLoopIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }
    fn for_if(_plan: &AsyncGeneratorIfIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(super) fn for_with(_plan: &lila_ir::AsyncGeneratorWithIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(super) fn for_switch(_plan: &lila_ir::AsyncGeneratorSwitchIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(super) fn for_for_in(_plan: &lila_ir::AsyncGeneratorForInIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(super) fn for_array_destructuring(
        _plan: &lila_ir::AsyncGeneratorArrayDestructuringIr,
    ) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(super) fn for_for_of(_plan: &lila_ir::AsyncGeneratorForOfIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(super) fn for_resource_scope(_plan: &lila_ir::AsyncGeneratorResourceScopeIr) -> Self {
        Self(AsyncGeneratorEnvironmentOrigin::Region)
    }

    pub(crate) fn for_source(
        plan: &lila_ir::AsyncGeneratorResumeEnvironmentPlanIr,
    ) -> Option<Self> {
        (!plan.invocation_resume_states().is_empty()
            || !plan.enclosing_scope_resume_states().is_empty())
        .then_some(Self(AsyncGeneratorEnvironmentOrigin::FunctionSource))
    }

    pub(crate) fn is_region(self) -> bool {
        match self.0 {
            AsyncGeneratorEnvironmentOrigin::Region => true,
            AsyncGeneratorEnvironmentOrigin::FunctionSource => false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ResumableGeneratorRegion<'a> {
    Generator(&'a GeneratorLoopRegionIr),
    AsyncGenerator(&'a AsyncGeneratorLoopRegionIr),
    Complete(&'a ResumableRegionIr),
}

#[derive(Clone, Copy)]
pub(super) enum ResumableGeneratorExpression<'a> {
    Generator(&'a GeneratorLoopExpressionIr),
    AsyncGenerator(&'a AsyncGeneratorLoopExpressionIr),
    Complete(&'a ResumableExpressionIr),
}

impl<'a> ResumableGeneratorRegion<'a> {
    pub(super) fn block(self) -> &'a BlockIr {
        match self {
            Self::Generator(region) => region.block(),
            Self::AsyncGenerator(region) => region.block(),
            Self::Complete(region) => region.block(),
        }
    }
    pub(super) fn entry_state(self) -> u32 {
        match self {
            Self::Generator(region) => region.entry_state(),
            Self::AsyncGenerator(region) => region.entry_state(),
            Self::Complete(region) => region.entry_state(),
        }
    }
    pub(super) fn end_state(self) -> u32 {
        match self {
            Self::Generator(region) => region.end_state(),
            Self::AsyncGenerator(region) => region.end_state(),
            Self::Complete(region) => region.end_state(),
        }
    }
}

impl<'a> ResumableGeneratorExpression<'a> {
    pub(super) fn region(self) -> ResumableGeneratorRegion<'a> {
        match self {
            Self::Generator(value) => ResumableGeneratorRegion::Generator(value.region()),
            Self::AsyncGenerator(value) => ResumableGeneratorRegion::AsyncGenerator(value.region()),
            Self::Complete(value) => ResumableGeneratorRegion::Complete(value.region()),
        }
    }
    pub(super) fn value(self) -> &'a TypedExpr {
        match self {
            Self::Generator(value) => value.value(),
            Self::AsyncGenerator(value) => value.value(),
            Self::Complete(value) => value.value(),
        }
    }
}

impl<'a> ResumableGeneratorLoop<'a> {
    pub(super) fn execution_kind(self) -> FunctionExecutionKind {
        match self {
            Self::Generator(_) => FunctionExecutionKind::Generator,
            Self::AsyncGenerator(plan) => match plan.execution() {
                ResumableRegionProtocolIr::Generator => FunctionExecutionKind::Generator,
                ResumableRegionProtocolIr::Async => FunctionExecutionKind::Async,
                ResumableRegionProtocolIr::AsyncGenerator => FunctionExecutionKind::AsyncGenerator,
            },
        }
    }
    fn kind(self) -> GeneratorLoopKindIr {
        match self {
            Self::Generator(plan) => plan.kind(),
            Self::AsyncGenerator(plan) => plan.kind(),
        }
    }
    fn initialization(self) -> Option<ResumableGeneratorRegion<'a>> {
        match self {
            Self::Generator(plan) => plan
                .initialization()
                .map(ResumableGeneratorRegion::Generator),
            Self::AsyncGenerator(plan) => plan
                .initialization()
                .map(ResumableGeneratorRegion::Complete),
        }
    }
    fn test(self) -> ResumableGeneratorExpression<'a> {
        match self {
            Self::Generator(plan) => ResumableGeneratorExpression::Generator(plan.test()),
            Self::AsyncGenerator(plan) => ResumableGeneratorExpression::Complete(plan.test()),
        }
    }
    fn body(self) -> ResumableGeneratorRegion<'a> {
        match self {
            Self::Generator(plan) => ResumableGeneratorRegion::Generator(plan.body()),
            Self::AsyncGenerator(plan) => ResumableGeneratorRegion::Complete(plan.body()),
        }
    }
    fn update(self) -> Option<ResumableGeneratorExpression<'a>> {
        match self {
            Self::Generator(plan) => plan.update().map(ResumableGeneratorExpression::Generator),
            Self::AsyncGenerator(plan) => plan.update().map(ResumableGeneratorExpression::Complete),
        }
    }
    fn lexical_environment(self) -> Option<&'a ForLexicalEnvironmentIr> {
        match self {
            Self::Generator(plan) => plan.lexical_environment(),
            Self::AsyncGenerator(plan) => plan.lexical_environment(),
        }
    }
    pub(super) fn value_binding(self) -> &'a OwnedEnvBindingIr {
        match self {
            Self::Generator(plan) => plan.value_binding(),
            Self::AsyncGenerator(plan) => plan.value_binding(),
        }
    }
    fn entry_state(self) -> u32 {
        match self {
            Self::Generator(plan) => plan.entry_state(),
            Self::AsyncGenerator(plan) => plan.entry_state(),
        }
    }
    fn continue_state(self) -> u32 {
        match self {
            Self::Generator(plan) => plan.continue_state(),
            Self::AsyncGenerator(plan) => plan.continue_state(),
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
    fn restore_resumable_loop_value(
        &mut self,
        plan: ResumableGeneratorLoop<'_>,
        function: &mut Function,
    ) {
        let value = self.runtime_schema().reserve_value_local(function);
        self.read_generator_statement_list_binding(plan.value_binding(), &value, function);
        self.completion().set_normal(&value, function);
        value.clear(function);
    }

    fn save_resumable_loop_value(
        &mut self,
        plan: ResumableGeneratorLoop<'_>,
        function: &mut Function,
    ) {
        let value = self.runtime_schema().reserve_value_local(function);
        value.copy_from(self.completion().value(), function);
        self.write_generator_statement_list_binding(plan.value_binding(), &value, function);
        value.clear(function);
    }

    fn compile_resumable_generator_region(
        &mut self,
        region: ResumableGeneratorRegion<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.push_scope();
        self.compile_resumable_block_contents(
            region.block(),
            region.entry_state(),
            true,
            function,
        )?;
        self.pop_scope();
        Ok(())
    }

    fn emit_resumable_generator_region_guard(
        &self,
        region: ResumableGeneratorRegion<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_resumable_state_in_range(region.entry_state(), region.end_state(), true, function)
    }

    fn compile_resumable_generator_loop_test(
        &mut self,
        plan: ResumableGeneratorLoop<'_>,
        test: ResumableGeneratorExpression<'_>,
        break_frame: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_resumable_generator_region_guard(test.region(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_resumable_operand_region_then(
            test.region().block(),
            test.region().entry_state(),
            function,
            |builder, function| {
                builder.restore_resumable_loop_value(plan, function);
                builder.compile_classic_for_test(test.value(), break_frame, function)
            },
        )?;
        self.emit_set_resumable_resume_point(plan.body().entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(super) fn compile_ordinary_generator_loop(
        &mut self,
        plan: &OrdinaryGeneratorLoopIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_generator_loop(
            ResumableGeneratorLoop::Generator(plan),
            labels,
            function,
        )
    }

    pub(super) fn compile_async_generator_loop(
        &mut self,
        plan: &AsyncGeneratorLoopIr,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if plan.execution() != ResumableRegionProtocolIr::AsyncGenerator {
            return self.compile_resumable_generator_loop(
                ResumableGeneratorLoop::AsyncGenerator(plan),
                labels,
                function,
            );
        }
        let previous = self.checked_async_generator_environment_owner.take();
        self.checked_async_generator_environment_owner =
            Some(CheckedAsyncGeneratorEnvironmentOwner::for_loop(plan));
        let result = self.compile_resumable_generator_loop(
            ResumableGeneratorLoop::AsyncGenerator(plan),
            labels,
            function,
        );
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_resumable_generator_loop(
        &mut self,
        plan: ResumableGeneratorLoop<'_>,
        labels: &[String],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == plan.execution_kind())
            || self
                .body_entry_locals()
                .and_then(|entry| entry.resume_frame())
                .is_none()
            || !self
                .owned_env_bindings
                .iter()
                .any(|binding| binding == plan.value_binding())
        {
            return Err(EmitError::unsupported(
                "compiler invariant: generator loop requires its exact source activation",
            ));
        }
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        // Break unwinds the loop's lexical record on every path, including a
        // false head or a resumed finalizer delivering its saved branch.
        let break_frame = self.open_frame(ControlFrameKind::Block, function);
        self.breakable_stack.push(break_frame);
        let mixed = matches!(plan, ResumableGeneratorLoop::AsyncGenerator(_));
        if mixed {
            self.begin_resumable_classic_loop_value(plan, break_frame)?;
        }
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let initial = self.runtime_schema().reserve_value_local(function);
        initial.set_undefined(function);
        // This is compiler-private storage in the checked invocation, not a
        // source declaration. Nested lexical records cannot change its owner.
        self.write_generator_statement_list_binding(plan.value_binding(), &initial, function);
        initial.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        let runtime_environment =
            plan.lexical_environment()
                .map(|environment| LexicalEnvironmentIr {
                    initialization: lila_ir::LexicalEnvironmentInitializationIr::Uninitialized,
                    eval_environment: environment.eval_environment.clone(),
                    bindings: environment.bindings.clone(),
                });
        if let Some(environment) = &runtime_environment {
            self.emit_enter_resumable_lexical_environment(
                environment,
                plan.entry_state(),
                function,
            )?;
        }
        let schema = self.runtime_schema();
        let body_active = schema.reserve_i32_local(function);
        let resource = match plan {
            ResumableGeneratorLoop::Generator(_) => None,
            ResumableGeneratorLoop::AsyncGenerator(plan) => {
                CompleteMixedResourceLifetime::for_loop(plan)
            }
        };
        if let Some(resource) = resource {
            self.compile_complete_mixed_resource_lifetime_then(
                resource,
                function,
                |builder, disposal, function| {
                    builder.compile_resumable_generator_loop_phases(
                        plan,
                        labels,
                        break_frame,
                        disposal,
                        body_active,
                        function,
                    )
                },
            )?;
            // The capability has completed while the original loop record is
            // still attached. Its own normal exit now unwinds that record.
            self.emit_branch_to_target(break_frame, function);
        } else {
            self.compile_resumable_generator_loop_phases(
                plan,
                labels,
                break_frame,
                break_frame,
                body_active,
                function,
            )?;
        }
        self.breakable_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        if runtime_environment.is_some() {
            self.end_lexical_environment_scope();
        }
        self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_save_resumable_environment(function)?;
        let retired = schema.reserve_value_local(function);
        retired.set_undefined(function);
        self.write_generator_statement_list_binding(plan.value_binding(), &retired, function);
        retired.clear(function);
        if mixed {
            self.end_generator_statement_list_value();
        }
        self.pop_scope();
        schema.release_i32_local(body_active, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn compile_resumable_generator_loop_phases(
        &mut self,
        plan: ResumableGeneratorLoop<'_>,
        labels: &[String],
        break_frame: ControlTarget,
        normal_exit: ControlTarget,
        body_active: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let mixed = matches!(plan, ResumableGeneratorLoop::AsyncGenerator(_));
        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);
        let continue_frame = self.open_frame(ControlFrameKind::Block, function);
        self.loop_stack.push(LoopTargets { continue_frame });
        self.push_labels(labels, break_frame, Some(continue_frame));
        function.instruction(&Instruction::I32Const(0));
        body_active.store(function);

        if let Some(initialization) = plan.initialization() {
            self.emit_resumable_generator_region_guard(initialization, function)?;
            self.open_frame(ControlFrameKind::If, function);
            self.compile_resumable_operand_region_in_current_scope(
                initialization.block(),
                initialization.entry_state(),
                function,
                |_, _| Ok(()),
            )?;
            if let Some(environment) = plan.lexical_environment() {
                self.emit_replace_lexical_environment(environment, function)?;
            }
            self.restore_resumable_loop_value(plan, function);
            self.emit_set_resumable_resume_point(plan.test().region().entry_state(), function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        match plan.kind() {
            GeneratorLoopKindIr::For | GeneratorLoopKindIr::While => {
                self.compile_resumable_generator_loop_test(
                    plan,
                    plan.test(),
                    normal_exit,
                    function,
                )?;
            }
            GeneratorLoopKindIr::DoWhile => {}
        }

        if mixed {
            self.activate_generator_statement_list_value();
        }
        self.emit_resumable_generator_region_guard(plan.body(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(1));
        body_active.store(function);
        self.restore_resumable_loop_value(plan, function);
        self.compile_resumable_generator_region(plan.body(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        // A normal body and Continue meet here. This flag is computed afresh
        // before either path, never retained in a Wasm local across suspension.
        body_active.load(function);
        self.open_frame(ControlFrameKind::If, function);
        self.save_resumable_loop_value(plan, function);
        self.emit_set_resumable_resume_point(plan.continue_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        if mixed {
            self.deactivate_generator_statement_list_value();
        }

        // The body-continue label is closed now. Head finalizers see only live
        // control labels; the checked expression phases have no source branch
        // targeting the body's Continue destination.
        self.loop_stack.push(LoopTargets {
            continue_frame: loop_frame,
        });
        self.push_labels(labels, break_frame, Some(loop_frame));
        match plan.kind() {
            GeneratorLoopKindIr::For => {
                let update = plan.update().expect("checked For owns an update phase");
                self.emit_resumable_generator_region_guard(update.region(), function)?;
                self.open_frame(ControlFrameKind::If, function);
                self.emit_resumable_state_equals(update.region().entry_state(), function)?;
                self.open_frame(ControlFrameKind::If, function);
                if let Some(environment) = plan.lexical_environment() {
                    self.emit_replace_lexical_environment(environment, function)?;
                }
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.compile_resumable_operand_region_then(
                    update.region().block(),
                    update.region().entry_state(),
                    function,
                    |builder, function| {
                        builder.restore_resumable_loop_value(plan, function);
                        builder.compile_classic_for_update(update.value(), function)
                    },
                )?;
                self.emit_set_resumable_resume_point(plan.test().region().entry_state(), function)?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
            GeneratorLoopKindIr::While => {}
            GeneratorLoopKindIr::DoWhile => {
                self.compile_resumable_generator_loop_test(
                    plan,
                    plan.test(),
                    normal_exit,
                    function,
                )?;
            }
        }
        self.pop_labels(labels.len());
        self.loop_stack.pop();
        function.branch_to_label(loop_frame.label);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(super) fn compile_async_generator_if(
        &mut self,
        plan: &AsyncGeneratorIfIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_environment_owner.take();
        self.checked_async_generator_environment_owner =
            Some(CheckedAsyncGeneratorEnvironmentOwner::for_if(plan));
        let result = self.compile_checked_async_generator_if(plan, function);
        self.checked_async_generator_environment_owner = previous;
        result
    }

    fn compile_checked_async_generator_if(
        &mut self,
        plan: &AsyncGeneratorIfIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.current_function_meta().is_some_and(|meta| {
            meta.protocol().execution_kind() == FunctionExecutionKind::AsyncGenerator
        }) {
            return Err(EmitError::unsupported(
                "compiler invariant: mixed generator branch requires its exact source activation",
            ));
        }
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        let condition = plan.condition();
        self.emit_resumable_generator_region_guard(
            ResumableGeneratorRegion::AsyncGenerator(condition.region()),
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_resumable_operand_region_in_current_scope(
            condition.region().block(),
            condition.region().entry_state(),
            function,
            |builder, function| {
                builder.compile_truthy_i32(condition.value(), function)?;
                builder.emit_propagate_current_throw_if_needed(function);
                Ok(())
            },
        )?;
        self.emit_statement_result(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_set_resumable_resume_point(plan.then_branch().entry_state(), function)?;
        function.instruction(&Instruction::Else);
        self.emit_set_resumable_resume_point(plan.else_branch().entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for region in [plan.then_branch(), plan.else_branch()] {
            self.emit_resumable_generator_region_guard(
                ResumableGeneratorRegion::AsyncGenerator(region),
                function,
            )?;
            self.open_frame(ControlFrameKind::If, function);
            self.compile_resumable_generator_region(
                ResumableGeneratorRegion::AsyncGenerator(region),
                function,
            )?;
            self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(super) fn compile_ordinary_generator_if(
        &mut self,
        plan: &OrdinaryGeneratorIfIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.compile_truthy_i32(plan.condition(), function)?;
        self.emit_propagate_current_throw_if_needed(function);
        self.emit_statement_result(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_set_resumable_resume_point(plan.then_branch().entry_state(), function)?;
        function.instruction(&Instruction::Else);
        self.emit_set_resumable_resume_point(plan.else_branch().entry_state(), function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        for region in [plan.then_branch(), plan.else_branch()] {
            self.emit_resumable_generator_region_guard(
                ResumableGeneratorRegion::Generator(region),
                function,
            )?;
            self.open_frame(ControlFrameKind::If, function);
            self.compile_resumable_generator_region(
                ResumableGeneratorRegion::Generator(region),
                function,
            )?;
            self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
