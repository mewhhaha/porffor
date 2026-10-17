use super::*;

impl FunctionBuilder<'_> {
    /// A prepared global Script with only bare var declarations has no frame
    /// reads or writes. Its declaration plan still creates every global binding.
    pub(super) fn source_frame_var_names(&self) -> Vec<String> {
        let FunctionModuleState::PreparedScript(unit, _) = self.module_state else {
            return self.hoisted_vars.clone();
        };
        let declaration_only = unit.has_global_variable_environment()
            && unit.body.lexical_environment.is_none()
            && unit.owned_env_bindings.is_empty()
            && unit.function_ids.is_empty()
            && unit.declarations.lexical_names_in_source_order.is_empty()
            && unit.declarations.functions_in_reverse_order.is_empty()
            && unit.declarations.annex_b_candidates.is_empty()
            && unit
                .body
                .statements
                .iter()
                .all(|statement| match statement {
                    StatementIr::Empty => true,
                    StatementIr::Var(declarations) => declarations
                        .iter()
                        .all(|declaration| declaration.init.is_none()),
                    _ => false,
                });
        if declaration_only {
            Vec::new()
        } else {
            self.hoisted_vars.clone()
        }
    }
}
