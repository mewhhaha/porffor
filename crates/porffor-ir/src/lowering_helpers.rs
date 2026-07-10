use super::*;

pub(crate) enum StaticStringGeneratorLoopBody {
    FromCharCode,
    FromCharCodeUnlessRegexpMatch(Regex),
}

pub(crate) fn function_name(
    interner: &Interner,
    function: &FunctionDeclaration,
    fallback: Option<&str>,
) -> String {
    fallback
        .map(ToString::to_string)
        .unwrap_or_else(|| interner.resolve_expect(function.name().sym()).to_string())
}

pub(crate) fn collect_simple_parameter_names(
    interner: &Interner,
    parameters: &FormalParameterList,
) -> Vec<String> {
    let mut names = Vec::with_capacity(parameters.as_ref().len());
    let mut seen = BTreeSet::new();
    for parameter in parameters.as_ref() {
        let Binding::Identifier(identifier) = parameter.variable().binding() else {
            return Vec::new();
        };
        if parameter.init().is_some() || parameter.is_rest_param() {
            return Vec::new();
        }
        let name = interner.resolve_expect(identifier.sym()).to_string();
        if !seen.insert(name.clone()) {
            return Vec::new();
        }
        names.push(name);
    }
    names
}

pub(crate) fn binding_parameter_storage_name(
    interner: &Interner,
    binding: &Binding,
    index: usize,
) -> String {
    match binding {
        Binding::Identifier(identifier) => interner.resolve_expect(identifier.sym()).to_string(),
        Binding::Pattern(_) => format!("$destructured.param.{index}"),
    }
}

pub(crate) fn collect_binding_names(
    interner: &Interner,
    binding: &Binding,
    names: &mut Vec<String>,
) {
    match binding {
        Binding::Identifier(identifier) => {
            names.push(interner.resolve_expect(identifier.sym()).to_string());
        }
        Binding::Pattern(Pattern::Object(pattern)) => {
            for element in pattern.bindings() {
                match element {
                    ObjectPatternElement::SingleName { ident, .. }
                    | ObjectPatternElement::RestProperty { ident } => {
                        names.push(interner.resolve_expect(ident.sym()).to_string());
                    }
                    ObjectPatternElement::Pattern { pattern, .. } => {
                        collect_binding_names(interner, &Binding::Pattern(pattern.clone()), names);
                    }
                    ObjectPatternElement::AssignmentPropertyAccess { .. }
                    | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => {}
                }
            }
        }
        Binding::Pattern(Pattern::Array(pattern)) => {
            for element in pattern.bindings() {
                match element {
                    ArrayPatternElement::SingleName { ident, .. }
                    | ArrayPatternElement::SingleNameRest { ident } => {
                        names.push(interner.resolve_expect(ident.sym()).to_string());
                    }
                    ArrayPatternElement::Pattern { pattern, .. }
                    | ArrayPatternElement::PatternRest { pattern } => {
                        collect_binding_names(interner, &Binding::Pattern(pattern.clone()), names);
                    }
                    ArrayPatternElement::Elision
                    | ArrayPatternElement::PropertyAccess { .. }
                    | ArrayPatternElement::PropertyAccessRest { .. } => {}
                }
            }
        }
    }
}

pub(crate) fn function_declaration_key(function: &FunctionDeclaration) -> String {
    let span = function.linear_span();
    format!(
        "function-declaration:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn is_supported_parameter_binding(binding: &Binding) -> bool {
    match binding {
        Binding::Identifier(_) => true,
        Binding::Pattern(Pattern::Object(pattern)) => pattern.bindings().iter().all(|element| {
            matches!(
                element,
                ObjectPatternElement::SingleName {
                    name: PropertyName::Literal(_),
                    ..
                }
            )
        }),
        Binding::Pattern(Pattern::Array(_)) => false,
    }
}

pub(crate) fn default_param_uses_current_or_later_name(
    expression: &Expression,
    blocked: &[String],
    interner: &Interner,
) -> bool {
    match expression {
        Expression::Identifier(identifier) => {
            let ident = interner.resolve_expect(identifier.sym()).to_string();
            blocked.iter().any(|name| name == &ident)
        }
        Expression::Parenthesized(expression) => {
            default_param_uses_current_or_later_name(expression.expression(), blocked, interner)
        }
        Expression::ArrayLiteral(array) => array
            .as_ref()
            .iter()
            .flatten()
            .any(|expr| default_param_uses_current_or_later_name(expr, blocked, interner)),
        Expression::ObjectLiteral(object) => {
            object.properties().iter().any(|property| match property {
                PropertyDefinition::Property(_, value)
                | PropertyDefinition::SpreadObject(value)
                | PropertyDefinition::CoverInitializedName(_, value) => {
                    default_param_uses_current_or_later_name(value, blocked, interner)
                }
                PropertyDefinition::MethodDefinition(_)
                | PropertyDefinition::IdentifierReference(_) => false,
            })
        }
        Expression::Unary(unary) => {
            default_param_uses_current_or_later_name(unary.target(), blocked, interner)
        }
        Expression::Binary(binary) => {
            default_param_uses_current_or_later_name(binary.lhs(), blocked, interner)
                || default_param_uses_current_or_later_name(binary.rhs(), blocked, interner)
        }
        Expression::Assign(assign) => {
            default_param_uses_current_or_later_name(assign.rhs(), blocked, interner)
                || match assign.lhs() {
                    AssignTarget::Identifier(identifier) => blocked
                        .iter()
                        .any(|name| name == &interner.resolve_expect(identifier.sym()).to_string()),
                    AssignTarget::Access(access) => {
                        default_param_property_access_uses_blocked(access, blocked, interner)
                    }
                    _ => false,
                }
        }
        Expression::Update(update) => match update.target() {
            UpdateTarget::Identifier(identifier) => blocked
                .iter()
                .any(|name| name == &interner.resolve_expect(identifier.sym()).to_string()),
            _ => false,
        },
        Expression::Call(call) => {
            default_param_uses_current_or_later_name(call.function(), blocked, interner)
                || call
                    .args()
                    .iter()
                    .any(|arg| default_param_uses_current_or_later_name(arg, blocked, interner))
        }
        Expression::PropertyAccess(access) => {
            default_param_property_access_uses_blocked(access, blocked, interner)
        }
        Expression::Optional(optional) => {
            default_param_uses_current_or_later_name(optional.target(), blocked, interner)
                || optional
                    .chain()
                    .iter()
                    .any(|operation| match operation.kind() {
                        OptionalOperationKind::SimplePropertyAccess {
                            field: PropertyAccessField::Expr(expr),
                        } => default_param_uses_current_or_later_name(expr, blocked, interner),
                        OptionalOperationKind::Call { args } => args.iter().any(|arg| {
                            default_param_uses_current_or_later_name(arg, blocked, interner)
                        }),
                        OptionalOperationKind::SimplePropertyAccess {
                            field: PropertyAccessField::Const(_),
                        }
                        | OptionalOperationKind::PrivatePropertyAccess { .. } => false,
                    })
        }
        Expression::FunctionExpression(_)
        | Expression::ArrowFunction(_)
        | Expression::AsyncArrowFunction(_)
        | Expression::Literal(_)
        | Expression::RegExpLiteral(_)
        | Expression::Spread(_)
        | Expression::GeneratorExpression(_)
        | Expression::AsyncFunctionExpression(_)
        | Expression::AsyncGeneratorExpression(_)
        | Expression::ClassExpression(_)
        | Expression::TemplateLiteral(_)
        | Expression::New(_)
        | Expression::SuperCall(_)
        | Expression::ImportCall(_)
        | Expression::TaggedTemplate(_)
        | Expression::NewTarget(_)
        | Expression::ImportMeta(_)
        | Expression::BinaryInPrivate(_)
        | Expression::Conditional(_)
        | Expression::Await(_)
        | Expression::Yield(_)
        | Expression::FormalParameterList(_)
        | Expression::This(_)
        | Expression::Debugger => false,
    }
}

pub(crate) fn default_param_property_access_uses_blocked(
    access: &PropertyAccess,
    blocked: &[String],
    interner: &Interner,
) -> bool {
    let PropertyAccess::Simple(access) = access else {
        return false;
    };
    default_param_uses_current_or_later_name(access.target(), blocked, interner)
        || matches!(
            access.field(),
            PropertyAccessField::Expr(expr)
                if default_param_uses_current_or_later_name(expr, blocked, interner)
        )
}

pub(crate) fn function_expression_key(function: &FunctionExpression) -> String {
    if let Some(span) = function.linear_span() {
        return format!("linear:{}:{}", span.start().pos(), span.end().pos());
    }
    let span = function.span();
    format!(
        "span:{}:{}:{}:{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number()
    )
}

pub(crate) fn generator_expression_callee(expression: &Expression) -> Option<&GeneratorExpression> {
    match expression {
        Expression::GeneratorExpression(generator) => Some(generator),
        Expression::Parenthesized(parenthesized) => {
            generator_expression_callee(parenthesized.expression())
        }
        _ => None,
    }
}

pub(crate) fn arrow_function_key(function: &ArrowFunction) -> String {
    let span = function.linear_span();
    format!("linear:{}:{}", span.start().pos(), span.end().pos())
}

pub(crate) fn object_method_key(method: &ObjectMethodDefinition) -> String {
    let span = method.linear_span();
    format!("object-method:{}:{}", span.start().pos(), span.end().pos())
}

pub(crate) fn for_in_loop_binding_storage_name(
    for_in: &boa_ast::statement::iteration::ForInLoop,
    source_name: &str,
) -> String {
    let span = for_in.target().span();
    format!(
        "$forin.lex.{}.{}.{}.{}.{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number(),
        source_name
    )
}

pub(crate) fn tdz_binding_storage_name(source_name: &str) -> String {
    format!("{TDZ_BINDING_STORAGE_PREFIX}{source_name}")
}

pub(crate) fn for_of_loop_binding_storage_name(for_of: &ForOfLoop, source_name: &str) -> String {
    let span = for_of.iterable().span();
    format!(
        "$forof.lex.{}.{}.{}.{}.{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number(),
        source_name
    )
}

pub(crate) fn class_method_key(method: &ClassMethodDefinition) -> String {
    let span = method.linear_span();
    format!("class-method:{}:{}", span.start().pos(), span.end().pos())
}

pub(crate) fn class_method_debug_key(key: &PropertyKeyIr) -> String {
    key.static_name().unwrap_or("<computed>").to_string()
}

pub(crate) fn class_constructor_key(function: &FunctionExpression) -> String {
    format!("class-constructor:{}", function_expression_key(function))
}

pub(crate) fn source_slice_from_positions(source_text: &str, start: usize, end: usize) -> String {
    let candidates = [
        (start, end),
        (start.saturating_sub(1), end.saturating_sub(1)),
        (start, end.saturating_sub(1)),
        (start.saturating_sub(1), end),
    ];
    for (candidate_start, candidate_end) in candidates {
        if candidate_start > candidate_end {
            continue;
        }
        if let Some(slice) = source_text.get(candidate_start..candidate_end) {
            if !slice.is_empty() || candidate_start == candidate_end {
                return slice.to_string();
            }
        }
    }
    String::new()
}

pub(crate) fn function_source_slice(function: &FunctionDeclaration, source_text: &str) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn function_expression_source_slice(
    function: &FunctionExpression,
    source_text: &str,
) -> String {
    if let Some(span) = function.linear_span() {
        return source_slice_from_positions(source_text, span.start().pos(), span.end().pos());
    }
    String::new()
}

pub(crate) fn generator_expression_source_slice(
    function: &GeneratorExpression,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn arrow_function_source_slice(function: &ArrowFunction, source_text: &str) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn object_method_source_slice(
    method: &ObjectMethodDefinition,
    source_text: &str,
) -> String {
    let span = method.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn class_method_source_slice(
    method: &ClassMethodDefinition,
    source_text: &str,
) -> String {
    let span = method.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn class_expression_source_slice(class: &ClassExpression, source_text: &str) -> String {
    let span = class.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn class_declaration_source_slice(
    class: &ClassDeclaration,
    source_text: &str,
) -> String {
    let span = class.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn private_name_key(interner: &Interner, name: PrivateName) -> String {
    interner.resolve_expect(name.description()).to_string()
}

pub(crate) fn labelled_function_declaration(
    labelled: &AstLabelled,
) -> Option<&FunctionDeclaration> {
    let mut item = labelled.item();
    loop {
        match item {
            LabelledItem::Statement(Statement::Labelled(next)) => {
                item = next.item();
            }
            LabelledItem::Statement(_) => return None,
            LabelledItem::FunctionDeclaration(function) => return Some(function),
        }
    }
}

pub(crate) fn labelled_base_statement<'b>(labelled: &'b AstLabelled) -> Option<&'b Statement> {
    let mut item = labelled.item();
    loop {
        match item {
            LabelledItem::Statement(Statement::Labelled(next)) => {
                item = next.item();
            }
            LabelledItem::Statement(statement) => return Some(statement),
            LabelledItem::FunctionDeclaration(_) => return None,
        }
    }
}
