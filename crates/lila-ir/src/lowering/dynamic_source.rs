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

#[derive(Debug, Clone, Copy)]
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
    pub(super) fn record_dynamic_source_call_targets(
        &mut self,
        callee: &TypedExpr,
        function_ids: &[FunctionId],
        args: &[Expression],
    ) -> bool {
        function_ids.iter().fold(false, |rejected, function_id| {
            let context = resolved_builtin_call_context(callee, function_id);
            self.record_dynamic_source_syntax_args(function_id, context, args) || rejected
        })
    }

    pub(super) fn record_constructable_dynamic_source_targets(
        &mut self,
        function_ids: &BTreeSet<FunctionId>,
        args: &[Expression],
    ) -> bool {
        let function_ids = function_ids
            .iter()
            .filter(|function_id| {
                self.function_signatures
                    .get(*function_id)
                    .is_some_and(|signature| {
                        signature.protocol.is_constructable()
                            && signature.protocol.flavor() != FunctionFlavor::Arrow
                    })
            })
            .collect::<Vec<_>>();
        self.record_dynamic_source_targets(function_ids, BuiltinCallContext::Construct, args)
    }

    pub(super) fn record_optional_dynamic_source(
        &mut self,
        function_id: &FunctionId,
        source: OptionalCallSource<'_>,
    ) -> bool {
        if dynamic_source_kind_for_function_id(function_id, BuiltinCallContext::Call).is_none() {
            return false;
        }
        if let OptionalCallSource::Syntax(args) = source {
            let recorded =
                self.record_dynamic_source_syntax_args(function_id, BuiltinCallContext::Call, args);
            debug_assert!(
                recorded,
                "resolved dynamic-source identity must be recorded"
            );
        }
        true
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

    /// Records a resolved dynamic-source identity. The boolean lets call and
    /// construct lowering stop before manufacturing executable dynamic-source
    /// IR; derived constructors have no emitter and realm eval has only a
    /// defensive host body.
    fn record_dynamic_source_for_function_id(
        &mut self,
        function_id: &str,
        context: BuiltinCallContext,
        proof: DynamicSourceProof,
    ) -> bool {
        if let Some(kind) = dynamic_source_kind_for_function_id(function_id, context) {
            if let Some(builtin) = StandardBuiltinId::from_function_id(function_id) {
                self.note_standard_builtin_call(builtin);
            }
            self.record_dynamic_source_gap(gap_for_source_proof(kind, proof));
            true
        } else {
            false
        }
    }

    pub(super) fn record_dynamic_source_targets<'a>(
        &mut self,
        function_ids: impl IntoIterator<Item = &'a FunctionId>,
        context: BuiltinCallContext,
        args: &[Expression],
    ) -> bool {
        function_ids
            .into_iter()
            .cloned()
            .fold(false, |found, function_id| {
                let Some(kind) = dynamic_source_kind_for_function_id(&function_id, context) else {
                    return found;
                };
                let proof = DynamicSourceProof::for_args(kind, args);
                self.record_dynamic_source_for_function_id(&function_id, context, proof) || found
            })
    }

    pub(super) fn record_dynamic_source_syntax_args(
        &mut self,
        function_id: &str,
        context: BuiltinCallContext,
        args: &[Expression],
    ) -> bool {
        let Some(kind) = dynamic_source_kind_for_function_id(function_id, context) else {
            return false;
        };
        let proof = DynamicSourceProof::for_args(kind, args);
        self.record_dynamic_source_for_function_id(function_id, context, proof)
    }

    pub(super) fn lower_dynamic_source_construct(
        &mut self,
        function_id: &str,
        args: &[Expression],
    ) -> TypedExpr {
        if self.lower_call_args_expanding_spread(args).is_some() {
            self.record_dynamic_source_syntax_args(
                function_id,
                BuiltinCallContext::Construct,
                args,
            );
        }
        TypedExpr::undefined()
    }

    pub(super) fn record_runtime_dynamic_source(
        &mut self,
        function_id: &str,
        context: BuiltinCallContext,
    ) -> bool {
        self.record_dynamic_source_for_function_id(
            function_id,
            context,
            DynamicSourceProof::Runtime,
        )
    }

    pub(super) fn record_dynamic_source_gap(&mut self, gap: DynamicSourceGap) {
        self.diagnostics
            .push(IrDiagnostic::unsupported_dynamic_source(gap));
    }
}
