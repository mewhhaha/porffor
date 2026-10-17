use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn borrows_direct_eval_variable_environment(&self) -> bool {
        self.current_owner_id == SCRIPT_OWNER_ID
            && !self.analysis.owner_plans[SCRIPT_OWNER_ID].strict
            && matches!(
                &self.analysis.script_instantiation,
                ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(_))
            )
    }

    pub(super) fn direct_eval_invocation(&self) -> Option<lila_front::EvalInvocationContext> {
        if self.current_owner_id != SCRIPT_OWNER_ID
            && !self
                .analysis
                .function_plans
                .get(&self.current_owner_id)
                .is_some_and(|function| {
                    function
                        .captures
                        .contains_key(DIRECT_EVAL_EXECUTION_CONTEXT_NAME)
                })
        {
            return None;
        }
        match &self.analysis.script_instantiation {
            ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(context)) => {
                Some(context.invocation())
            }
            _ => None,
        }
    }

    pub(super) fn direct_eval_context(&self) -> DirectEvalContextIr {
        let mut private_names = BTreeMap::new();
        let mut environment_id = self.private_environment_id;
        while let Some(id) = environment_id {
            let environment = &self.analysis.private_environment_plans[&id];
            for (name, private_name) in &environment.bindings {
                private_names.entry(name.clone()).or_insert(*private_name);
            }
            environment_id = environment.parent;
        }
        let mut owner_id = Some(self.current_owner_id.as_str());
        let mut derived_constructor_owner = None;
        while let Some(id) = owner_id {
            let owner = &self.analysis.owner_plans[id];
            if owner.lexical_super_owner_role == LexicalSuperOwnerRole::DerivedConstructorActivation
            {
                derived_constructor_owner =
                    Some(crate::DirectEvalDerivedOwner::LocalFunction(id.to_string()));
                break;
            }
            if id == SCRIPT_OWNER_ID {
                if let ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(context)) =
                    &self.analysis.script_instantiation
                {
                    derived_constructor_owner = context.derived_owner().cloned();
                }
                break;
            }
            if owner.flavor != FunctionFlavor::Arrow {
                break;
            }
            owner_id = owner.parent_owner_id.as_deref();
        }
        let mut origin = self.analysis.allocations.script_import_origin;
        let mut owner_id = Some(self.current_owner_id.as_str());
        while let Some(id) = owner_id {
            if self.analysis.function_plans.get(id).is_some_and(|plan| {
                matches!(
                    plan.protocol,
                    FunctionProtocolIr::ModuleActivation
                        | FunctionProtocolIr::AsyncModuleActivation
                )
            }) {
                // A linked module's body and every closure it defines retain
                // that Module Record, never the surrounding entry Script.
                origin = None;
                break;
            }
            owner_id = self.analysis.owner_plans[id].parent_owner_id.as_deref();
        }
        DirectEvalContextIr::new(
            self.reference_strictness(),
            self.analysis.owner_plans[&self.current_owner_id].eval_invocation,
            private_names,
            None,
        )
        .with_derived_owner(derived_constructor_owner)
        .with_script_import_origin(origin)
    }

    pub(super) fn prepared_literal_import_referrer(
        &mut self,
        call: &boa_ast::expression::ImportCall,
    ) -> crate::ImportCallSourceOriginIr {
        use crate::direct_eval_context::ScriptImportOrigin;
        use crate::ImportCallSourceOriginIr;
        let ScriptInstantiation::Prepared(kind) = &self.analysis.script_instantiation else {
            return ImportCallSourceOriginIr::Local(None);
        };
        if let PreparedScriptKind::DirectEval(context) = kind {
            if let Some(ScriptImportOrigin::ExternalCaller(origin)) = context.script_import_origin()
            {
                return ImportCallSourceOriginIr::ExternalCaller(origin);
            }
        }
        let referrer = match kind {
            PreparedScriptKind::DirectEval(context)
                if matches!(Self::unwrap_parenthesized_expr(call.argument()), Expression::Literal(literal)
                    if matches!(literal.kind(), LiteralKind::String(_))) =>
            {
                context
                    .script_import_origin()
                    .and_then(|origin| origin.referrer())
            }
            PreparedScriptKind::DirectEval(_)
            | PreparedScriptKind::IndirectEval
            | PreparedScriptKind::RealmScript
            | PreparedScriptKind::ShadowRealmEvaluate => None,
        };
        if referrer.is_none() {
            let kind = match kind {
                PreparedScriptKind::DirectEval(_) => DynamicSourceKind::DirectEval,
                PreparedScriptKind::IndirectEval => DynamicSourceKind::IndirectEval,
                PreparedScriptKind::RealmScript => DynamicSourceKind::RealmEvalScript,
                PreparedScriptKind::ShadowRealmEvaluate => DynamicSourceKind::ShadowRealmEvaluate,
            };
            // An unbound or computed request cannot claim an entry base. An
            // omitted runtime candidate still reaches the existing typed AOT
            // source rejection if its String is actually evaluated.
            self.diagnostics
                .push(IrDiagnostic::unsupported_dynamic_source(
                    DynamicSourceGap::aot_known_source(kind),
                ));
        }
        ImportCallSourceOriginIr::Local(referrer)
    }

    pub(super) fn register_direct_eval_source(
        &mut self,
        context: &DirectEvalContextIr,
        arguments: &[Expression],
    ) {
        let Some(sources) = self.finite_script_argument_text_candidates(arguments) else {
            return;
        };
        for source in sources {
            self.register_dynamic_script_source(DynamicScriptSource {
                admission: PreparedScriptAdmission::RuntimeCandidate,
                kind: PreparedScriptKind::DirectEval(context.clone()),
                source: source.clone(),
            });
            self.register_dynamic_script_source(DynamicScriptSource {
                admission: PreparedScriptAdmission::RuntimeCandidate,
                kind: PreparedScriptKind::IndirectEval,
                source,
            });
        }
    }
}
