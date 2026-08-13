use super::*;

/// Proof that the dynamic source text is fixed by syntax before lowering.
///
/// Its constructors are private to this module. A folded `ExprIr::String`
/// therefore cannot be promoted to AOT-known source by a downstream caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DynamicSourceProof {
    Runtime,
    AotSyntax,
}

/// The exhaustive result of resolving a call to a dynamic-source identity.
///
/// Call lowering must consume this value before it can emit executable IR. The
/// pass-through variant proves that `%eval%` never reaches source evaluation;
/// it is not evidence that any source text is AOT-compilable.
#[derive(Debug)]
#[must_use = "resolved dynamic-source calls must execute a proven no-source eval branch or record their typed gap"]
pub(super) enum ResolvedDynamicSourceCall {
    EvalPassThrough(ProvenEvalPassThrough),
    Unsupported(DynamicSourceGap),
}

/// Proof that the intrinsic `%eval%` call returns before parsing source.
///
/// The field and constructors stay private to this module so lowered call sites
/// cannot skip the dynamic-source diagnostic from an arbitrary return fact.
#[derive(Debug)]
pub(super) struct ProvenEvalPassThrough {
    result: ValueInfo,
}

impl ProvenEvalPassThrough {
    fn from_args(source_args: Option<&[Expression]>, lowered_args: &[TypedExpr]) -> Option<Self> {
        let source_has_spread = source_args
            .is_some_and(|args| args.iter().any(|arg| matches!(arg, Expression::Spread(_))));
        let lowered_has_spread = lowered_args
            .iter()
            .any(|arg| matches!(arg.expr, ExprIr::SpreadArgument(_)));
        if source_has_spread || lowered_has_spread {
            return None;
        }

        match lowered_args {
            [] if source_args.is_none_or(<[Expression]>::is_empty) => Some(Self {
                result: ValueInfo::undefined(),
            }),
            [first, ..]
                if source_args.is_none_or(|args| !args.is_empty())
                    && first.possible_kinds != KindSet::EMPTY
                    && !first.possible_kinds.contains(ValueKind::String) =>
            {
                Some(Self {
                    result: first.value_info(),
                })
            }
            _ => None,
        }
    }

    pub(super) fn into_result_info(self) -> ValueInfo {
        self.result
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum OptionalCallSource<'a> {
    AlreadyAccounted,
    Syntax(&'a [Expression]),
}

impl<'a> OptionalCallSource<'a> {
    pub(super) const fn syntax(self) -> Option<&'a [Expression]> {
        match self {
            Self::AlreadyAccounted => None,
            Self::Syntax(args) => Some(args),
        }
    }

    pub(super) const fn owns_diagnostic(self) -> bool {
        matches!(self, Self::Syntax(_))
    }
}

pub(super) fn already_accounted_optional_calls<'a>(
    chain: &[OptionalChainOperationIr],
) -> Vec<OptionalCallSource<'a>> {
    chain
        .iter()
        .filter(|operation| matches!(operation, OptionalChainOperationIr::Call { .. }))
        .map(|_| OptionalCallSource::AlreadyAccounted)
        .collect()
}

impl DynamicSourceProof {
    fn from_expression(expression: &Expression) -> Self {
        if has_aot_source_text_proof(expression) {
            Self::AotSyntax
        } else {
            Self::Runtime
        }
    }

    fn for_args(kind: DynamicSourceKind, args: &[Expression]) -> Self {
        match kind {
            DynamicSourceKind::DirectEval
            | DynamicSourceKind::IndirectEval
            | DynamicSourceKind::RealmEvalScript => args
                .first()
                .map_or(Self::Runtime, |source| Self::from_expression(source)),
            DynamicSourceKind::Function(
                DynamicFunctionKind::Ordinary
                | DynamicFunctionKind::Generator
                | DynamicFunctionKind::Async
                | DynamicFunctionKind::AsyncGenerator,
            ) => {
                if args.iter().all(has_aot_source_text_proof) {
                    Self::AotSyntax
                } else {
                    Self::Runtime
                }
            }
        }
    }
}

/// Recognizes only syntax whose evaluation is already the primitive string
/// value. This deliberately does not consult lowered facts or constant folds.
fn has_aot_source_text_proof(expression: &Expression) -> bool {
    match ScriptLowerer::unwrap_parenthesized_expr(expression) {
        Expression::Literal(literal) => matches!(literal.kind(), LiteralKind::String(_)),
        Expression::TemplateLiteral(template)
            if template
                .elements()
                .iter()
                .all(|element| matches!(element, TemplateElement::String(_))) =>
        {
            true
        }
        Expression::Binary(binary) if binary.op() == BinaryOp::Arithmetic(ArithmeticOp::Add) => {
            has_aot_source_text_proof(binary.lhs()) && has_aot_source_text_proof(binary.rhs())
        }
        _ => false,
    }
}

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

const fn gap_for_source_proof(
    kind: DynamicSourceKind,
    proof: DynamicSourceProof,
) -> DynamicSourceGap {
    match proof {
        DynamicSourceProof::Runtime => DynamicSourceGap::runtime_source(kind),
        DynamicSourceProof::AotSyntax => DynamicSourceGap::aot_known_source(kind),
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn resolve_constructable_dynamic_source_calls(
        &self,
        function_ids: &BTreeSet<FunctionId>,
        source_args: &[Expression],
        lowered_args: &[TypedExpr],
    ) -> Vec<(FunctionId, ResolvedDynamicSourceCall)> {
        function_ids
            .iter()
            .filter(|function_id| {
                self.function_signatures
                    .get(*function_id)
                    .is_some_and(|signature| {
                        signature.protocol.is_constructable()
                            && signature.protocol.flavor() != FunctionFlavor::Arrow
                    })
            })
            .filter_map(|function_id| {
                self.resolve_dynamic_source_call(
                    function_id,
                    BuiltinCallContext::Construct,
                    Some(source_args),
                    lowered_args,
                )
                .map(|resolved| (function_id.clone(), resolved))
            })
            .collect()
    }

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

    /// Classifies a resolved dynamic-source identity before call lowering can
    /// manufacture executable IR.
    pub(super) fn resolve_dynamic_source_call(
        &self,
        function_id: &str,
        context: BuiltinCallContext,
        source_args: Option<&[Expression]>,
        lowered_args: &[TypedExpr],
    ) -> Option<ResolvedDynamicSourceCall> {
        let Some(kind) = dynamic_source_kind_for_function_id(function_id, context) else {
            return None;
        };

        if matches!(
            kind,
            DynamicSourceKind::DirectEval | DynamicSourceKind::IndirectEval
        ) {
            if let Some(proof) = ProvenEvalPassThrough::from_args(source_args, lowered_args) {
                return Some(ResolvedDynamicSourceCall::EvalPassThrough(proof));
            }
        }

        let proof = source_args
            .map(|args| DynamicSourceProof::for_args(kind, args))
            .unwrap_or(DynamicSourceProof::Runtime);
        Some(ResolvedDynamicSourceCall::Unsupported(
            gap_for_source_proof(kind, proof),
        ))
    }

    pub(super) fn record_unsupported_dynamic_source(
        &mut self,
        function_id: &str,
        gap: DynamicSourceGap,
    ) {
        if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
            self.note_standard_builtin_call(builtin);
        }
        self.record_dynamic_source_gap(gap);
    }

    pub(super) fn lower_dynamic_source_construct(
        &mut self,
        function_id: &str,
        source_args: &[Expression],
    ) -> TypedExpr {
        let Some(lowered_args) = self.lower_call_args_expanding_spread(source_args) else {
            return TypedExpr::undefined();
        };
        let resolved = self
            .resolve_dynamic_source_call(
                function_id,
                BuiltinCallContext::Construct,
                Some(source_args),
                &lowered_args,
            )
            .expect("dynamic-source construct lowering requires a dynamic-source identity");
        match resolved {
            ResolvedDynamicSourceCall::EvalPassThrough(_) => {
                unreachable!("the intrinsic eval function is not constructable")
            }
            ResolvedDynamicSourceCall::Unsupported(gap) => {
                self.record_unsupported_dynamic_source(function_id, gap);
            }
        }
        TypedExpr::undefined()
    }

    pub(super) fn record_dynamic_source_gap(&mut self, gap: DynamicSourceGap) {
        self.diagnostics
            .push(IrDiagnostic::unsupported_dynamic_source(gap));
    }
}
