use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_new(&mut self, new_expr: &New) -> TypedExpr {
        let callee = self.lower_expression(new_expr.constructor());
        if callee.kind != ValueKind::Function {
            let Some(args) = self.lower_call_args_expanding_spread(new_expr.arguments()) else {
                return TypedExpr::undefined();
            };
            let function_or_uninitialized = KindSet::from_kind(ValueKind::Function)
                .union(KindSet::from_kind(ValueKind::Undefined));
            if callee
                .possible_kinds
                .is_subset_of(function_or_uninitialized)
                && callee
                    .function_targets
                    .contains(&StandardBuiltinId::BoundFunctionInvoker.function_id())
            {
                return TypedExpr::from_info(
                    Self::fresh_constructed_instance_info(),
                    ExprIr::Construct {
                        callee: Box::new(callee),
                        args,
                        static_regexp_compilation: None,
                    },
                );
            }
            return TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    // The Construct abstract operation always yields an Object (or throws);
                    // it never produces a primitive, even when the constructor target isn't
                    // statically resolved. Narrowing here (rather than `all_runtime_tags()`)
                    // keeps downstream object-like reasoning (e.g. `.valueOf()`/`.toString()`
                    // identity handling) sound for dynamically-typed `new` targets.
                    possible_kinds: Self::object_like_kind_set(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::Construct {
                    callee: Box::new(callee),
                    args,
                    static_regexp_compilation: None,
                },
            );
        }
        let Some(function_id) = self.resolve_single_function_target(&callee) else {
            let constructor_targets = callee
                .function_targets
                .iter()
                .filter_map(|function_id| StandardBuiltinId::from_function_id(function_id))
                .collect::<Vec<_>>();
            if !constructor_targets.is_empty()
                && constructor_targets
                    .iter()
                    .all(|builtin| Self::is_typed_array_constructor(*builtin))
            {
                let Some(args) = self.lower_call_args_expanding_spread(new_expr.arguments()) else {
                    return TypedExpr::undefined();
                };
                return TypedExpr::from_info(
                    Self::value_info_from_shape(Some(Self::typed_array_instance_shape())),
                    ExprIr::Construct {
                        callee: Box::new(callee),
                        args,
                        static_regexp_compilation: None,
                    },
                );
            }
            let Some(args) = self.lower_call_args_expanding_spread(new_expr.arguments()) else {
                return TypedExpr::undefined();
            };
            let dynamic_source_calls = self.resolve_constructable_dynamic_source_calls(
                &callee.function_targets,
                new_expr.arguments(),
                &args,
            );
            if !dynamic_source_calls.is_empty() {
                for (function_id, resolved) in dynamic_source_calls {
                    match resolved {
                        ResolvedDynamicSourceCall::EvalPassThrough(_) => {
                            unreachable!("the intrinsic eval function is not constructable")
                        }
                        ResolvedDynamicSourceCall::Unsupported(gap) => {
                            self.record_unsupported_dynamic_source(&function_id, gap);
                        }
                    }
                }
                return TypedExpr::undefined();
            }
            return TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    // See comment above: Construct always yields an object-like value.
                    possible_kinds: Self::object_like_kind_set(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::Construct {
                    callee: Box::new(callee),
                    args,
                    static_regexp_compilation: None,
                },
            );
        };
        let Some(signature) = self.function_signatures.get(&function_id).cloned() else {
            return self.unsupported_expr("construct");
        };
        if !signature.protocol.is_constructable()
            || signature.protocol.flavor() == FunctionFlavor::Arrow
        {
            let Some(args) = self.lower_call_args_expanding_spread(new_expr.arguments()) else {
                return TypedExpr::undefined();
            };
            return TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    // See comment above: Construct always yields an object-like value.
                    possible_kinds: Self::object_like_kind_set(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
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
            if builtin == StandardBuiltinId::ProxyConstructor {
                self.observe_proxy_handler_trap_expression_hints(new_expr.arguments());
            }
            if builtin == StandardBuiltinId::FunctionConstructor {
                return self.lower_dynamic_source_construct(&function_id, new_expr.arguments());
            }
            let args = if Self::is_typed_array_constructor(builtin) {
                let Some(args) = self.lower_call_args_expanding_spread(new_expr.arguments()) else {
                    return TypedExpr::undefined();
                };
                args
            } else {
                self.lower_call_args(&function_id, new_expr.arguments()).0
            };
            let Some(result) =
                self.standard_builtin_call_info(builtin, &args, BuiltinCallContext::Construct)
            else {
                return TypedExpr::undefined();
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
            return TypedExpr::from_info(
                result,
                ExprIr::Construct {
                    callee: Box::new(callee),
                    args,
                    static_regexp_compilation,
                },
            );
        }
        let (args, _) = self.lower_call_args(&function_id, new_expr.arguments());
        let null_heritage_return_path = signature.class_heritage_kind == ClassHeritageKind::Null
            && !signature
                .return_possible_kinds
                .is_subset_of(KindSet::PRIMITIVE_ONLY);
        let mut result = if null_heritage_return_path {
            ValueInfo {
                kind: signature.return_kind,
                possible_kinds: signature.return_possible_kinds,
                heap_shape: signature.return_shape.clone(),
                function_targets: signature.return_targets.clone(),
            }
        } else {
            signature.constructor_instance.clone()
        };
        if let Some(ObjectShapeProperty::Data(prototype_info)) =
            self.read_object_shape_property(&callee, "prototype")
        {
            if matches!(
                prototype_info.kind,
                ValueKind::Object | ValueKind::Array | ValueKind::Function | ValueKind::Arguments
            ) {
                result = Self::with_instance_prototype(result, prototype_info.heap_shape);
            }
        }
        self.merge_function_this_info(&function_id, result.clone());
        if !null_heritage_return_path
            && !signature
                .return_possible_kinds
                .is_subset_of(KindSet::PRIMITIVE_ONLY)
        {
            result = self.merge_value_infos(
                result,
                ValueInfo {
                    kind: signature.return_kind,
                    possible_kinds: signature.return_possible_kinds,
                    heap_shape: signature.return_shape,
                    function_targets: signature.return_targets,
                },
            );
        }
        TypedExpr::from_info(
            result,
            ExprIr::Construct {
                callee: Box::new(callee),
                args,
                static_regexp_compilation: None,
            },
        )
    }
}
