use super::*;
use lila_front::{prepare_dynamic_function, FunctionParseKind, ParseDiagnosticKind};

pub(super) fn compile_dynamic_function_sources(
    program: &mut ProgramIr,
    sources: Vec<DynamicFunctionSource>,
    host_surface_policy: HostSurfacePolicy,
    allocations: &mut AnalysisAllocationState,
) {
    for source in sources {
        let parser_kind = match source.kind {
            DynamicFunctionKind::Ordinary => FunctionParseKind::Ordinary,
            DynamicFunctionKind::Generator => FunctionParseKind::Generator,
            DynamicFunctionKind::Async => FunctionParseKind::Async,
            DynamicFunctionKind::AsyncGenerator => FunctionParseKind::AsyncGenerator,
        };
        let parsed = match prepare_dynamic_function(parser_kind, &source.arguments) {
            Ok(parsed) => parsed,
            Err(error) => {
                match error.diagnostic().kind() {
                    ParseDiagnosticKind::MalformedJavaScript => {
                        program
                            .script
                            .as_mut()
                            .expect("source owner was lowered")
                            .prepared_dynamic_functions
                            .push(PreparedDynamicFunction {
                                kind: source.kind,
                                arguments: source.arguments,
                                outcome: PreparedDynamicFunctionOutcome::SyntaxError {
                                    message: error.message().to_string(),
                                },
                            });
                    }
                    ParseDiagnosticKind::UnsupportedParserFeature => {
                        program
                            .diagnostics
                            .push(IrDiagnostic::unsupported_parser_feature(error.message()));
                    }
                }
                continue;
            }
        };
        let mut compiled = lower_script_program_with_allocations(
            &parsed,
            ParseGoal::Script,
            parsed.source_text.len(),
            vec![LoweringStage::ParsedSource],
            None,
            host_surface_policy,
            allocations,
            ScriptInstantiation::FreshEntry,
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
                    .expect("source owner was lowered")
                    .prepared_dynamic_functions
                    .push(PreparedDynamicFunction {
                        kind: source.kind,
                        arguments: source.arguments,
                        outcome: PreparedDynamicFunctionOutcome::SyntaxError {
                            message: compiled
                                .diagnostics
                                .iter()
                                .map(|diagnostic| diagnostic.message.as_str())
                                .collect::<Vec<_>>()
                                .join("; "),
                        },
                    });
            } else {
                if source.admission
                    == crate::prepared_function::DynamicFunctionSourceAdmission::RuntimeCandidate
                {
                    compiled
                        .diagnostics
                        .retain(|diagnostic| !diagnostic.can_omit_for_runtime_source_candidate());
                }
                program.diagnostics.append(&mut compiled.diagnostics);
            }
            continue;
        }
        let mut unit = compiled
            .script
            .expect("independent function source lowered as Script");
        if !unit.owned_env_bindings.is_empty()
            || unit.global_bindings.lexical_names().len() != 0
            || unit
                .global_bindings
                .iter()
                .any(|binding| binding.declarations != GlobalDeclarationSetIr::None)
        {
            program.diagnostics.push(IrDiagnostic::lowering(
                "prepared function expression unexpectedly owns Script declarations",
            ));
            continue;
        }
        let function_id = match unit.body.statements.last() {
            Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::FunctionValue(function_id),
                ..
            })) => function_id.clone(),
            _ => {
                program.diagnostics.push(IrDiagnostic::lowering(
                    "prepared function source did not lower to a function expression",
                ));
                continue;
            }
        };
        let function = unit
            .functions
            .iter_mut()
            .find(|function| function.id == function_id)
            .expect("prepared function expression owns its lowered function");
        if !function.captured_bindings.is_empty() {
            program.diagnostics.push(IrDiagnostic::lowering(
                "prepared dynamic function captured an independent Script activation",
            ));
            continue;
        }
        function.name = "anonymous".to_string();
        function.to_string_representation =
            CallableToStringRepresentation::ExactSource(parser_kind.source_text(&source.arguments));
        let script = program.script.as_mut().expect("source owner was lowered");
        append_prepared_unit(script, unit);
        script
            .prepared_dynamic_functions
            .push(PreparedDynamicFunction {
                kind: source.kind,
                arguments: source.arguments,
                outcome: PreparedDynamicFunctionOutcome::Compiled { function_id },
            });
    }
    record_prepared_compilation_stage(program);
}

pub(super) fn record_prepared_compilation_stage(program: &mut ProgramIr) {
    if !program.diagnostics.is_empty() {
        program
            .stages
            .retain(|stage| *stage != LoweringStage::WasmReady);
        if !program
            .stages
            .contains(&LoweringStage::UnsupportedFeaturesRecorded)
        {
            program
                .stages
                .push(LoweringStage::UnsupportedFeaturesRecorded);
        }
    }
}

pub(super) fn append_prepared_unit(script: &mut ScriptIr, mut unit: ScriptIr) {
    script.functions.append(&mut unit.functions);
    script
        .prepared_dynamic_functions
        .append(&mut unit.prepared_dynamic_functions);
    for builtin in unit.host_builtins {
        if !script.host_builtins.contains(&builtin) {
            script.host_builtins.push(builtin);
        }
    }
    script.builtin_ctor_calls += unit.builtin_ctor_calls;
    script.builtin_static_calls += unit.builtin_static_calls;
    script.error_builtin_calls += unit.error_builtin_calls;
    script.aggregate_errors += unit.aggregate_errors;
    script.function_proto_calls += unit.function_proto_calls;
    script.function_proto_applies += unit.function_proto_applies;
    script.function_proto_binds += unit.function_proto_binds;
    script.function_proto_to_strings += unit.function_proto_to_strings;
    script.bound_functions += unit.bound_functions;
    script.bound_function_constructs += unit.bound_function_constructs;
    script.boxed_builtin_calls += unit.boxed_builtin_calls;
    script.boxed_builtin_constructs += unit.boxed_builtin_constructs;
    script.boxed_receiver_adaptations += unit.boxed_receiver_adaptations;
    script.top_level_this_uses += unit.top_level_this_uses;
    script.host_builtin_calls += unit.host_builtin_calls;
    script.error_proto_to_strings += unit.error_proto_to_strings;
    script.prepared_scripts.append(&mut unit.prepared_scripts);
}

impl ScriptLowerer<'_> {
    pub(super) fn merge_child_compilation_records(&mut self, child: &mut ScriptLowerer<'_>) {
        self.diagnostics.append(&mut child.diagnostics);
        self.generated_functions
            .append(&mut child.generated_functions);
        for source in child.dynamic_script_sources.drain(..) {
            self.register_dynamic_script_source(source);
        }
        for source in child.dynamic_function_sources.drain(..) {
            self.register_dynamic_function_source(source);
        }
    }
}
