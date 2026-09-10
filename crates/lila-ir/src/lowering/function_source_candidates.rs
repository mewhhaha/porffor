use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lowered_function_source_candidate(expression: &TypedExpr) -> Option<String> {
        if expression.possible_kinds == KindSet::from_kind(ValueKind::Undefined) {
            return Some("undefined".to_string());
        }
        if expression.possible_kinds == KindSet::from_kind(ValueKind::Null) {
            return Some("null".to_string());
        }
        match &expression.expr {
            ExprIr::String(value) => Some(value.clone()),
            ExprIr::Number(bits) => Some(Self::js_number_to_string(f64::from_bits(*bits))),
            ExprIr::Boolean(value) => Some(value.to_string()),
            ExprIr::Null => Some("null".to_string()),
            ExprIr::Undefined => Some("undefined".to_string()),
            _ => None,
        }
    }

    pub(super) fn function_source_candidate(&self, expression: &Expression) -> Option<String> {
        if matches!(Self::unwrap_parenthesized_expr(expression), Expression::Unary(unary)
            if unary.op() == UnaryOp::Void)
        {
            return Some("undefined".to_string());
        }
        if let Expression::Identifier(identifier) = Self::unwrap_parenthesized_expr(expression) {
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            if self.lookup_binding(&name).is_some_and(|binding| {
                binding.possible_kinds == KindSet::from_kind(ValueKind::Undefined)
            }) {
                return Some("undefined".to_string());
            }
        }
        self.aot_source_text(expression)
            .or_else(|| self.static_string_receiver_value(expression))
            .or_else(|| self.static_parse_float_input(expression))
            .or_else(|| {
                self.is_static_undefined_expr(expression)
                    .then(|| "undefined".to_string())
            })
            .or_else(|| self.coerced_function_source_candidate(expression))
    }

    // These are compilation candidates, not value facts: the constructor still
    // performs ToString and checks the resulting tuple before selecting code.
    // Keeping an initializer candidate across later effects cannot replace the
    // live value or suppress its coercion hooks.
    fn coerced_function_source_candidate(&self, expression: &Expression) -> Option<String> {
        match Self::unwrap_parenthesized_expr(expression) {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                self.finite_binding_source_candidates(&name)
                    .and_then(|candidates| candidates.iter().find_map(FiniteSourceValue::text))
                    .map(str::to_string)
            }
            Expression::Call(call) => {
                self.boxed_function_source_candidate(call.function(), call.args())
            }
            Expression::New(new) => {
                self.boxed_function_source_candidate(new.constructor(), new.arguments())
            }
            Expression::ObjectLiteral(object) => {
                for property in object.properties() {
                    let body = match property {
                        PropertyDefinition::Property(
                            name,
                            Expression::FunctionExpression(function),
                        ) if self.property_name_to_static_key(name).as_deref()
                            == Some("toString") =>
                        {
                            function.body()
                        }
                        PropertyDefinition::MethodDefinition(method)
                            if method.kind() == MethodDefinitionKind::Ordinary
                                && self.property_name_to_static_key(method.name()).as_deref()
                                    == Some("toString") =>
                        {
                            method.body()
                        }
                        _ => continue,
                    };
                    for statement in body.statements().iter() {
                        let StatementListItem::Statement(statement) = statement else {
                            continue;
                        };
                        if let Statement::Return(return_statement) = statement.as_ref() {
                            let value = return_statement.target()?;
                            return self
                                .aot_source_text(value)
                                .or_else(|| self.static_parse_float_input(value));
                        }
                    }
                }
                Some("[object Object]".to_string())
            }
            _ => None,
        }
    }

    fn boxed_function_source_candidate(
        &self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Option<String> {
        let Expression::Identifier(identifier) = Self::unwrap_parenthesized_expr(callee) else {
            return None;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        let argument = arguments.first();
        match name.as_str() {
            "Object" => match argument {
                None => Some("[object Object]".to_string()),
                Some(argument)
                    if self.is_static_undefined_expr(argument)
                        || matches!(Self::unwrap_parenthesized_expr(argument), Expression::Literal(literal)
                            if matches!(literal.kind(), LiteralKind::Null)) =>
                {
                    Some("[object Object]".to_string())
                }
                Some(argument) => self.function_source_candidate(argument),
            },
            "String" => match argument {
                None => Some(String::new()),
                Some(argument) => self.function_source_candidate(argument),
            },
            "Number" => argument
                .map_or(Some(0.0), |argument| self.static_to_number_expr(argument))
                .map(Self::js_number_to_string),
            "Boolean" => argument
                .map_or(Some(false), |argument| {
                    self.static_to_boolean_expr(argument)
                })
                .map(|value| value.to_string()),
            _ => None,
        }
    }
}
