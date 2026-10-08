use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_function_declaration(
        &mut self,
        function: &FunctionDeclaration,
    ) -> StatementIr {
        let key = function_declaration_key(function);
        let Some(function_id) = self.analysis.function_declaration_ids.get(&key).cloned() else {
            self.unsupported("function declaration");
            return StatementIr::Empty;
        };
        let name = function_name(self.interner, function, None);
        let storage_name = self
            .analysis
            .annex_b_function_plans
            .get(&key)
            .map(|plan| plan.block_storage_name.clone())
            .unwrap_or_else(|| self.direct_lexical_storage_name(&name, function.name().span()));
        let function_info = self.function_value_info(&function_id);
        self.declare_binding(
            name.clone(),
            BindingInfo {
                mode: BindingMode::Let,
                storage_name: storage_name.clone(),
                kind: function_info.kind,
                possible_kinds: function_info.possible_kinds,
                heap_shape: function_info.heap_shape.clone(),
                function_targets: function_info.function_targets.clone(),
                initialization: Initialization::Initialized,
            },
        );
        StatementIr::Lexical {
            mode: BindingMode::Let,
            name: storage_name,
            init: TypedExpr::from_info(function_info, ExprIr::FunctionValue(function_id)),
        }
    }

    pub(super) fn lower_generator_declaration(
        &mut self,
        function: &GeneratorDeclaration,
    ) -> StatementIr {
        let key = generator_declaration_key(function);
        let Some(function_id) = self.analysis.function_declaration_ids.get(&key).cloned() else {
            self.unsupported("generator declaration");
            return StatementIr::Empty;
        };
        let name = self
            .interner
            .resolve_expect(function.name().sym())
            .to_string();
        let storage_name = self.direct_lexical_storage_name(&name, function.name().span());
        let function_info = self.function_value_info(&function_id);
        self.declare_binding(
            name,
            BindingInfo {
                mode: BindingMode::Let,
                storage_name: storage_name.clone(),
                kind: function_info.kind,
                possible_kinds: function_info.possible_kinds,
                heap_shape: function_info.heap_shape.clone(),
                function_targets: function_info.function_targets.clone(),
                initialization: Initialization::Initialized,
            },
        );
        StatementIr::Lexical {
            mode: BindingMode::Let,
            name: storage_name,
            init: TypedExpr::from_info(function_info, ExprIr::FunctionValue(function_id)),
        }
    }

    pub(super) fn lower_async_function_declaration(
        &mut self,
        function: &AsyncFunctionDeclaration,
    ) -> StatementIr {
        let key = async_function_declaration_key(function);
        let Some(function_id) = self.analysis.function_declaration_ids.get(&key).cloned() else {
            self.unsupported("async function declaration");
            return StatementIr::Empty;
        };
        let name = self
            .interner
            .resolve_expect(function.name().sym())
            .to_string();
        let storage_name = self.direct_lexical_storage_name(&name, function.name().span());
        let function_info = self.function_value_info(&function_id);
        self.declare_binding(
            name,
            BindingInfo {
                mode: BindingMode::Let,
                storage_name: storage_name.clone(),
                kind: function_info.kind,
                possible_kinds: function_info.possible_kinds,
                heap_shape: function_info.heap_shape.clone(),
                function_targets: function_info.function_targets.clone(),
                initialization: Initialization::Initialized,
            },
        );
        StatementIr::Lexical {
            mode: BindingMode::Let,
            name: storage_name,
            init: TypedExpr::from_info(function_info, ExprIr::FunctionValue(function_id)),
        }
    }

    pub(super) fn lower_async_generator_declaration(
        &mut self,
        function: &AsyncGeneratorDeclaration,
    ) -> StatementIr {
        let key = async_generator_declaration_key(function);
        let Some(function_id) = self.analysis.function_declaration_ids.get(&key).cloned() else {
            self.unsupported("async generator declaration");
            return StatementIr::Empty;
        };
        let name = self
            .interner
            .resolve_expect(function.name().sym())
            .to_string();
        let storage_name = self.direct_lexical_storage_name(&name, function.name().span());
        let function_info = self.function_value_info(&function_id);
        self.declare_binding(
            name,
            BindingInfo {
                mode: BindingMode::Let,
                storage_name: storage_name.clone(),
                kind: function_info.kind,
                possible_kinds: function_info.possible_kinds,
                heap_shape: function_info.heap_shape.clone(),
                function_targets: function_info.function_targets.clone(),
                initialization: Initialization::Initialized,
            },
        );
        StatementIr::Lexical {
            mode: BindingMode::Let,
            name: storage_name,
            init: TypedExpr::from_info(function_info, ExprIr::FunctionValue(function_id)),
        }
    }

    pub(super) fn lower_annex_b_function_copy(
        &mut self,
        function: &FunctionDeclaration,
    ) -> Option<StatementIr> {
        let key = function_declaration_key(function);
        let plan = self.analysis.annex_b_function_plans.get(&key)?.clone();
        if !plan.copy_to_variable_environment {
            return None;
        }
        let source = self.lookup_binding(&plan.source_name);
        let Some(source) = source else {
            self.unsupported_with_message(format!(
                "unsupported in lila wasm-aot first slice: Annex B declaration `{}` has no active block binding `{}`",
                plan.source_name, plan.block_storage_name
            ));
            return Some(StatementIr::Empty);
        };
        if source.storage_name != plan.block_storage_name {
            self.unsupported_with_message(format!(
                "unsupported in lila wasm-aot first slice: Annex B declaration `{}` resolved block storage `{}` instead of planned `{}`",
                plan.source_name, source.storage_name, plan.block_storage_name
            ));
            return Some(StatementIr::Empty);
        }
        let value_info = ValueInfo {
            kind: source.kind,
            possible_kinds: source.possible_kinds,
            heap_shape: source.heap_shape,
            function_targets: source.function_targets,
        };
        self.set_owner_binding_value_info(&plan.source_name, value_info.clone());
        if self.current_owner_id == SCRIPT_OWNER_ID && self.script_variables_are_global() {
            self.set_global_property_value_info_with_source(
                plan.source_name.clone(),
                value_info,
                GlobalPropertySource::GlobalWrite,
            );
        }
        let admission = self.prepared_annex_b_admission(&plan.source_name);
        let copy = StatementIr::AnnexBFunctionCopy {
            admission,
            source_name: plan.source_name.clone(),
            block_storage_name: plan.block_storage_name,
            target: if self.borrows_direct_eval_variable_environment() {
                AnnexBFunctionCopyTargetIr::DirectEvalVariable {
                    name: plan.source_name,
                }
            } else if self.current_owner_id == SCRIPT_OWNER_ID && self.script_variables_are_global()
            {
                AnnexBFunctionCopyTargetIr::ScriptGlobal {
                    name: plan.source_name,
                }
            } else {
                AnnexBFunctionCopyTargetIr::OwnerBinding {
                    storage_name: plan.source_name,
                }
            },
        };
        Some(copy)
    }
}
