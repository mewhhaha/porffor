use super::*;
use boa_ast::visitor::{VisitWith, Visitor};
use core::ops::ControlFlow;

fn parameters_contain_expression(parameters: &[FormalParameter]) -> bool {
    struct ContainsExpression;
    impl<'ast> Visitor<'ast> for ContainsExpression {
        type BreakTy = ();
        fn visit_expression(&mut self, _: &'ast Expression) -> ControlFlow<()> {
            ControlFlow::Break(())
        }
    }
    parameters.iter().any(|parameter| {
        parameter.init().is_some()
            || matches!(parameter.variable().binding(), Binding::Pattern(pattern)
                if pattern.visit_with(&mut ContainsExpression).is_break())
    })
}

impl<'a> AnalysisBuilder<'a> {
    pub(super) fn body_environment_cursor(&self, owner_id: &str) -> EnvironmentCursor {
        let owner = &self.owner_plans[owner_id];
        EnvironmentCursor {
            owner_id: owner_id.to_string(),
            environment_id: owner
                .body_environment_id
                .unwrap_or(owner.activation_environment_id),
        }
    }

    pub(super) fn prepare_function_body_environment(
        &mut self,
        owner_id: &str,
        parameters: &[FormalParameter],
        items: &'a [StatementListItem],
        interner: &Interner,
    ) {
        if !parameters_contain_expression(parameters) {
            return;
        }
        let owner = self.owner_plans[owner_id].clone();
        let activation_id = owner.activation_environment_id;
        self.environment_plans
            .get_mut(&activation_id)
            .expect("parameter activation exists")
            .kind = EnvironmentKind::FunctionParameters;
        let parameter_bindings = self.parameter_environment_bindings[owner_id].clone();
        let mut body_bindings = owner
            .root_bindings
            .difference(&parameter_bindings)
            .cloned()
            .collect();
        self.collect_owner_root_bindings_from_items(interner, items, &mut body_bindings);
        body_bindings.extend(owner.function_bindings.keys().cloned());
        let activation_modes = self.environment_plans[&activation_id].binding_modes.clone();
        let body_id = self.alloc_environment_id();
        self.register_environment_plan(
            body_id,
            owner_id,
            EnvironmentKind::FunctionBody,
            Some(self.activation_environment_cursor(owner_id)),
            body_bindings,
        );
        self.set_environment_binding_modes(body_id, activation_modes);
        let activation = self
            .environment_plans
            .get_mut(&activation_id)
            .expect("parameter activation exists");
        let removed = activation
            .binding_storage_names
            .difference(&parameter_bindings)
            .cloned()
            .collect::<Vec<_>>();
        activation.binding_storage_names = parameter_bindings.clone();
        activation
            .binding_modes
            .retain(|name, _| parameter_bindings.contains(name));
        for name in &owner.parameter_names {
            activation
                .binding_modes
                .insert(name.clone(), BindingMode::Let);
        }
        for name in removed {
            self.physical_binding_environments
                .get_mut(&name)
                .expect("binding has a physical owner")
                .remove(&activation_id);
        }
        for name in &parameter_bindings {
            self.physical_binding_environments
                .entry(name.clone())
                .or_default()
                .insert(activation_id);
        }
        let parameter_eval_id = if self.eval_visible && !owner.strict {
            let id = self.alloc_environment_id();
            self.register_environment_plan(
                id,
                owner_id,
                EnvironmentKind::ParameterEvalVariable,
                Some(owner.definition_environment_cursor.clone()),
                BTreeSet::new(),
            );
            self.set_environment_parent_cursor(
                activation_id,
                EnvironmentCursor {
                    owner_id: owner_id.to_string(),
                    environment_id: id,
                },
            );
            Some(id)
        } else {
            None
        };
        let owner = self
            .owner_plans
            .get_mut(owner_id)
            .expect("function owner exists");
        owner.body_environment_id = Some(body_id);
        owner.parameter_eval_environment_id = parameter_eval_id;
        owner.owned_env_slots = owner
            .owned_env_slots
            .keys()
            .filter(|name| parameter_bindings.contains(*name))
            .enumerate()
            .map(|(slot, name)| (name.clone(), slot as u32))
            .collect();
    }
}
