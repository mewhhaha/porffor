use super::*;
use boa_ast::operations::{var_scoped_declarations, VarScopedDeclaration};
use lila_front::{ParseDiagnosticKind, ParseOptions};

pub(super) fn compile_dynamic_script_sources(
    program: &mut ProgramIr,
    sources: Vec<DynamicScriptSource>,
    host_surface_policy: HostSurfacePolicy,
    allocations: &mut AnalysisAllocationState,
) {
    for source in sources {
        let parsed_source = match &source.kind {
            PreparedScriptKind::RealmScript => {
                lila_front::parse(source.source.clone(), ParseOptions::script()).map(|parsed| {
                    let ParsedSource::Script(parsed) = parsed else {
                        unreachable!("Script goal produces a Script")
                    };
                    parsed
                })
            }
            PreparedScriptKind::IndirectEval => lila_front::prepare_eval_source(
                source.source.clone(),
                &lila_front::EvalParseContext::Indirect,
            ),
            PreparedScriptKind::DirectEval(context) => lila_front::prepare_eval_source(
                source.source.clone(),
                &lila_front::EvalParseContext::Direct(context.parse_context()),
            ),
        };
        let parsed = match parsed_source {
            Ok(parsed) => parsed,
            Err(error) => {
                match error.diagnostic().kind() {
                    ParseDiagnosticKind::MalformedJavaScript => program
                        .script
                        .as_mut()
                        .expect("prepared source has a lowered owner")
                        .prepared_scripts
                        .push(PreparedScript {
                            admission: source.admission,
                            kind: source.kind.clone(),
                            source: source.source,
                            outcome: PreparedScriptOutcome::DeferredSyntaxError {
                                message: error.message().to_string(),
                            },
                        }),
                    ParseDiagnosticKind::UnsupportedParserFeature => {
                        program
                            .diagnostics
                            .push(IrDiagnostic::unsupported_parser_feature(error.message()));
                    }
                }
                continue;
            }
        };
        let id = allocations.allocate_static_script_id();
        let mut compiled = lower_script_program_with_allocations(
            &parsed,
            ParseGoal::Script,
            parsed.source_text.len(),
            vec![LoweringStage::ParsedSource],
            None,
            &modules::DefaultExportDefinitions::default(),
            host_surface_policy,
            allocations,
            ScriptInstantiation::Prepared(source.kind.clone()),
        );
        if !compiled.diagnostics.is_empty() {
            if compiled
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.error_type() == Some(NativeErrorKind::SyntaxError))
            {
                program
                    .script
                    .as_mut()
                    .expect("prepared source has a lowered owner")
                    .prepared_scripts
                    .push(PreparedScript {
                        admission: source.admission,
                        kind: source.kind.clone(),
                        source: source.source,
                        outcome: PreparedScriptOutcome::DeferredSyntaxError {
                            message: compiled
                                .diagnostics
                                .iter()
                                .map(|diagnostic| diagnostic.message.as_str())
                                .collect::<Vec<_>>()
                                .join("; "),
                        },
                    });
            } else {
                if source.admission == PreparedScriptAdmission::RuntimeCandidate {
                    compiled
                        .diagnostics
                        .retain(|diagnostic| !diagnostic.can_omit_for_runtime_source_candidate());
                }
                program.diagnostics.append(&mut compiled.diagnostics);
            }
            continue;
        }
        let mut compiled = compiled
            .script
            .expect("independent source lowered as Script");
        let unit = PreparedScriptUnit {
            eval_environment: compiled.eval_environment.take(),
            id,
            kind: source.kind.clone(),
            strict: compiled.strict,
            body: std::mem::replace(
                &mut compiled.body,
                BlockIr {
                    statements: Vec::new(),
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
            ),
            owned_env_bindings: std::mem::take(&mut compiled.owned_env_bindings),
            global_bindings: std::mem::take(&mut compiled.global_bindings),
            declarations: std::mem::take(&mut compiled.runtime_declarations),
            function_ids: compiled
                .functions
                .iter()
                .map(|function| function.id.clone())
                .collect(),
        };
        let script = program
            .script
            .as_mut()
            .expect("prepared source has a lowered owner");
        append_prepared_unit(script, compiled);
        script.prepared_scripts.push(PreparedScript {
            admission: source.admission,
            kind: source.kind.clone(),
            source: source.source,
            outcome: PreparedScriptOutcome::Executable(unit),
        });
    }
    record_prepared_compilation_stage(program);
}

impl ScriptLowerer<'_> {
    pub(super) fn runtime_global_declarations(
        &self,
        script: &Script,
    ) -> RuntimeGlobalDeclarationPlan {
        let lexical_names_in_source_order = lexically_declared_names(script)
            .into_iter()
            .map(|name| self.interner.resolve_expect(name).to_string())
            .collect();
        let mut function_names = BTreeSet::new();
        let functions_in_reverse_order = self
            .analysis
            .script_root_functions
            .iter()
            .rev()
            .filter(|function| function_names.insert(function.name.clone()))
            .map(|function| GlobalFunctionDeclarationIr {
                name: function.name.clone(),
                function_id: function.id.clone(),
            })
            .collect();
        let mut seen_vars = BTreeSet::new();
        let var_names = var_scoped_declarations(script)
            .into_iter()
            .filter(|declaration| {
                matches!(declaration, VarScopedDeclaration::VariableDeclaration(_))
            })
            .flat_map(|declaration| declaration.bound_names())
            .map(|name| self.interner.resolve_expect(name).to_string())
            .filter(|name| seen_vars.insert(name.clone()))
            .collect();
        let mut seen_annex_b = BTreeSet::new();
        let annex_b_candidates = annex_b_function_declarations(script)
            .into_iter()
            .filter_map(|function| {
                self.analysis
                    .annex_b_function_plans
                    .get(&function_declaration_key(function))
            })
            .filter(|plan| plan.owner_id == SCRIPT_OWNER_ID && plan.copy_to_variable_environment)
            .map(|plan| plan.source_name.clone())
            .filter(|name| seen_annex_b.insert(name.clone()))
            .filter(|_| {
                matches!(
                    self.analysis.script_instantiation,
                    ScriptInstantiation::Prepared(_)
                )
            })
            .map(|name| {
                let storage_name = annex_b_admission_binding_name(&name);
                let slot =
                    self.analysis.owner_plans[SCRIPT_OWNER_ID].owned_env_slots[&storage_name];
                AnnexBGlobalDeclarationIr {
                    name,
                    admission: OwnedEnvBindingIr {
                        name: storage_name,
                        slot,
                    },
                }
            })
            .collect();
        RuntimeGlobalDeclarationPlan {
            lexical_names_in_source_order,
            functions_in_reverse_order,
            var_names,
            annex_b_candidates,
        }
    }

    pub(super) fn script_variables_are_global(&self) -> bool {
        self.analysis
            .script_instantiation
            .has_global_variable_environment(self.analysis.owner_plans[SCRIPT_OWNER_ID].strict)
    }

    pub(super) fn root_functions_need_body_initialization(&self) -> bool {
        if self.borrows_direct_eval_variable_environment() {
            return false;
        }
        self.current_owner_id != SCRIPT_OWNER_ID || !self.script_variables_are_global()
    }
}

fn annex_b_admission_binding_name(name: &str) -> String {
    format!("$annex.b.admitted:{name}")
}

impl Analysis<'_> {
    pub(super) fn prepare_runtime_script_slots(&mut self, script: &Script, interner: &Interner) {
        if !matches!(self.script_instantiation, ScriptInstantiation::Prepared(_)) {
            return;
        }
        let mut names = lexically_declared_names(script)
            .into_iter()
            .map(|name| interner.resolve_expect(name).to_string())
            .collect::<Vec<_>>();
        names.extend(
            self.annex_b_function_plans
                .values()
                .filter(|plan| {
                    plan.owner_id == SCRIPT_OWNER_ID && plan.copy_to_variable_environment
                })
                .map(|plan| annex_b_admission_binding_name(&plan.source_name)),
        );
        let owner = self
            .owner_plans
            .get_mut(SCRIPT_OWNER_ID)
            .expect("Script activation exists");
        let environment = self
            .environment_plans
            .get_mut(&owner.activation_environment_id)
            .expect("Script activation environment exists");
        for name in names {
            if owner.owned_env_slots.contains_key(&name) {
                continue;
            }
            let slot = owner.owned_env_slots.len() as u32;
            owner.owned_env_slots.insert(name.clone(), slot);
            environment.owned_env_slots.insert(name, slot);
        }
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn prepared_annex_b_admission(&self, name: &str) -> Option<OwnedEnvBindingIr> {
        if self.current_owner_id != SCRIPT_OWNER_ID
            || !matches!(
                self.analysis.script_instantiation,
                ScriptInstantiation::Prepared(_)
            )
        {
            return None;
        }
        let name = annex_b_admission_binding_name(name);
        let slot = self.analysis.owner_plans[SCRIPT_OWNER_ID].owned_env_slots[&name];
        Some(OwnedEnvBindingIr { name, slot })
    }
}
