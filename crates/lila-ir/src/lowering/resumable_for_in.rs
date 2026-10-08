//! One physical complete head, protocol-owned key initialization and body pipeline.
use super::resumable_operand::ResumableOperandProtocol;
use super::*;
use crate::async_generator_source::{AsyncGeneratorForInSource, AsyncGeneratorForInSourceStates};

impl ScriptLowerer<'_> {
    pub(super) fn lower_resumable_for_in(
        &mut self,
        checked: AsyncGeneratorForInSource<'_>,
        states: AsyncGeneratorForInSourceStates,
        proof: GeneratorForInHeadProof,
    ) -> Option<(StatementIr, ValueKind)> {
        let source = checked.source();
        let execution = states.execution();
        let protocol = ResumableOperandProtocol::for_execution(execution);
        let original_head = self.prepare_for_in_iteration_head(source)?;
        let lexical_environment = self.lower_for_in_of_environment(source as *const _ as usize);
        protocol.set_phase(self, states.head().entry());
        let mut prefix = if let IterableLoopInitializer::Var(variable) = source.initializer() {
            if variable.init().is_some_and(|value| {
                contains(value, ContainsSymbol::YieldExpression)
                    || contains(value, ContainsSymbol::AwaitExpression)
            }) {
                self.lower_resumable_var_identifier_initializer(variable, protocol)?
            } else {
                self.lower_for_in_initializer_prefix(source.initializer())
                    .into_iter()
                    .collect()
            }
        } else {
            Vec::new()
        };
        let (staged, mut raw) =
            self.lower_for_in_head_target(&original_head, source.target(), |this, expression| {
                protocol.lower(this, expression)
            })?;
        prefix.extend(staged);
        self.widen_for_in_parameter_target(&mut raw, source.target());
        let info = raw.value_info();
        let head_binding =
            self.allocate_resumable_for_in_binding("resumable.forin.head.", info.clone());
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: head_binding.name.clone(),
            init: raw,
        });
        (protocol.entry_state(self)? == states.head().end()).then_some(())?;
        let head = ResumableExpressionIr::new(
            ResumableRegionIr::new(
                BlockIr {
                    statements: prefix,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.head(),
                execution,
            )
            .ok()?,
            TypedExpr::from_info(info, ExprIr::Identifier(head_binding.name.clone())),
        );
        let enumerator_binding = self.allocate_resumable_for_in_binding(
            "resumable.forin.enumerator.",
            ValueInfo::undefined(),
        );
        let key_binding = self.allocate_resumable_for_in_binding(
            "resumable.forin.key.",
            ValueInfo::new(ValueKind::String),
        );
        let value_binding = self
            .allocate_resumable_for_in_binding("resumable.forin.value.", ValueInfo::undefined());
        // OwnKeys, descriptor and prototype traps can replace source facts
        // before the selected key is assigned through its original Reference.
        self.invalidate_unknown_user_code_effects();
        let (initialization, ignored) = self.lower_checked_async_generator_for_in_initializer(
            source,
            &states,
            &original_head,
            &key_binding,
            lexical_environment.as_ref(),
        )?;
        let proof = proof.validate_identifier_write(source, ignored)?;
        protocol.set_phase(self, states.body().entry());
        let previous_context = self.async_value_branch_context;
        match execution {
            ResumableRegionProtocolIr::Generator => self.ordinary_generator_for_in_depth += 1,
            ResumableRegionProtocolIr::Async => {
                self.plain_async_for_in_depth += 1;
                self.async_value_branch_context = AsyncValueBranchContext::ForInBody {
                    loop_depth: self.loop_depth + 1,
                };
            }
            ResumableRegionProtocolIr::AsyncGenerator => {
                self.mixed_async_generator_region_depth += 1
            }
        }
        let (body, kind) = self.lower_loop_body(source.body());
        match execution {
            ResumableRegionProtocolIr::Generator => self.ordinary_generator_for_in_depth -= 1,
            ResumableRegionProtocolIr::Async => self.plain_async_for_in_depth -= 1,
            ResumableRegionProtocolIr::AsyncGenerator => {
                self.mixed_async_generator_region_depth -= 1
            }
        }
        self.async_value_branch_context = previous_context;
        (protocol.entry_state(self)? == states.body().end()).then_some(())?;
        // The actual source Block keeps its own nested lexical record. This
        // owner attaches only the original per-iteration environment.
        let body = ResumableRegionIr::new(
            BlockIr {
                statements: vec![body],
                result_kind: kind,
                lexical_environment: None,
            },
            states.body(),
            execution,
        )
        .ok()?;
        let exit = states.exit();
        let plan = AsyncGeneratorForInIr::new(
            states,
            proof,
            head,
            head_binding,
            initialization,
            lexical_environment,
            enumerator_binding,
            key_binding,
            value_binding,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        protocol.set_phase(self, exit);
        Some((StatementIr::AsyncGeneratorForIn(Box::new(plan)), kind))
    }

    fn allocate_resumable_for_in_binding(
        &mut self,
        hint: &str,
        info: ValueInfo,
    ) -> OwnedEnvBindingIr {
        let name = self.alloc_suspension_owned_binding(hint, info);
        self.generated_owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .expect("complete ForIn uses its actual allocated activation row")
            .clone()
    }
}
