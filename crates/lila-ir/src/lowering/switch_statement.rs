use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_switch(&mut self, switch: &AstSwitch) -> (StatementIr, ValueKind) {
        let discriminant = self.lower_expression(switch.val());
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.breakable_depth += 1;
        // 14.12.4 CaseBlockEvaluation pushes the CaseBlock's Environment Record
        // and instantiates its LexicallyScopedDeclarations once for the whole
        // block, not per case, so one token map is shared by every case body
        // below. The push is the constructor's; the pop is `scope.finish`.
        let mut scope = LexicalScopeInstantiation::instantiate_switch(self, switch);

        let mut last_function_by_name = BTreeMap::new();
        for case in switch.cases() {
            for item in case.body().statements() {
                let Some(function) = statement_list_item_function_declaration(item) else {
                    continue;
                };
                last_function_by_name
                    .insert(function_name(self.interner, function, None), function);
            }
        }
        let lexical_declarations = last_function_by_name
            .into_values()
            .map(|function| self.lower_function_declaration(function))
            .collect::<Vec<_>>();

        let case_conditions = switch
            .cases()
            .iter()
            .map(|case| {
                self.var_bindings = before_vars.clone();
                self.global_properties = before_globals.clone();
                (
                    case.condition().map(|expr| self.lower_expression(expr)),
                    self.var_bindings.clone(),
                    self.global_properties.clone(),
                )
            })
            .collect::<Vec<_>>();

        let mut cases = Vec::with_capacity(switch.cases().len());
        let mut result_kind: Option<ValueKind> = None;
        let mut merged_vars = before_vars.clone();
        let mut merged_globals = before_globals.clone();

        for (case, (condition, case_vars, case_globals)) in
            switch.cases().iter().zip(case_conditions.into_iter())
        {
            self.var_bindings = case_vars;
            self.global_properties = case_globals;
            let body = self.lower_statement_items_without_function_initialization(
                case.body().statements(),
                &mut scope,
            );
            merged_vars = self.merge_var_bindings(&merged_vars, &self.var_bindings);
            merged_globals = self.merge_global_properties(&merged_globals, &self.global_properties);
            if let Some(kind) = result_kind {
                if kind != body.result_kind {
                    result_kind = Some(ValueKind::Undefined);
                }
            } else {
                result_kind = Some(body.result_kind);
            }
            cases.push(SwitchCaseIr { condition, body });
        }

        self.breakable_depth -= 1;
        scope.finish(self);
        self.var_bindings = merged_vars;
        self.global_properties = merged_globals;

        (
            StatementIr::Switch {
                discriminant,
                lexical_environment: self.lower_materialized_lexical_environment(
                    self.analysis
                        .switch_environment_ids
                        .get(&(switch as *const AstSwitch as usize))
                        .copied(),
                ),
                lexical_declarations,
                cases,
            },
            result_kind.unwrap_or(ValueKind::Undefined),
        )
    }
}
