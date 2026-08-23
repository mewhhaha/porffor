use super::*;

struct LoweredCatchClause {
    source_name: String,
    storage_name: String,
    parameter_environment: Option<LexicalEnvironmentIr>,
    block: BlockIr,
    generator_entry_state: Option<u32>,
    generator_exit_state: Option<u32>,
    async_entry_state: Option<u32>,
    async_exit_state: Option<u32>,
}

struct LoweredFinallyClause {
    block: BlockIr,
    generator_entry_state: Option<u32>,
    generator_exit_state: Option<u32>,
    async_entry_state: Option<u32>,
    async_exit_state: Option<u32>,
}

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_try(&mut self, try_statement: &AstTry) -> (StatementIr, ValueKind) {
        let generator_entry_state = self.current_generator_resume_state;
        let async_entry_state = self.current_async_resume_state;
        let uses_preplanned_resumable_states = self.current_resumable_plan.is_some();
        // 14.15.2: the try Block's Environment Record is the Block's own, and
        // `lower_block` owns it (see `LexicalScopeInstantiation`).
        let try_block = self.lower_block(try_statement.block());
        if !uses_preplanned_resumable_states {
            if let Some(state) = self.current_generator_resume_state.as_mut() {
                *state += 1;
            }
            if let Some(state) = self.current_async_resume_state.as_mut() {
                *state += 1;
            }
        }
        let generator_try_exit_state = self.current_generator_resume_state;
        let async_try_exit_state = self.current_async_resume_state;

        let catch_parts = if let Some(catch) = try_statement.catch() {
            let generator_catch_entry_state = self.current_generator_resume_state;
            let async_catch_entry_state = self.current_async_resume_state;
            // 14.15.3 step 2's `catchEnv` — the record that holds the catch
            // *parameter*, and a different Environment Record from the catch
            // Block's, which `lower_block` pushes for itself. This push stays.
            self.push_direct_lexical_scope();
            let catch_parameter_environment = self.lower_materialized_lexical_environment(
                catch
                    .parameter()
                    .and_then(|parameter| {
                        self.analysis
                            .catch_parameter_environment_ids
                            .get(&(parameter as *const Binding as usize))
                    })
                    .copied(),
            );
            let mut catch_info = self.infer_catch_binding_info(&try_block);
            let (catch_source_name, catch_storage_name, mut catch_prefix) = match catch.parameter()
            {
                Some(Binding::Identifier(identifier)) => {
                    let source_name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let storage_name =
                        self.direct_lexical_storage_name(&source_name, identifier.span());
                    catch_info.storage_name = storage_name.clone();
                    self.declare_binding(source_name.clone(), catch_info);
                    (source_name, storage_name, Vec::new())
                }
                Some(Binding::Pattern(pattern)) => {
                    let storage_name = self.alloc_temp_binding_name("catch.internal.");
                    catch_info.storage_name = storage_name.clone();
                    let catch_value = TypedExpr::from_info(
                        ValueInfo {
                            kind: catch_info.kind,
                            possible_kinds: catch_info.possible_kinds,
                            heap_shape: catch_info.heap_shape.clone(),
                            function_targets: catch_info.function_targets.clone(),
                        },
                        ExprIr::Identifier(storage_name.clone()),
                    );
                    self.declare_binding(storage_name.clone(), catch_info);
                    let Some(prefix) = self.lower_pattern_lexical_binding_from_value(
                        BindingMode::Let,
                        pattern,
                        catch_value,
                    ) else {
                        self.pop_scope();
                        return (StatementIr::Empty, ValueKind::Undefined);
                    };
                    (storage_name.clone(), storage_name, prefix)
                }
                None => {
                    let storage_name = self.alloc_temp_binding_name("catch.internal.");
                    catch_info.storage_name = storage_name.clone();
                    self.declare_binding(storage_name.clone(), catch_info);
                    (storage_name.clone(), storage_name, Vec::new())
                }
            };
            let mut catch_block = self.lower_block(catch.block());
            if !catch_prefix.is_empty() {
                catch_prefix.append(&mut catch_block.statements);
                catch_block.statements = catch_prefix;
            }
            self.pop_scope();
            if !uses_preplanned_resumable_states {
                if let Some(state) = self.current_generator_resume_state.as_mut() {
                    *state += 1;
                }
                if let Some(state) = self.current_async_resume_state.as_mut() {
                    *state += 1;
                }
            }
            Some(LoweredCatchClause {
                source_name: catch_source_name,
                storage_name: catch_storage_name,
                parameter_environment: catch_parameter_environment,
                block: catch_block,
                generator_entry_state: generator_catch_entry_state,
                generator_exit_state: self.current_generator_resume_state,
                async_entry_state: async_catch_entry_state,
                async_exit_state: self.current_async_resume_state,
            })
        } else {
            None
        };

        let finally_block = if let Some(finally_block) = try_statement.finally() {
            let generator_finally_entry_state = self.current_generator_resume_state;
            let async_finally_entry_state = self.current_async_resume_state;
            // As for the try Block: `lower_block` owns the frame.
            let lowered = self.lower_block(finally_block.block());
            if !uses_preplanned_resumable_states {
                if let Some(state) = self.current_generator_resume_state.as_mut() {
                    *state += 1;
                }
                if let Some(state) = self.current_async_resume_state.as_mut() {
                    *state += 1;
                }
            }
            Some(LoweredFinallyClause {
                block: lowered,
                generator_entry_state: generator_finally_entry_state,
                generator_exit_state: self.current_generator_resume_state,
                async_entry_state: async_finally_entry_state,
                async_exit_state: self.current_async_resume_state,
            })
        } else {
            None
        };

        let generator_plan = generator_entry_state.map(|entry_state| {
            let (catch_entry_state, catch_exit_state) = catch_parts
                .as_ref()
                .map(|catch| (catch.generator_entry_state, catch.generator_exit_state))
                .unwrap_or((None, None));
            let (finally_entry_state, finally_exit_state) = finally_block
                .as_ref()
                .map(|finally| (finally.generator_entry_state, finally.generator_exit_state))
                .unwrap_or((None, None));
            GeneratorTryPlanIr {
                entry_state,
                try_exit_state: generator_try_exit_state.unwrap_or(entry_state),
                catch_entry_state,
                catch_exit_state,
                finally_entry_state,
                finally_exit_state,
                exit_state: self.current_generator_resume_state.unwrap_or(entry_state),
            }
        });

        let async_plan = async_entry_state.map(|entry_state| {
            let (catch_entry_state, catch_exit_state) = catch_parts
                .as_ref()
                .map(|catch| (catch.async_entry_state, catch.async_exit_state))
                .unwrap_or((None, None));
            let (finally_entry_state, finally_exit_state) = finally_block
                .as_ref()
                .map(|finally| (finally.async_entry_state, finally.async_exit_state))
                .unwrap_or((None, None));
            AsyncTryPlanIr {
                entry_state,
                try_exit_state: async_try_exit_state.unwrap_or(entry_state),
                catch_entry_state,
                catch_exit_state,
                finally_entry_state,
                finally_exit_state,
                exit_state: self.current_async_resume_state.unwrap_or(entry_state),
            }
        });

        match (catch_parts, finally_block) {
            (Some(catch), Some(finally)) => {
                let result_kind = self.merge_value_kinds(
                    self.merge_value_kinds(try_block.result_kind, catch.block.result_kind),
                    finally.block.result_kind,
                );
                (
                    StatementIr::TryCatchFinally {
                        try_block,
                        catch_name: catch.storage_name,
                        catch_source_name: catch.source_name,
                        catch_parameter_environment: catch.parameter_environment,
                        catch_block: catch.block,
                        finally_block: finally.block,
                        generator_plan,
                        async_plan,
                    },
                    result_kind,
                )
            }
            (Some(catch), None) => {
                let result_kind =
                    self.merge_value_kinds(try_block.result_kind, catch.block.result_kind);
                (
                    StatementIr::TryCatch {
                        try_block,
                        catch_name: catch.storage_name,
                        catch_source_name: catch.source_name,
                        catch_parameter_environment: catch.parameter_environment,
                        catch_block: catch.block,
                        generator_plan,
                        async_plan,
                    },
                    result_kind,
                )
            }
            (None, Some(finally)) => {
                let result_kind =
                    self.merge_value_kinds(try_block.result_kind, finally.block.result_kind);
                (
                    StatementIr::TryFinally {
                        try_block,
                        finally_block: finally.block,
                        generator_plan,
                        async_plan,
                    },
                    result_kind,
                )
            }
            (None, None) => {
                self.unsupported("control-flow or non-expression statement");
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    fn infer_catch_binding_info(&self, try_block: &BlockIr) -> BindingInfo {
        let info = self.infer_block_throw_info(try_block).unwrap_or(ValueInfo {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::all_runtime_tags(),
            heap_shape: None,
            function_targets: BTreeSet::new(),
        });
        BindingInfo {
            mode: BindingMode::Let,
            storage_name: String::new(),
            kind: info.kind,
            possible_kinds: info.possible_kinds,
            heap_shape: info.heap_shape,
            function_targets: info.function_targets,
            initialization: Initialization::Initialized,
        }
    }
}
