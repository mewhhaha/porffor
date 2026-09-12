use super::*;

pub(super) const MAX_SOURCE_CANDIDATES: usize = 256;

// Optional compilation hints never replace an expression or authorize a call.
// Prepared constructors and eval still check the live arguments before dispatch.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum FiniteSourceValue {
    Text(String),
    Function(FunctionId),
    FunctionConstructor(DynamicFunctionKind),
    Record(BTreeMap<String, Vec<FiniteSourceValue>>),
    Array(Vec<FiniteSourceValue>),
}

impl FiniteSourceValue {
    pub(super) fn text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Function(_) | Self::FunctionConstructor(_) | Self::Record(_) | Self::Array(_) => {
                None
            }
        }
    }
}

pub(super) fn bounded_candidates(
    candidates: impl IntoIterator<Item = FiniteSourceValue>,
) -> Vec<FiniteSourceValue> {
    let mut distinct = BTreeSet::new();
    for candidate in candidates {
        distinct.insert(candidate);
        if distinct.len() > MAX_SOURCE_CANDIDATES {
            return Vec::new();
        }
    }
    distinct.into_iter().collect()
}

impl ScriptLowerer<'_> {
    pub(super) fn finite_binding_source_candidates(
        &self,
        name: &str,
    ) -> Option<&[FiniteSourceValue]> {
        let binding = self.lookup_binding(name);
        let storage_name = match &binding {
            Some(binding) => binding.storage_name.as_str(),
            // Nested functions read Script variables through global properties,
            // but their optional compilation hints still belong to the declaration.
            None if self.is_script_global_var_name(name) => name,
            None => return None,
        };
        self.function_source_binding_candidates
            .get(storage_name)
            .map(Vec::as_slice)
    }

    pub(super) fn function_source_value_candidates(
        &self,
        expression: &Expression,
    ) -> Vec<FiniteSourceValue> {
        let expression = Self::unwrap_parenthesized_expr(expression);
        let prototype_source = match expression {
            Expression::Call(call) => match Self::unwrap_parenthesized_expr(call.function()) {
                Expression::PropertyAccess(PropertyAccess::Simple(access))
                    if matches!(access.field(), PropertyAccessField::Const(name)
                        if self.interner.resolve_expect(name.sym()).to_string() == "getPrototypeOf")
                        && matches!(Self::unwrap_parenthesized_expr(access.target()), Expression::Identifier(identifier)
                        if matches!(self.interner.resolve_expect(identifier.sym()).to_string().as_str(), "Object" | "Reflect")) =>
                {
                    call.args().first().map(Self::unwrap_parenthesized_expr)
                }
                _ => None,
            },
            _ => Some(expression),
        };
        let constructor_kind = prototype_source.and_then(|source| match source {
            Expression::FunctionExpression(_) | Expression::ArrowFunction(_) => {
                Some(DynamicFunctionKind::Ordinary)
            }
            Expression::GeneratorExpression(_) => Some(DynamicFunctionKind::Generator),
            Expression::AsyncFunctionExpression(_) | Expression::AsyncArrowFunction(_) => {
                Some(DynamicFunctionKind::Async)
            }
            Expression::AsyncGeneratorExpression(_) => Some(DynamicFunctionKind::AsyncGenerator),
            _ => None,
        });
        if let Some(kind) = constructor_kind {
            let mut candidates = vec![FiniteSourceValue::Record(BTreeMap::from([(
                "constructor".to_string(),
                vec![FiniteSourceValue::FunctionConstructor(kind)],
            )]))];
            if let Some(function_id) = match expression {
                Expression::FunctionExpression(function) => self
                    .analysis
                    .function_expr_ids
                    .get(&function_expression_key(function)),
                Expression::ArrowFunction(function) => self
                    .analysis
                    .function_expr_ids
                    .get(&arrow_function_key(function)),
                _ => None,
            } {
                candidates.push(FiniteSourceValue::Function(function_id.clone()));
            }
            return candidates;
        }
        match expression {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                if let Some(candidates) = self.finite_binding_source_candidates(&name) {
                    return bounded_candidates(
                        candidates.iter().cloned().chain(
                            self.static_string_receiver_value(expression)
                                .map(FiniteSourceValue::Text),
                        ),
                    );
                }
            }
            Expression::ArrayLiteral(array) => {
                let elements = bounded_candidates(
                    array
                        .as_ref()
                        .iter()
                        .flatten()
                        .flat_map(|element| self.function_source_value_candidates(element)),
                );
                return vec![FiniteSourceValue::Array(elements)];
            }
            Expression::ObjectLiteral(object) => {
                let properties = object
                    .properties()
                    .iter()
                    .filter_map(|property| {
                        let (name, candidates) = match property {
                            PropertyDefinition::Property(name, value) => {
                                (name, self.function_source_value_candidates(value))
                            }
                            PropertyDefinition::MethodDefinition(method)
                                if matches!(
                                    method.kind(),
                                    MethodDefinitionKind::Ordinary | MethodDefinitionKind::Get
                                ) =>
                            {
                                let function_id = self
                                    .analysis
                                    .function_expr_ids
                                    .get(&object_method_key(method))?;
                                let candidates =
                                    vec![FiniteSourceValue::Function(function_id.clone())];
                                let candidates = if method.kind() == MethodDefinitionKind::Get {
                                    self.finite_source_call_returns(candidates)
                                } else {
                                    candidates
                                };
                                (method.name(), candidates)
                            }
                            _ => return None,
                        };
                        let name = match name {
                            PropertyName::Computed(expression) => self
                                .finite_source_computed_key_candidate(expression)
                                .or_else(|| self.property_name_to_static_key(name)),
                            PropertyName::Literal(_) => self.property_name_to_static_key(name),
                        }?;
                        (!candidates.is_empty()).then_some((name, candidates))
                    })
                    .collect();
                let mut candidates = vec![FiniteSourceValue::Record(properties)];
                if let Some(text) = self.function_source_candidate(expression) {
                    candidates.push(FiniteSourceValue::Text(text));
                }
                return candidates;
            }
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                let name = match access.field() {
                    PropertyAccessField::Const(identifier) => {
                        Some(self.interner.resolve_expect(identifier.sym()).to_string())
                    }
                    PropertyAccessField::Expr(expression) => {
                        self.finite_source_computed_key_candidate(expression)
                    }
                };
                let candidates = bounded_candidates(
                    self.function_source_value_candidates(access.target())
                        .into_iter()
                        .flat_map(|candidate| match candidate {
                            FiniteSourceValue::Record(mut properties) => name
                                .as_ref()
                                .and_then(|name| properties.remove(name))
                                .unwrap_or_default(),
                            FiniteSourceValue::Array(elements)
                                if matches!(access.field(), PropertyAccessField::Expr(_)) =>
                            {
                                elements
                            }
                            FiniteSourceValue::Text(_)
                            | FiniteSourceValue::Function(_)
                            | FiniteSourceValue::FunctionConstructor(_)
                            | FiniteSourceValue::Array(_) => Vec::new(),
                        }),
                );
                if !candidates.is_empty() {
                    return candidates;
                }
            }
            Expression::Binary(binary) if binary.op() == BinaryOp::Comma => {
                return self.function_source_value_candidates(binary.rhs());
            }
            Expression::Conditional(conditional) => {
                return bounded_candidates(
                    self.function_source_value_candidates(conditional.if_true())
                        .into_iter()
                        .chain(self.function_source_value_candidates(conditional.if_false())),
                );
            }
            Expression::Binary(binary)
                if binary.op() == BinaryOp::Arithmetic(ArithmeticOp::Add) =>
            {
                if let Some(text) = self.function_source_candidate(expression) {
                    return vec![FiniteSourceValue::Text(text)];
                }
                let left = self.function_source_value_candidates(binary.lhs());
                let right = self.function_source_value_candidates(binary.rhs());
                return bounded_candidates(
                    left.iter()
                        .filter_map(FiniteSourceValue::text)
                        .flat_map(|left| {
                            right
                                .iter()
                                .filter_map(FiniteSourceValue::text)
                                .map(move |right| FiniteSourceValue::Text(format!("{left}{right}")))
                        }),
                );
            }
            _ => {}
        }
        self.function_source_candidate(expression)
            .map(FiniteSourceValue::Text)
            .into_iter()
            .collect()
    }

    pub(super) fn function_source_argument_candidates(
        &self,
        arguments: &[&Expression],
    ) -> Vec<Vec<String>> {
        let mut tuples = vec![Vec::new()];
        for argument in arguments {
            let candidates = self.function_source_value_candidates(argument);
            let strings = candidates
                .iter()
                .filter_map(FiniteSourceValue::text)
                .collect::<BTreeSet<_>>();
            if strings.is_empty()
                || tuples.len().saturating_mul(strings.len()) > MAX_SOURCE_CANDIDATES
            {
                return Vec::new();
            }
            tuples = tuples
                .into_iter()
                .flat_map(|tuple| {
                    strings.iter().map(move |text| {
                        let mut next = tuple.clone();
                        next.push((*text).to_string());
                        next
                    })
                })
                .collect();
        }
        tuples
    }

    pub(super) fn register_array_callback_source_candidates(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) {
        let Expression::PropertyAccess(PropertyAccess::Simple(access)) =
            Self::unwrap_parenthesized_expr(callee)
        else {
            return;
        };
        let PropertyAccessField::Const(method) = access.field() else {
            return;
        };
        let method = self.interner.resolve_expect(method.sym()).to_string();
        if !matches!(
            method.as_str(),
            "forEach" | "map" | "filter" | "some" | "every" | "find" | "findIndex"
        ) {
            return;
        }
        let Some(callback) = arguments.first() else {
            return;
        };
        let key = match Self::unwrap_parenthesized_expr(callback) {
            Expression::ArrowFunction(function) => arrow_function_key(function),
            Expression::FunctionExpression(function) => function_expression_key(function),
            _ => return,
        };
        let Some(function_id) = self.analysis.function_expr_ids.get(&key).cloned() else {
            return;
        };
        let candidates = bounded_candidates(
            self.function_source_value_candidates(access.target())
                .into_iter()
                .flat_map(|candidate| match candidate {
                    FiniteSourceValue::Array(elements) => elements,
                    FiniteSourceValue::Text(_)
                    | FiniteSourceValue::Function(_)
                    | FiniteSourceValue::FunctionConstructor(_)
                    | FiniteSourceValue::Record(_) => Vec::new(),
                }),
        );
        if candidates.is_empty() {
            return;
        }
        let previous = self
            .array_callback_source_candidates
            .remove(&function_id)
            .unwrap_or_default();
        self.array_callback_source_candidates.insert(
            function_id,
            bounded_candidates(previous.into_iter().chain(candidates)),
        );
    }

    pub(super) fn install_array_callback_source_candidates(
        &mut self,
        function_id: &str,
        parameters: &FormalParameterList,
    ) {
        let Some(parameter) = parameters.as_ref().first() else {
            return;
        };
        let Binding::Identifier(identifier) = parameter.variable().binding() else {
            return;
        };
        if parameter.is_rest_param() {
            return;
        }
        let Some(candidates) = self
            .array_callback_source_candidates
            .get(function_id)
            .cloned()
        else {
            return;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let binding = self
            .lookup_binding(&name)
            .expect("callback parameter is already lowered");
        self.function_source_binding_candidates
            .insert(binding.storage_name, candidates);
    }
}
