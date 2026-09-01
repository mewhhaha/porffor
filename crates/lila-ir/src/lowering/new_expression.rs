use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_new(&mut self, new_expr: &New) -> TypedExpr {
        let mut callee = self.lower_expression(new_expr.constructor());
        if callee
            .function_targets
            .known_targets()
            .contains(&StandardBuiltinId::ProxyConstructor.function_id())
        {
            self.observe_proxy_handler_trap_expression_hints(new_expr.arguments());
        }
        let function_id = self.resolve_single_function_target(&callee);
        if let Some(function_id) = function_id {
            let Some(signature) = self.function_signatures.get(&function_id).cloned() else {
                return self.unsupported_expr("construct");
            };
            if !signature.protocol.is_constructable()
                || signature.protocol.flavor() == FunctionFlavor::Arrow
            {
                let args = self
                    .lower_call_args_expanding_spread(new_expr.arguments())
                    .into_arguments_after_expression(&mut callee);
                return TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        // Construct always yields an object-like value or throws.
                        possible_kinds: Self::object_like_kind_set(),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::unknown(),
                    },
                    ExprIr::Construct {
                        callee: Box::new(callee),
                        args,
                        static_regexp_compilation: None,
                    },
                );
            }
            if StandardBuiltinId::from_function_id(&function_id).is_none()
                && DynamicSourceIntrinsic::from_function_id(&function_id).is_some()
            {
                return self.lower_dynamic_source_construct(&function_id, new_expr.arguments());
            }
            if let Some(builtin) = StandardBuiltinId::from_function_id(&function_id) {
                if builtin == StandardBuiltinId::FunctionConstructor {
                    return self.lower_dynamic_source_construct(&function_id, new_expr.arguments());
                }
                let (args, result, invocation_effects) =
                    if Self::is_typed_array_constructor(builtin) {
                        let args = self
                            .lower_call_args_expanding_spread(new_expr.arguments())
                            .into_arguments_after_expression(&mut callee);
                        let Some(result) = self.standard_builtin_call_info(
                            builtin,
                            &args,
                            BuiltinCallContext::Construct,
                        ) else {
                            return TypedExpr::undefined();
                        };
                        let (result, invocation_effects) = result.into_parts();
                        (args, result, invocation_effects)
                    } else {
                        let (_, args, result, invocation_effects) = self
                            .lower_call_args_with_target(
                                &function_id,
                                new_expr.arguments(),
                                BuiltinCallContext::Construct,
                                InvocationThisObservation::ConstructorCallee(&mut callee),
                            );
                        (args, result, invocation_effects)
                    };
                let static_regexp_compilation = if builtin == StandardBuiltinId::RegExpConstructor {
                    let compilation = match args.as_slice() {
                        [TypedExpr {
                            expr: ExprIr::String(pattern),
                            ..
                        }] => Some(RegExpProgram::compile(pattern, "")),
                        [TypedExpr {
                            expr: ExprIr::String(pattern),
                            ..
                        }, TypedExpr {
                            expr: ExprIr::String(flags),
                            ..
                        }, ..] => Some(RegExpProgram::compile(pattern, flags)),
                        _ => None,
                    };
                    // See the note at the sibling site in
                    // `static_regexp_compilation_for_direct_call`: exhaustive on
                    // `RegExpCompileErrorKind` so a third variant is a compile
                    // error, not a silent "unsupported". Behaviour unchanged.
                    match compilation {
                        Some(Ok(program)) => Some(StaticRegExpCompilation::Program(program)),
                        Some(Err(error)) => match error.kind {
                            RegExpCompileErrorKind::InvalidSyntax => {
                                Some(StaticRegExpCompilation::InvalidSyntax {
                                    message: format!("invalid regular-expression pattern: {error}"),
                                })
                            }
                            RegExpCompileErrorKind::UnsupportedFeature => None,
                        },
                        None => None,
                    }
                } else {
                    None
                };
                let construct = TypedExpr::from_info(
                    result,
                    ExprIr::Construct {
                        callee: Box::new(callee),
                        args,
                        static_regexp_compilation,
                    },
                );
                return invocation_effects.attach_to_emitted_call(construct);
            }
        }

        let args = self
            .lower_call_args_expanding_spread(new_expr.arguments())
            .into_arguments_after_expression(&mut callee);
        let analysis = self.analyze_known_construct_candidates(
            &callee.value_info(),
            &args,
            new_expr.arguments(),
        );
        let CallCandidateAnalysis::Accepted { result, effects } = analysis else {
            return TypedExpr::undefined();
        };
        let construct = TypedExpr::from_info(
            result,
            ExprIr::Construct {
                callee: Box::new(callee),
                args,
                static_regexp_compilation: None,
            },
        );
        effects.attach_to_emitted_call(construct)
    }
}
