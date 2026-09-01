use super::*;

/// Proof that the dynamic source text is fixed by syntax before lowering.
///
/// Its constructors are private to this module. A folded `ExprIr::String`
/// therefore cannot be promoted to AOT-known source by a downstream caller.
enum DynamicSourceProof {
    Runtime,
    AotSyntax,
}

/// The exhaustive result of resolving a call to a dynamic-source identity.
///
/// Call lowering must consume this value before it can emit executable IR. The
/// pass-through variant proves that `%eval%` never reaches source evaluation;
/// it is not evidence that any source text is AOT-compilable.
#[must_use = "resolved dynamic-source calls must execute a proven no-source eval branch or record their typed gap"]
pub(super) enum ResolvedDynamicSourceCall {
    EvalPassThrough(ProvenEvalPassThrough),
    Unsupported(UnsupportedDynamicSourceCall),
}

/// One-shot ownership of an unsupported dynamic-source invocation.
///
/// The fields stay private so the builtin-accounting identity and diagnostic
/// gap cannot be paired independently after target resolution.
pub(super) struct UnsupportedDynamicSourceCall {
    standard_builtin: Option<StandardBuiltinId>,
    gap: DynamicSourceGap,
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

pub(super) enum OptionalCallSource<'a> {
    AlreadyAccounted,
    Syntax(&'a [Expression]),
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
#[derive(Debug, PartialEq, Eq)]
pub(super) enum BuiltinCallContext {
    Call,
    DirectEval(DirectEvalCallSite),
    Construct,
    RegExpLiteral,
}

/// Proof that a resolved `%eval%` target still has direct-reference syntax.
///
/// The field is private to this module, so sibling lowering modules can route
/// ordinary calls but cannot manufacture caller-environment eval semantics.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct DirectEvalCallSite(());

/// A possible intrinsic direct-eval target captured before argument
/// evaluation can mutate the global binding.
#[must_use = "captured direct-eval identity must be resolved after argument evaluation"]
pub(super) struct ErasedDirectEvalCall {
    call_site: DirectEvalCallSite,
}

impl ErasedDirectEvalCall {
    pub(super) fn resolve(
        self,
        lowerer: &ScriptLowerer<'_>,
        source_args: &[Expression],
        lowered_args: &[TypedExpr],
    ) -> ResolvedDynamicSourceCall {
        let function_id = StandardBuiltinId::EvalFunction.function_id();
        lowerer
            .resolve_dynamic_source_call(
                &function_id,
                &BuiltinCallContext::DirectEval(self.call_site),
                Some(source_args),
                lowered_args,
            )
            .expect("a captured intrinsic eval call is a dynamic-source identity")
    }
}

pub(super) fn dynamic_source_kind_for_function_id(
    function_id: &str,
    context: &BuiltinCallContext,
) -> Option<DynamicSourceKind> {
    if StandardBuiltinId::from_function_id(function_id) == Some(StandardBuiltinId::EvalFunction) {
        return Some(match context {
            BuiltinCallContext::DirectEval(_) => DynamicSourceKind::DirectEval,
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
    /// Direct eval requires both the intrinsic target and the original
    /// direct-reference syntax. Every other resolved `%eval%` call is indirect.
    pub(super) fn resolved_builtin_call_context(
        &self,
        source_callee: &Expression,
        callee: &TypedExpr,
        function_id: &FunctionId,
    ) -> BuiltinCallContext {
        let source_is_eval_identifier = matches!(
            Self::unwrap_parenthesized_expr(source_callee),
            Expression::Identifier(identifier)
                if self.interner.resolve_expect(identifier.sym()).to_string() == "eval"
        );
        if StandardBuiltinId::from_function_id(function_id) == Some(StandardBuiltinId::EvalFunction)
            && source_is_eval_identifier
            && matches!(
                &callee.expr,
                ExprIr::GlobalPropertyRead { name } | ExprIr::GlobalIdentifierRead { name }
                    if name == "eval"
            )
        {
            BuiltinCallContext::DirectEval(DirectEvalCallSite(()))
        } else {
            BuiltinCallContext::Call
        }
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
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            DynamicSourceIntrinsic::RealmEvalScript => ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
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
            return_shape: FunctionReturnShape::flow_sensitive(return_info.heap_shape.clone()),
            return_targets: return_info.function_targets.clone(),
            constructor_instance: return_info,
            this_info: self.global_this_info(),
            this_observed: false,
            source_call_flow_effects: SourceCallFlowEffects::unobserved(),
        }
    }

    pub(super) fn record_boxed_builtin_invocation(
        &mut self,
        builtin: StandardBuiltinId,
        context: &BuiltinCallContext,
    ) {
        if !builtin.is_boxed_primitive_constructor() {
            return;
        }
        match context {
            BuiltinCallContext::Call | BuiltinCallContext::DirectEval(_) => {
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
        context: &BuiltinCallContext,
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
            UnsupportedDynamicSourceCall {
                standard_builtin: StandardBuiltinId::from_function_id(function_id),
                gap: gap_for_source_proof(kind, proof),
            },
        ))
    }

    /// Unknown user code can erase the global `%eval%` value fact without
    /// proving that it replaced or deleted the intrinsic. A direct reference
    /// must retain that capability possibility, while a proven replacement
    /// remains an ordinary call.
    pub(super) fn capture_erased_direct_eval_call(
        &self,
        source_callee: &Expression,
        callee: &TypedExpr,
    ) -> Option<ErasedDirectEvalCall> {
        if !matches!(
            InvocationTargetProvenance::from(callee),
            InvocationTargetProvenance::Erased
        ) || !callee.possible_kinds.contains(ValueKind::Function)
        {
            return None;
        }

        let eval_function_id = StandardBuiltinId::EvalFunction.function_id();
        if callee
            .function_targets
            .known_targets()
            .contains(&eval_function_id)
        {
            return None;
        }

        let direct_reference_may_resolve_to_intrinsic = match &callee.expr {
            ExprIr::GlobalPropertyRead { name } => name == "eval",
            ExprIr::GlobalIdentifierRead { name } => {
                name == "eval"
                    && self
                        .lookup_global_property_info(name)
                        .is_some_and(|property| property.source == GlobalPropertySource::Merged)
            }
            _ => false,
        };
        if !direct_reference_may_resolve_to_intrinsic {
            return None;
        }

        match self.resolved_builtin_call_context(source_callee, callee, &eval_function_id) {
            BuiltinCallContext::DirectEval(call_site) => Some(ErasedDirectEvalCall { call_site }),
            BuiltinCallContext::Call
            | BuiltinCallContext::Construct
            | BuiltinCallContext::RegExpLiteral => None,
        }
    }

    pub(super) fn record_unsupported_dynamic_source(
        &mut self,
        unsupported: UnsupportedDynamicSourceCall,
    ) {
        let UnsupportedDynamicSourceCall {
            standard_builtin,
            gap,
        } = unsupported;
        if let Some(builtin) = standard_builtin {
            self.note_standard_builtin_call(builtin);
        }
        self.diagnostics
            .push(IrDiagnostic::unsupported_dynamic_source(gap));
    }

    pub(super) fn lower_dynamic_source_construct(
        &mut self,
        function_id: &str,
        source_args: &[Expression],
    ) -> TypedExpr {
        let lowered_args = self
            .lower_call_args_expanding_spread(source_args)
            .into_arguments_without_predecessor();
        let resolved = self
            .resolve_dynamic_source_call(
                function_id,
                &BuiltinCallContext::Construct,
                Some(source_args),
                &lowered_args,
            )
            .expect("dynamic-source construct lowering requires a dynamic-source identity");
        match resolved {
            ResolvedDynamicSourceCall::EvalPassThrough(_) => {
                unreachable!("the intrinsic eval function is not constructable")
            }
            ResolvedDynamicSourceCall::Unsupported(unsupported) => {
                self.record_unsupported_dynamic_source(unsupported);
            }
        }
        TypedExpr::undefined()
    }
}
