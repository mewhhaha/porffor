use super::*;

/// The closed call-site contexts observed by standard-builtin analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuiltinCallContext {
    Call,
    DirectEval,
    Construct,
    RegExpLiteral,
}

/// Direct eval requires both the intrinsic target and the direct-reference
/// syntax. Every other resolved `%eval%` call is indirect.
pub(super) fn resolved_builtin_call_context(
    callee: &TypedExpr,
    function_id: &FunctionId,
) -> BuiltinCallContext {
    if StandardBuiltinId::from_function_id(function_id) == Some(StandardBuiltinId::EvalFunction)
        && matches!(
            &callee.expr,
            ExprIr::GlobalPropertyRead { name } if name == "eval"
        )
    {
        BuiltinCallContext::DirectEval
    } else {
        BuiltinCallContext::Call
    }
}

pub(super) fn dynamic_source_kind_for_function_id(
    function_id: &str,
    context: BuiltinCallContext,
) -> Option<DynamicSourceKind> {
    if StandardBuiltinId::from_function_id(function_id) == Some(StandardBuiltinId::EvalFunction) {
        return Some(match context {
            BuiltinCallContext::DirectEval => DynamicSourceKind::DirectEval,
            BuiltinCallContext::Call
            | BuiltinCallContext::Construct
            | BuiltinCallContext::RegExpLiteral => DynamicSourceKind::IndirectEval,
        });
    }

    DynamicSourceIntrinsic::from_function_id(function_id).map(DynamicSourceIntrinsic::source_kind)
}

pub(super) fn gap_for_lowered_args(
    kind: DynamicSourceKind,
    args: &[TypedExpr],
) -> DynamicSourceGap {
    let source_is_known = match kind {
        DynamicSourceKind::DirectEval
        | DynamicSourceKind::IndirectEval
        | DynamicSourceKind::RealmEvalScript => {
            matches!(
                args.first(),
                Some(TypedExpr {
                    expr: ExprIr::String(_),
                    ..
                })
            )
        }
        DynamicSourceKind::Function(
            DynamicFunctionKind::Ordinary
            | DynamicFunctionKind::Generator
            | DynamicFunctionKind::Async
            | DynamicFunctionKind::AsyncGenerator,
        ) => args.iter().all(|arg| matches!(arg.expr, ExprIr::String(_))),
    };
    gap_for_source_proof(kind, source_is_known)
}

pub(super) fn gap_for_optional_eval(optional: &Optional) -> DynamicSourceGap {
    let source_is_known = optional.chain().first().is_some_and(|operation| {
        let OptionalOperationKind::Call { args } = operation.kind() else {
            return false;
        };
        args.first().is_some_and(|arg| {
            matches!(
                ScriptLowerer::unwrap_parenthesized_expr(arg),
                Expression::Literal(literal) if matches!(literal.kind(), LiteralKind::String(_))
            )
        })
    });
    gap_for_source_proof(DynamicSourceKind::IndirectEval, source_is_known)
}

pub(super) const fn gap_for_source_proof(
    kind: DynamicSourceKind,
    source_is_known: bool,
) -> DynamicSourceGap {
    if source_is_known {
        DynamicSourceGap::aot_known_source(kind)
    } else {
        DynamicSourceGap::runtime_source(kind)
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn register_dynamic_source_intrinsic_signatures(&mut self) {
        for intrinsic in DynamicSourceIntrinsic::ALL
            .iter()
            .copied()
            .filter(|intrinsic| {
                !matches!(
                    intrinsic,
                    DynamicSourceIntrinsic::Function(DynamicFunctionKind::Ordinary)
                )
            })
        {
            self.function_signatures.insert(
                intrinsic.function_id().to_string(),
                self.dynamic_source_intrinsic_signature(intrinsic),
            );
        }
    }

    pub(super) fn dynamic_source_intrinsic_signature(
        &self,
        intrinsic: DynamicSourceIntrinsic,
    ) -> FunctionSignature {
        let return_info = match intrinsic {
            DynamicSourceIntrinsic::Function(
                DynamicFunctionKind::Ordinary
                | DynamicFunctionKind::Generator
                | DynamicFunctionKind::Async
                | DynamicFunctionKind::AsyncGenerator,
            ) => ValueInfo {
                kind: ValueKind::Function,
                possible_kinds: KindSet::from_kind(ValueKind::Function),
                heap_shape: Some(Self::function_heap_shape(false)),
                function_targets: BTreeSet::new(),
            },
            DynamicSourceIntrinsic::RealmEvalScript => ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
        };
        FunctionSignature {
            id: intrinsic.function_id().to_string(),
            to_string_representation: CallableToStringRepresentation::NativeNamed(
                intrinsic.source_kind().operation_name().to_string(),
            ),
            protocol: if intrinsic.constructable() {
                FunctionProtocolIr::OrdinaryCallAndConstruct
            } else {
                FunctionProtocolIr::OrdinaryCallOnly
            },
            callable: true,
            class_heritage_kind: ClassHeritageKind::None,
            params: Vec::new(),
            return_kind: return_info.kind,
            return_possible_kinds: return_info.possible_kinds,
            return_shape: return_info.heap_shape.clone(),
            return_targets: return_info.function_targets.clone(),
            constructor_instance: return_info,
            this_info: self.global_this_info(),
            this_observed: false,
        }
    }

    pub(super) fn record_boxed_builtin_invocation(
        &mut self,
        builtin: StandardBuiltinId,
        context: BuiltinCallContext,
    ) {
        if !builtin.is_boxed_primitive_constructor() {
            return;
        }
        match context {
            BuiltinCallContext::Call | BuiltinCallContext::DirectEval => {
                self.boxed_builtin_calls += 1
            }
            BuiltinCallContext::Construct => self.boxed_builtin_constructs += 1,
            BuiltinCallContext::RegExpLiteral => {}
        }
    }

    pub(super) fn record_dynamic_source(
        &mut self,
        builtin: StandardBuiltinId,
        context: BuiltinCallContext,
        args: &[TypedExpr],
    ) {
        self.record_dynamic_source_for_function_id(&builtin.function_id(), context, args);
    }

    /// Records a resolved dynamic-source identity. The boolean lets call and
    /// construct lowering stop before manufacturing executable dynamic-source
    /// IR; derived constructors have no emitter and realm eval has only a
    /// defensive host body.
    pub(super) fn record_dynamic_source_for_function_id(
        &mut self,
        function_id: &str,
        context: BuiltinCallContext,
        args: &[TypedExpr],
    ) -> bool {
        if let Some(kind) = dynamic_source_kind_for_function_id(function_id, context) {
            if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                self.note_standard_builtin_call(builtin);
            }
            self.record_dynamic_source_gap(gap_for_lowered_args(kind, args));
            true
        } else {
            false
        }
    }

    pub(super) fn record_dynamic_source_targets<'a>(
        &mut self,
        function_ids: impl IntoIterator<Item = &'a FunctionId>,
        context: BuiltinCallContext,
        args: &[TypedExpr],
    ) -> bool {
        function_ids
            .into_iter()
            .cloned()
            .fold(false, |found, function_id| {
                self.record_dynamic_source_for_function_id(&function_id, context, args) || found
            })
    }

    pub(super) fn record_dynamic_source_gap(&mut self, gap: DynamicSourceGap) {
        self.diagnostics
            .push(IrDiagnostic::unsupported_dynamic_source(gap));
    }
}
