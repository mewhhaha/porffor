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
/// pass-through variant proves that `%eval%` never reaches source evaluation.
/// The function variant retains runtime coercion and guarded source dispatch.
#[must_use = "resolved dynamic-source calls must consume their admitted proof or record their typed gap"]
pub(super) enum ResolvedDynamicSourceCall {
    EvalPassThrough(ProvenEvalPassThrough),
    FunctionInvocation(AdmittedFunctionInvocation),
    CompiledScript(ProvenCompiledScript),
    Unsupported(UnsupportedDynamicSourceCall),
}

pub(super) struct ProvenCompiledScript(());

impl ProvenCompiledScript {
    pub(super) fn into_result_info(self) -> ValueInfo {
        ValueInfo::new(ValueKind::Dynamic)
    }
}

/// A Function invocation whose coercions and source selection execute at runtime.
/// Each invocation allocates a fresh function with its execution protocol
/// and active constructor's realm.
pub(super) struct AdmittedFunctionInvocation(DynamicFunctionKind);

impl AdmittedFunctionInvocation {
    pub(super) fn into_result_info(self) -> ValueInfo {
        ScriptLowerer::empty_dynamic_function_info(self.0)
    }
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
    Construct,
    RegExpLiteral,
}

pub(super) fn dynamic_source_kind_for_function_id(function_id: &str) -> Option<DynamicSourceKind> {
    if StandardBuiltinId::from_function_id(function_id) == Some(StandardBuiltinId::EvalFunction) {
        return Some(DynamicSourceKind::IndirectEval);
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
    pub(super) fn register_dynamic_function_source(&mut self, source: DynamicFunctionSource) {
        if let Some(existing) = self
            .dynamic_function_sources
            .iter_mut()
            .find(|existing| existing.kind == source.kind && existing.arguments == source.arguments)
        {
            if source.admission
                == crate::prepared_function::DynamicFunctionSourceAdmission::Intrinsic
            {
                existing.admission = source.admission;
            }
        } else {
            self.dynamic_function_sources.push(source);
        }
    }

    pub(super) fn register_dynamic_script_source(&mut self, source: DynamicScriptSource) {
        if let Some(existing) = self
            .dynamic_script_sources
            .iter_mut()
            .find(|existing| existing.kind == source.kind && existing.source == source.source)
        {
            if source.admission == PreparedScriptAdmission::ResolvedIntrinsic {
                existing.admission = source.admission;
            }
        } else {
            self.dynamic_script_sources.push(source);
        }
    }

    pub(super) fn register_dynamic_source_candidates(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) {
        let property_name =
            |access: &boa_ast::expression::access::SimplePropertyAccess| match access.field() {
                PropertyAccessField::Const(name) => {
                    Some(self.interner.resolve_expect(name.sym()).to_string())
                }
                PropertyAccessField::Expr(expression) => self.aot_source_text(expression),
            };
        let mut candidate_callee = Self::unwrap_parenthesized_expr(callee);
        let mut candidate_arguments = arguments.iter().collect::<Vec<_>>();
        let mut forwarded = false;
        // A comma produces the right-hand value, never its Reference. Keep
        // the original expression in call lowering; this only discovers
        // optional source units when named lookup erased callable facts.
        while let Expression::Binary(binary) = candidate_callee {
            if binary.op() != BinaryOp::Comma {
                break;
            }
            candidate_callee = Self::unwrap_parenthesized_expr(binary.rhs());
            forwarded = true;
        }
        let mut reflected_constructor = false;
        if let Expression::PropertyAccess(PropertyAccess::Simple(access)) = candidate_callee {
            match property_name(access).as_deref() {
                Some("call") => {
                    candidate_callee = Self::unwrap_parenthesized_expr(access.target());
                    candidate_arguments = candidate_arguments.into_iter().skip(1).collect();
                    forwarded = true;
                }
                Some(method @ ("apply" | "construct")) => {
                    let is_reflect = matches!(
                        Self::unwrap_parenthesized_expr(access.target()),
                        Expression::Identifier(identifier)
                            if self.interner.resolve_expect(identifier.sym()).to_string() == "Reflect"
                    );
                    reflected_constructor = method == "construct" && is_reflect;
                    let (target, argument_list_index) = match (method, is_reflect) {
                        ("apply", true) => (arguments.first(), 2),
                        ("construct", true) => (arguments.first(), 1),
                        ("apply", false) => (Some(access.target()), 1),
                        ("construct", false) => return,
                        _ => unreachable!("forwarding method was matched above"),
                    };
                    let Some(target) = target else {
                        return;
                    };
                    let Some(arguments) = arguments.get(argument_list_index) else {
                        return;
                    };
                    let Expression::ArrayLiteral(array) =
                        Self::unwrap_parenthesized_expr(arguments)
                    else {
                        return;
                    };
                    let Some(arguments) = array
                        .as_ref()
                        .iter()
                        .map(Option::as_ref)
                        .collect::<Option<Vec<_>>>()
                    else {
                        return;
                    };
                    candidate_callee = Self::unwrap_parenthesized_expr(target);
                    candidate_arguments = arguments;
                    forwarded = true;
                }
                _ => {}
            }
        }
        // Forwarded calls may themselves obtain their target from a comma.
        while let Expression::Binary(binary) = candidate_callee {
            if binary.op() != BinaryOp::Comma {
                break;
            }
            candidate_callee = Self::unwrap_parenthesized_expr(binary.rhs());
            forwarded = true;
        }
        let mut known_constructor_kinds = self
            .function_source_value_candidates(candidate_callee)
            .into_iter()
            .filter_map(|candidate| match candidate {
                FiniteSourceValue::FunctionConstructor(kind) => Some(kind),
                FiniteSourceValue::Text(_)
                | FiniteSourceValue::Record(_)
                | FiniteSourceValue::Array(_) => None,
            })
            .collect::<Vec<_>>();
        let name = match candidate_callee {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let targets = self
                    .lookup_binding(&name)
                    .map(|binding| binding.function_targets)
                    .or_else(|| {
                        self.lookup_global_property_info(&name)
                            .map(|property| property.value_info.function_targets.clone())
                    });
                if let Some(targets) = targets {
                    for function_id in targets.known_targets() {
                        if let Some(DynamicSourceIntrinsic::Function(kind)) =
                            DynamicSourceIntrinsic::from_function_id(function_id)
                        {
                            if !known_constructor_kinds.contains(&kind) {
                                known_constructor_kinds.push(kind);
                            }
                        }
                    }
                }
                if name != "Function"
                    && !(forwarded && name == "eval")
                    && !reflected_constructor
                    && known_constructor_kinds.is_empty()
                {
                    return;
                }
                name
            }
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                let Some(name) = property_name(access) else {
                    return;
                };
                name
            }
            _ if reflected_constructor => String::new(),
            _ => return,
        };
        let script_kind = match name.as_str() {
            "evalScript" => Some(PreparedScriptKind::RealmScript),
            "eval" => Some(PreparedScriptKind::IndirectEval),
            _ => None,
        };
        if let Some(kind) = script_kind {
            if let Some(source) = candidate_arguments
                .first()
                .and_then(|argument| self.aot_source_text(argument))
            {
                self.register_dynamic_script_source(DynamicScriptSource {
                    admission: PreparedScriptAdmission::RuntimeCandidate,
                    kind,
                    source,
                });
            }
            return;
        }
        let kinds = if name == "Function" {
            &[DynamicFunctionKind::Ordinary][..]
        } else if reflected_constructor {
            // An unknown Reflect.construct target may be any source constructor.
            // Prepare its finite argument tuples under each constructor grammar;
            // actual callable dispatch remains the only execution authority.
            DynamicFunctionKind::ALL
        } else if !known_constructor_kinds.is_empty() {
            // Known possible targets admit source candidates even when a named
            // environment call must retain runtime Reference resolution.
            known_constructor_kinds.as_slice()
        } else {
            return;
        };
        for arguments in self.function_source_argument_candidates(&candidate_arguments) {
            for kind in kinds {
                self.register_dynamic_function_source(DynamicFunctionSource {
                    admission:
                        crate::prepared_function::DynamicFunctionSourceAdmission::RuntimeCandidate,
                    kind: *kind,
                    arguments: arguments.clone(),
                });
            }
        }
    }

    pub(super) fn aot_source_text(&self, expression: &Expression) -> Option<String> {
        match Self::unwrap_parenthesized_expr(expression) {
            Expression::Literal(literal) => match literal.kind() {
                LiteralKind::String(symbol) => Some(self.interner.resolve_expect(*symbol).join(
                    str::to_string,
                    Self::utf16_units_to_runtime_string,
                    true,
                )),
                _ => None,
            },
            Expression::TemplateLiteral(template) => {
                let mut source = String::new();
                for element in template.elements() {
                    let TemplateElement::String(symbol) = element else {
                        return None;
                    };
                    source.push_str(&self.interner.resolve_expect(*symbol).join(
                        str::to_string,
                        Self::utf16_units_to_runtime_string,
                        true,
                    ));
                }
                Some(source)
            }
            Expression::Binary(binary)
                if binary.op() == BinaryOp::Arithmetic(ArithmeticOp::Add) =>
            {
                let mut source = self.aot_source_text(binary.lhs())?;
                source.push_str(&self.aot_source_text(binary.rhs())?);
                Some(source)
            }
            _ => None,
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
            DynamicSourceIntrinsic::Function(kind) => Self::empty_dynamic_function_info(kind),
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
            BuiltinCallContext::Call => self.boxed_builtin_calls += 1,
            BuiltinCallContext::Construct => self.boxed_builtin_constructs += 1,
            BuiltinCallContext::RegExpLiteral => {}
        }
    }

    /// Classifies a resolved dynamic-source identity before call lowering can
    /// manufacture executable IR.
    pub(super) fn resolve_dynamic_source_call(
        &mut self,
        function_id: &str,
        source_args: Option<&[Expression]>,
        lowered_args: &[TypedExpr],
    ) -> Option<ResolvedDynamicSourceCall> {
        let Some(kind) = dynamic_source_kind_for_function_id(function_id) else {
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

        if let DynamicSourceKind::Function(kind) = kind {
            if source_args.is_none_or(<[Expression]>::is_empty) && lowered_args.is_empty() {
                return Some(ResolvedDynamicSourceCall::FunctionInvocation(
                    AdmittedFunctionInvocation(kind),
                ));
            }
        }

        if let DynamicSourceKind::Function(function_kind) = kind {
            if let Some(arguments) = source_args {
                for arguments in
                    self.function_source_argument_candidates(&arguments.iter().collect::<Vec<_>>())
                {
                    self.register_dynamic_function_source(DynamicFunctionSource {
                        admission: crate::prepared_function::DynamicFunctionSourceAdmission::RuntimeCandidate,
                        kind: function_kind,
                        arguments,
                    });
                }
            }
            let candidates = source_args
                .and_then(|arguments| {
                    arguments
                        .iter()
                        .enumerate()
                        .map(|(index, argument)| {
                            self.function_source_candidate(argument).or_else(|| {
                                (arguments.len() == lowered_args.len())
                                    .then(|| lowered_args.get(index))
                                    .flatten()
                                    .and_then(Self::lowered_function_source_candidate)
                            })
                        })
                        .collect::<Option<Vec<_>>>()
                })
                .or_else(|| {
                    lowered_args
                        .iter()
                        .map(Self::lowered_function_source_candidate)
                        .collect::<Option<Vec<_>>>()
                });
            if let Some(arguments) = candidates {
                let source = DynamicFunctionSource {
                    admission: if source_args.is_some_and(|arguments| {
                        arguments
                            .iter()
                            .all(|argument| self.aot_source_text(argument).is_some())
                    }) {
                        crate::prepared_function::DynamicFunctionSourceAdmission::Intrinsic
                    } else {
                        crate::prepared_function::DynamicFunctionSourceAdmission::RuntimeCandidate
                    },
                    kind: function_kind,
                    arguments,
                };
                self.register_dynamic_function_source(source);
            }
            // ToString on an argument can invoke arbitrary user code.
            self.invalidate_unknown_user_code_effects();
            return Some(ResolvedDynamicSourceCall::FunctionInvocation(
                AdmittedFunctionInvocation(function_kind),
            ));
        }

        let script_kind = match kind {
            DynamicSourceKind::RealmEvalScript => Some(PreparedScriptKind::RealmScript),
            DynamicSourceKind::IndirectEval => Some(PreparedScriptKind::IndirectEval),
            DynamicSourceKind::DirectEval | DynamicSourceKind::Function(_) => None,
        };
        if let Some(kind) = script_kind {
            if let Some(source) = source_args
                .and_then(|args| args.first())
                .and_then(|expression| self.aot_source_text(expression))
            {
                let source = DynamicScriptSource {
                    admission: PreparedScriptAdmission::ResolvedIntrinsic,
                    kind,
                    source,
                };
                self.register_dynamic_script_source(source);
                self.invalidate_unknown_user_code_effects();
                return Some(ResolvedDynamicSourceCall::CompiledScript(
                    ProvenCompiledScript(()),
                ));
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
        mut callee: TypedExpr,
        source_args: &[Expression],
    ) -> TypedExpr {
        let lowered_args = self
            .lower_call_args_expanding_spread(source_args)
            .into_arguments_after_expression(&mut callee);
        let resolved = self
            .resolve_dynamic_source_call(function_id, Some(source_args), &lowered_args)
            .expect("dynamic-source construct lowering requires a dynamic-source identity");
        match resolved {
            ResolvedDynamicSourceCall::EvalPassThrough(_) => {
                unreachable!("the intrinsic eval function is not constructable")
            }
            ResolvedDynamicSourceCall::FunctionInvocation(proof) => {
                self.mark_host_builtin_from_function_id(function_id);
                return TypedExpr::from_info(
                    proof.into_result_info(),
                    ExprIr::Construct {
                        callee: Box::new(callee),
                        args: lowered_args,
                        static_regexp_compilation: None,
                    },
                );
            }
            ResolvedDynamicSourceCall::CompiledScript(proof) => {
                self.mark_host_builtin_from_function_id(function_id);
                return TypedExpr::from_info(
                    proof.into_result_info(),
                    ExprIr::Construct {
                        callee: Box::new(callee),
                        args: lowered_args,
                        static_regexp_compilation: None,
                    },
                );
            }
            ResolvedDynamicSourceCall::Unsupported(unsupported) => {
                self.record_unsupported_dynamic_source(unsupported);
            }
        }
        TypedExpr::undefined()
    }
}
