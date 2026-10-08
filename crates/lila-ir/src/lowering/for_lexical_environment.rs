//! Actual classic For lexical environment and per-iteration cell ownership.
use super::*;
use crate::generator_loop_control::visit_lexical_head_statement_bindings;

impl ScriptLowerer<'_> {
    /// Both activation retention and per-iteration cloning consume the actual
    /// lowered declaration bindings, including every nested pattern leaf.
    pub(super) fn visit_for_head_lexical_bindings(
        init: &ForInitIr,
        visit: &mut impl FnMut(BindingMode, &str),
    ) {
        match init {
            ForInitIr::Lexical { mode, name, .. } => visit(*mode, name),
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    visit(binding.mode, &binding.name);
                }
            }
            ForInitIr::Statements(statements) => {
                visit_lexical_head_statement_bindings(statements, visit);
            }
            ForInitIr::Var(_)
            | ForInitIr::Expression(_)
            | ForInitIr::SyncDisposable(_)
            | ForInitIr::AsyncDisposable(_) => {}
        }
    }

    pub(super) fn lower_for_lexical_environment(
        &self,
        for_loop: &ForLoop,
        init: Option<&ForInitIr>,
    ) -> Option<ForLexicalEnvironmentIr> {
        let environment_id = self
            .analysis
            .for_lexical_environment_ids
            .get(&(for_loop as *const ForLoop as usize))
            .copied()?;
        let environment = self.analysis.materialized_environment(environment_id)?;
        if !self.analysis.environment_has_runtime_storage(environment) {
            return None;
        }
        let mut per_iteration_names = BTreeSet::new();
        if let Some(init) = init {
            Self::visit_for_head_lexical_bindings(init, &mut |mode, name| {
                if mode == BindingMode::Let {
                    per_iteration_names.insert(name.to_string());
                }
            });
        }
        Some(ForLexicalEnvironmentIr {
            eval_environment: environment.eval_environment.clone(),
            bindings: environment
                .owned_env_slots
                .iter()
                .map(|(name, slot)| OwnedEnvBindingIr {
                    name: name.clone(),
                    slot: *slot,
                })
                .collect(),
            per_iteration_slots: environment
                .owned_env_slots
                .iter()
                .filter_map(|(name, slot)| per_iteration_names.contains(name).then_some(*slot))
                .collect(),
        })
    }
}
