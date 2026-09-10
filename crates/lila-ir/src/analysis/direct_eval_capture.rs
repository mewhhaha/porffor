use super::*;

impl AnalysisBuilder<'_> {
    pub(super) fn prepare_direct_eval_context_captures(
        &mut self,
        owned_names: &mut BTreeMap<EnvironmentId, BTreeSet<String>>,
    ) {
        if !matches!(
            &self.script_instantiation,
            ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(_))
        ) {
            return;
        }
        let name = DIRECT_EVAL_EXECUTION_CONTEXT_NAME.to_string();
        let root = self
            .owner_plans
            .get_mut(SCRIPT_OWNER_ID)
            .expect("direct Script owns an activation");
        root.root_bindings.insert(name.clone());
        let environment_id = root.activation_environment_id;
        let environment = self
            .environment_plans
            .get_mut(&environment_id)
            .expect("direct Script activation is planned");
        environment.binding_storage_names.insert(name.clone());
        environment
            .binding_modes
            .insert(name.clone(), BindingMode::Const);
        self.physical_binding_environments
            .entry(name.clone())
            .or_default()
            .insert(environment_id);
        owned_names
            .entry(environment_id)
            .or_default()
            .insert(name.clone());
        for function_id in self.function_order.clone() {
            let mut owner_id = function_id.as_str();
            let inherits_caller = loop {
                if owner_id == SCRIPT_OWNER_ID {
                    break true;
                }
                let owner = &self.owner_plans[owner_id];
                if owner.flavor != FunctionFlavor::Arrow {
                    break false;
                }
                owner_id = owner
                    .parent_owner_id
                    .as_deref()
                    .expect("an arrow has a lexical owner");
            };
            if inherits_caller {
                self.function_free_refs
                    .entry(function_id)
                    .or_default()
                    .insert(name.clone(), name.clone());
            }
        }
    }
}
