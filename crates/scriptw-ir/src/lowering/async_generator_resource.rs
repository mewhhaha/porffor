//! Staged acquisition retains one original capability for the lexical list.
use super::async_disposable::LoweredStatementListItemIr;
use super::pattern_target::{owned_pattern_binding, retain_pattern_value};
use super::*;
use crate::async_generator_source::{
    AsyncGeneratorResourceRegistrationSource, AsyncGeneratorResourceScopeSource,
    AsyncGeneratorScopedResourceSourceStates, AsyncGeneratorSourceRange,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckedAsyncGeneratorResourceRegistration {
    source: AsyncGeneratorResourceRegistrationSource,
    capability_binding: OwnedEnvBindingIr,
    binding_name: String,
    initializer: TypedExpr,
}
impl CheckedAsyncGeneratorResourceRegistration {
    pub(crate) fn source(&self) -> &AsyncGeneratorResourceRegistrationSource {
        &self.source
    }
    pub(crate) fn capability_binding(&self) -> &OwnedEnvBindingIr {
        &self.capability_binding
    }
    pub(crate) fn binding_name(&self) -> &str {
        &self.binding_name
    }
    pub(crate) fn initializer(&self) -> &TypedExpr {
        &self.initializer
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_async_generator_resource_items(
        &mut self,
        source: AsyncGeneratorResourceScopeSource<'_>,
        scope: &mut LexicalScopeInstantiation,
        mut prefix: Vec<StatementIr>,
        placement: StatementListPlacement,
    ) -> Option<(StatementIr, ValueKind)> {
        let execution = source.execution();
        let states = source.states(self.resource_entry_state(execution)?)?;
        let capability = owned_pattern_binding(
            self,
            "async.generator.resource.capability.",
            ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            },
        );
        let registrations = states.registrations().to_vec();
        let mut next_registration = 0;
        let mut result_kind = ValueKind::Undefined;
        let previous_context = self.async_value_branch_context;
        match execution {
            ResumableRegionProtocolIr::Generator => self.ordinary_generator_region_depth += 1,
            ResumableRegionProtocolIr::Async => {
                self.plain_async_resource_depth += 1;
                self.async_value_branch_context = AsyncValueBranchContext::ResourceScope {
                    loop_depth: self.loop_depth,
                };
            }
            ResumableRegionProtocolIr::AsyncGenerator => {
                self.mixed_async_generator_region_depth += 1
            }
        }
        let lowered = (|| {
            for item in source.items() {
                if let StatementListItem::Declaration(declaration) = item {
                    let variables = match declaration.as_ref() {
                        Declaration::Lexical(
                            LexicalDeclaration::Using(list) | LexicalDeclaration::AwaitUsing(list),
                        ) => Some(list.as_ref()),
                        _ => None,
                    };
                    if let Some(variables) = variables {
                        let mut declaration_prefix = Vec::new();
                        for variable in variables {
                            let descriptor = registrations.get(next_registration)?;
                            self.lower_checked_async_generator_resource_registration(
                                variable,
                                descriptor,
                                &capability,
                                scope,
                                &mut declaration_prefix,
                            )?;
                            next_registration += 1;
                        }
                        prefix.push(StatementIr::EmptyStatementCompletion(Box::new(
                            EmptyStatementCompletionIr::new(
                                CheckedEmptyStatementCompletionSource::from_declaration(
                                    declaration,
                                ),
                                StatementIr::LexicalBlock(declaration_prefix),
                            ),
                        )));
                        result_kind = ValueKind::Undefined;
                        continue;
                    }
                }
                if let Some(item) =
                    self.lower_statement_list_item_after_hoisting(item, scope, placement)
                {
                    match item {
                        LoweredStatementListItemIr::Statement {
                            statement,
                            result_kind: kind,
                        } => {
                            prefix.push(statement);
                            result_kind = kind;
                        }
                        LoweredStatementListItemIr::SyncDisposableScope { .. }
                        | LoweredStatementListItemIr::AsyncDisposableScope(_) => return None,
                    }
                }
            }
            (next_registration == registrations.len()
                && self.resource_entry_state(execution)? == states.body().end())
            .then_some(())?;
            Some(())
        })();
        match execution {
            ResumableRegionProtocolIr::Generator => self.ordinary_generator_region_depth -= 1,
            ResumableRegionProtocolIr::Async => self.plain_async_resource_depth -= 1,
            ResumableRegionProtocolIr::AsyncGenerator => {
                self.mixed_async_generator_region_depth -= 1
            }
        }
        self.async_value_branch_context = previous_context;
        lowered?;
        let exit = states.exit();
        let body = BlockIr {
            statements: prefix,
            result_kind,
            lexical_environment: None,
        };
        let plan = AsyncGeneratorResourceScopeIr::new(
            states,
            capability,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        self.set_resource_phase(execution, exit);
        Some((
            StatementIr::AsyncGeneratorResourceScope(Box::new(plan)),
            result_kind,
        ))
    }

    fn lower_checked_async_generator_resource_registration(
        &mut self,
        variable: &Variable,
        source: &AsyncGeneratorResourceRegistrationSource,
        capability: &OwnedEnvBindingIr,
        scope: &mut LexicalScopeInstantiation,
        prefix: &mut Vec<StatementIr>,
    ) -> Option<()> {
        if !source.matches_variable(variable) {
            return None;
        }
        let Binding::Identifier(identifier) = variable.binding() else {
            return None;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let pending = scope.take(&name);
        let storage = self.direct_lexical_storage_name(&name, identifier.span());
        self.lower_checked_async_generator_resource_variable(
            variable, source, capability, pending, storage, prefix,
        )
    }

    fn lower_checked_async_generator_resource_variable(
        &mut self,
        variable: &Variable,
        source: &AsyncGeneratorResourceRegistrationSource,
        capability: &OwnedEnvBindingIr,
        pending: Option<PendingInitialization>,
        storage: String,
        prefix: &mut Vec<StatementIr>,
    ) -> Option<()> {
        if !source.matches_variable(variable) {
            return None;
        }
        let Binding::Identifier(identifier) = variable.binding() else {
            return None;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let execution = match super::resumable_operand::ResumableOperandProtocol::current(self)? {
            super::resumable_operand::ResumableOperandProtocol::Generator => {
                ResumableRegionProtocolIr::Generator
            }
            super::resumable_operand::ResumableOperandProtocol::Async => {
                ResumableRegionProtocolIr::Async
            }
            super::resumable_operand::ResumableOperandProtocol::Mixed => {
                ResumableRegionProtocolIr::AsyncGenerator
            }
        };
        let (staged, value) = super::resumable_operand::ResumableOperandProtocol::current(self)?
            .lower(self, variable.init()?)?;
        prefix.extend(staged);
        let value =
            retain_pattern_value(self, prefix, "async.generator.resource.initializer.", value);
        if self.resource_entry_state(execution)? != source.register_state() {
            return None;
        }
        // Runtime AddDisposableResource happens before InitializeBinding;
        // this original conversion owns only the compiler lifecycle update.
        self.invalidate_unknown_user_code_effects();
        self.static_to_string_regexp_object_bindings.remove(&name);
        let init = LoweredInitializer::evaluated(value);
        let initialized = match pending {
            Some(pending) => pending.initialize(init),
            None => InitializedBinding::without_creation(
                name.clone(),
                BindingMode::Const,
                storage,
                init.into_expr(),
            ),
        };
        let registration = self.finish_checked_async_generator_resource_registration(
            source,
            capability,
            &name,
            initialized,
        )?;
        prefix.push(StatementIr::AsyncGeneratorResourceRegistration(Box::new(
            registration,
        )));
        Some(())
    }

    pub(super) fn allocate_mixed_resource_capability(&mut self, hint: &str) -> OwnedEnvBindingIr {
        owned_pattern_binding(
            self,
            hint,
            ValueInfo {
                kind: ValueKind::Object,
                possible_kinds: KindSet::from_kind(ValueKind::Object),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            },
        )
    }

    pub(super) fn lower_mixed_classic_resource_initialization(
        &mut self,
        variables: &[Variable],
        states: &AsyncGeneratorScopedResourceSourceStates,
        capability: &OwnedEnvBindingIr,
    ) -> Option<Vec<StatementIr>> {
        if variables.len() != states.registrations().len() {
            return None;
        }
        let mut prefix = Vec::new();
        for (variable, source) in variables.iter().zip(states.registrations()) {
            let Binding::Identifier(identifier) = variable.binding() else {
                return None;
            };
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            self.lower_checked_async_generator_resource_variable(
                variable,
                source,
                capability,
                None,
                scoped_lexical_binding_storage_name(&name, identifier.span()),
                &mut prefix,
            )?;
        }
        Some(prefix)
    }

    pub(super) fn lower_mixed_case_resource_items(
        &mut self,
        items: &[StatementListItem],
        scope: &mut LexicalScopeInstantiation,
        states: &AsyncGeneratorScopedResourceSourceStates,
        capability: &OwnedEnvBindingIr,
    ) -> Option<BlockIr> {
        let mut statements = Vec::new();
        let mut result_kind = ValueKind::Undefined;
        for item in items {
            if let StatementListItem::Declaration(declaration) = item {
                let variables = match declaration.as_ref() {
                    Declaration::Lexical(
                        LexicalDeclaration::Using(list) | LexicalDeclaration::AwaitUsing(list),
                    ) => Some(list.as_ref()),
                    _ => None,
                };
                if let Some(variables) = variables {
                    let mut prefix = Vec::new();
                    for variable in variables {
                        let source = states
                            .registrations()
                            .iter()
                            .find(|source| source.matches_variable(variable))?;
                        self.lower_checked_async_generator_resource_registration(
                            variable,
                            source,
                            capability,
                            scope,
                            &mut prefix,
                        )?;
                    }
                    statements.push(StatementIr::EmptyStatementCompletion(Box::new(
                        EmptyStatementCompletionIr::new(
                            CheckedEmptyStatementCompletionSource::from_declaration(declaration),
                            StatementIr::LexicalBlock(prefix),
                        ),
                    )));
                    result_kind = ValueKind::Undefined;
                    continue;
                }
            }
            if let Some(item) = self.lower_statement_list_item_after_hoisting(
                item,
                scope,
                StatementListPlacement::Block,
            ) {
                match item {
                    LoweredStatementListItemIr::Statement {
                        statement,
                        result_kind: kind,
                    } => {
                        statements.push(statement);
                        result_kind = kind;
                    }
                    LoweredStatementListItemIr::SyncDisposableScope { .. }
                    | LoweredStatementListItemIr::AsyncDisposableScope(_) => return None,
                }
            }
        }
        Some(BlockIr {
            statements,
            result_kind,
            lexical_environment: None,
        })
    }

    pub(super) fn finish_mixed_resource_region(
        &self,
        states: &AsyncGeneratorScopedResourceSourceStates,
        range: AsyncGeneratorSourceRange,
        capability: &OwnedEnvBindingIr,
        block: BlockIr,
    ) -> Option<AsyncGeneratorLoopRegionIr> {
        if self.async_generator_entry_state()? != range.end() {
            return None;
        }
        AsyncGeneratorScopedResourceIr::checked_region(
            states,
            range,
            capability,
            block,
            &self.generated_owned_env_bindings,
        )
        .ok()
    }

    pub(super) fn finish_resumable_resource_region(
        &self,
        states: &AsyncGeneratorScopedResourceSourceStates,
        range: AsyncGeneratorSourceRange,
        capability: &OwnedEnvBindingIr,
        block: BlockIr,
    ) -> Option<ResumableRegionIr> {
        if self.resource_entry_state(states.execution())? != range.end() {
            return None;
        }
        AsyncGeneratorScopedResourceIr::checked_resumable_region(
            states,
            range,
            capability,
            block,
            &self.generated_owned_env_bindings,
        )
        .ok()
    }

    pub(super) fn resource_entry_state(&self, execution: ResumableRegionProtocolIr) -> Option<u32> {
        match execution {
            ResumableRegionProtocolIr::Generator => self.plain_generator_entry_state(),
            ResumableRegionProtocolIr::Async => self.plain_async_entry_state(),
            ResumableRegionProtocolIr::AsyncGenerator => self.async_generator_entry_state(),
        }
    }

    pub(super) fn set_resource_phase(&mut self, execution: ResumableRegionProtocolIr, state: u32) {
        match execution {
            ResumableRegionProtocolIr::Generator => {
                self.current_generator_resume_state = Some(state)
            }
            ResumableRegionProtocolIr::Async => self.current_async_resume_state = Some(state),
            ResumableRegionProtocolIr::AsyncGenerator => self.set_async_generator_phase(state),
        }
    }

    pub(super) fn lower_async_generator_for_of_resource_registration(
        &mut self,
        variable: &ForOfLoop,
        source: &AsyncGeneratorResourceRegistrationSource,
        capability: &OwnedEnvBindingIr,
        value: TypedExpr,
    ) -> Option<AsyncGeneratorResourceRegistrationIr> {
        let entry = match super::resumable_operand::ResumableOperandProtocol::current(self)? {
            super::resumable_operand::ResumableOperandProtocol::Generator => {
                self.plain_generator_entry_state()?
            }
            super::resumable_operand::ResumableOperandProtocol::Async => {
                self.plain_async_entry_state()?
            }
            super::resumable_operand::ResumableOperandProtocol::Mixed => {
                self.async_generator_entry_state()?
            }
        };
        if !source.matches_for_of(variable) || entry != source.register_state() {
            return None;
        }
        let binding = match variable.initializer() {
            IterableLoopInitializer::Using(binding)
            | IterableLoopInitializer::AwaitUsing(binding) => binding,
            _ => return None,
        };
        let Binding::Identifier(identifier) = binding else {
            return None;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        self.invalidate_unknown_user_code_effects();
        self.static_to_string_regexp_object_bindings.remove(&name);
        let initialized = InitializedBinding::without_creation(
            name.clone(),
            BindingMode::Const,
            for_of_loop_binding_storage_name(variable, &name),
            value,
        );
        self.finish_checked_async_generator_resource_registration(
            source,
            capability,
            &name,
            initialized,
        )
    }

    fn finish_checked_async_generator_resource_registration(
        &mut self,
        source: &AsyncGeneratorResourceRegistrationSource,
        capability: &OwnedEnvBindingIr,
        name: &str,
        initialized: InitializedBinding,
    ) -> Option<AsyncGeneratorResourceRegistrationIr> {
        let (binding_name, initializer) = match source.hint() {
            ResourceDisposalHintIr::Sync => {
                let resource = initialized.into_sync_disposable_resource(self);
                (resource.binding_name, resource.initializer)
            }
            ResourceDisposalHintIr::Async => {
                let resource = initialized.into_async_disposable_resource(self);
                (
                    resource.binding_name().to_string(),
                    resource.initializer().clone(),
                )
            }
        };
        let binding = self.lookup_binding(name)?;
        self.static_string_bindings.remove(&binding);
        let proof = CheckedAsyncGeneratorResourceRegistration {
            source: source.clone(),
            capability_binding: capability.clone(),
            binding_name,
            initializer,
        };
        Some(AsyncGeneratorResourceRegistrationIr::new(proof))
    }
}
