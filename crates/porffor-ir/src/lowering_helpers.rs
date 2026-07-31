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
        if seen.insert(name.clone()) {
            names.push(name);
        }
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

#[derive(Clone)]
pub(crate) struct SupportedBoundName {
    pub(crate) source_name: String,
    pub(crate) span: boa_ast::Span,
}

pub(crate) fn supported_bound_names(
    interner: &Interner,
    binding: &Binding,
) -> Option<Vec<SupportedBoundName>> {
    let identifiers = match binding {
        Binding::Identifier(identifier) => vec![identifier],
        Binding::Pattern(Pattern::Object(pattern)) => {
            let mut identifiers = Vec::with_capacity(pattern.bindings().len());
            for element in pattern.bindings() {
                let ObjectPatternElement::SingleName { name, ident, .. } = element else {
                    return None;
                };
                if !matches!(name, PropertyName::Literal(_)) {
                    return None;
                }
                identifiers.push(ident);
            }
            identifiers
        }
        Binding::Pattern(Pattern::Array(pattern)) => {
            fn collect<'a>(
                pattern: &'a boa_ast::pattern::ArrayPattern,
                identifiers: &mut Vec<&'a boa_ast::expression::Identifier>,
            ) -> Option<()> {
                for element in pattern.bindings() {
                    match element {
                        ArrayPatternElement::SingleName { ident, .. }
                        | ArrayPatternElement::SingleNameRest { ident } => identifiers.push(ident),
                        ArrayPatternElement::Pattern { pattern, .. }
                        | ArrayPatternElement::PatternRest { pattern } => {
                            let Pattern::Array(pattern) = pattern else {
                                return None;
                            };
                            collect(pattern, identifiers)?;
                        }
                        ArrayPatternElement::Elision => {}
                        ArrayPatternElement::PropertyAccess { .. }
                        | ArrayPatternElement::PropertyAccessRest { .. } => return None,
                    }
                }
                Some(())
            }

            let mut identifiers = Vec::with_capacity(pattern.bindings().len());
            collect(pattern, &mut identifiers)?;
            identifiers
        }
    };

    Some(
        identifiers
            .into_iter()
            .map(|identifier| SupportedBoundName {
                source_name: interner.resolve_expect(identifier.sym()).to_string(),
                span: identifier.span(),
            })
            .collect(),
    )
}

pub(crate) fn function_declaration_key(function: &FunctionDeclaration) -> String {
    let span = function.linear_span();
    format!(
        "function-declaration:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn generator_declaration_key(function: &GeneratorDeclaration) -> String {
    let span = function.linear_span();
    format!(
        "generator-declaration:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn async_function_declaration_key(function: &AsyncFunctionDeclaration) -> String {
    let span = function.linear_span();
    format!(
        "async-function-declaration:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn async_generator_declaration_key(function: &AsyncGeneratorDeclaration) -> String {
    let span = function.linear_span();
    format!(
        "async-generator-declaration:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn statement_list_item_function_declaration(
    item: &StatementListItem,
) -> Option<&FunctionDeclaration> {
    match item {
        StatementListItem::Declaration(declaration) => match declaration.as_ref() {
            Declaration::FunctionDeclaration(function) => Some(function),
            _ => None,
        },
        StatementListItem::Statement(statement) => match statement.as_ref() {
            Statement::Labelled(labelled) => labelled_function_declaration(labelled),
            _ => None,
        },
    }
}

pub(crate) fn annex_b_block_storage_name(
    function: &FunctionDeclaration,
    source_name: &str,
) -> String {
    let span = function.linear_span();
    format!(
        "$annexb.block.{}.{}.{}",
        span.start().pos(),
        span.end().pos(),
        source_name
    )
}

pub(crate) fn scoped_lexical_binding_storage_name(
    source_name: &str,
    span: boa_ast::Span,
) -> String {
    format!(
        "$scoped.lex.{}.{}.{}.{}.{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number(),
        source_name
    )
}

pub(crate) fn class_name_binding_storage_name(source_name: &str, span: boa_ast::Span) -> String {
    format!(
        "$class.name.{}.{}.{}.{}.{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number(),
        source_name
    )
}

pub(crate) fn is_class_name_binding_storage_name(storage_name: &str) -> bool {
    storage_name.starts_with("$class.name.")
}

pub(crate) fn is_supported_parameter_binding(binding: &Binding) -> bool {
    fn is_supported_pattern(pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Object(pattern) => pattern.bindings().iter().all(|element| match element {
                ObjectPatternElement::SingleName { .. }
                | ObjectPatternElement::RestProperty { .. } => true,
                ObjectPatternElement::Pattern { pattern, .. } => is_supported_pattern(pattern),
                ObjectPatternElement::AssignmentPropertyAccess { .. }
                | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => false,
            }),
            Pattern::Array(pattern) => pattern.bindings().iter().all(|element| match element {
                ArrayPatternElement::Elision
                | ArrayPatternElement::SingleName { .. }
                | ArrayPatternElement::SingleNameRest { .. } => true,
                ArrayPatternElement::Pattern { pattern, .. }
                | ArrayPatternElement::PatternRest { pattern } => is_supported_pattern(pattern),
                ArrayPatternElement::PropertyAccess { .. }
                | ArrayPatternElement::PropertyAccessRest { .. } => false,
            }),
        }
    }

    match binding {
        Binding::Identifier(_) => true,
        Binding::Pattern(pattern) => is_supported_pattern(pattern),
    }
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

pub(crate) fn generator_expression_key(function: &GeneratorExpression) -> String {
    let span = function.linear_span();
    format!(
        "generator-expression:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn async_function_expression_key(function: &AsyncFunctionExpression) -> String {
    let span = function.linear_span();
    format!(
        "async-function-expression:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn async_generator_expression_key(function: &AsyncGeneratorExpression) -> String {
    let span = function.linear_span();
    format!(
        "async-generator-expression:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn generator_body_has_no_suspension(body: &FunctionBody) -> bool {
    !contains(body, ContainsSymbol::YieldExpression)
}

#[derive(Default)]
struct ResumableStateAllocator {
    current_state: u32,
    suspension_points: Vec<ResumableSuspensionPointIr>,
}

impl ResumableStateAllocator {
    fn suspend(&mut self, kind: ResumableSuspensionKindIr) {
        let suspend_state = self.current_state;
        self.current_state += 1;
        self.suspension_points.push(ResumableSuspensionPointIr {
            kind,
            suspend_state,
            resume_state: self.current_state,
        });
    }

    fn finish(self) -> ResumablePlanIr {
        ResumablePlanIr {
            entry_state: 0,
            state_count: self.current_state + 1,
            suspension_points: self.suspension_points,
        }
    }
}

#[derive(Default)]
struct AsyncGeneratorSuspensionCollector {
    states: ResumableStateAllocator,
}

impl<'ast> Visitor<'ast> for AsyncGeneratorSuspensionCollector {
    type BreakTy = ();

    fn visit_return(&mut self, return_statement: &'ast AstReturn) -> ControlFlow<Self::BreakTy> {
        let Some(target) = return_statement.target() else {
            return ControlFlow::Continue(());
        };
        let _ = target.visit_with(self);
        self.states.suspend(ResumableSuspensionKindIr::Await);
        ControlFlow::Continue(())
    }

    fn visit_await(
        &mut self,
        await_expression: &'ast boa_ast::expression::Await,
    ) -> ControlFlow<Self::BreakTy> {
        let _ = await_expression.visit_with(self);
        self.states.suspend(ResumableSuspensionKindIr::Await);
        ControlFlow::Continue(())
    }

    fn visit_yield(
        &mut self,
        yield_expression: &'ast boa_ast::expression::Yield,
    ) -> ControlFlow<Self::BreakTy> {
        let _ = yield_expression.visit_with(self);
        self.states.suspend(ResumableSuspensionKindIr::Yield);
        ControlFlow::Continue(())
    }

    fn visit_for_of_loop(&mut self, for_of: &'ast ForOfLoop) -> ControlFlow<Self::BreakTy> {
        let _ = for_of.initializer().visit_with(self);
        let _ = for_of.iterable().visit_with(self);
        if for_of.r#await() {
            self.states.suspend(ResumableSuspensionKindIr::ForAwaitNext);
        }
        let _ = for_of.body().visit_with(self);
        if for_of.r#await() {
            self.states
                .suspend(ResumableSuspensionKindIr::ForAwaitClose);
        }
        ControlFlow::Continue(())
    }

    fn visit_function_declaration(
        &mut self,
        _function: &'ast FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_generator_declaration(
        &mut self,
        _function: &'ast GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_async_function_declaration(
        &mut self,
        _function: &'ast AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_async_generator_declaration(
        &mut self,
        _function: &'ast AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_function_expression(
        &mut self,
        _function: &'ast FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_generator_expression(
        &mut self,
        _function: &'ast GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_async_function_expression(
        &mut self,
        _function: &'ast AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_async_generator_expression(
        &mut self,
        _function: &'ast AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_arrow_function(
        &mut self,
        _function: &'ast ArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_async_arrow_function(
        &mut self,
        _function: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_class_declaration(
        &mut self,
        class: &'ast ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(heritage) = class.super_ref() {
            let _ = heritage.visit_with(self);
        }
        for element in class.elements() {
            let _ = self.visit_class_element(element);
        }
        ControlFlow::Continue(())
    }

    fn visit_class_expression(
        &mut self,
        class: &'ast ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        if let Some(heritage) = class.super_ref() {
            let _ = heritage.visit_with(self);
        }
        for element in class.elements() {
            let _ = self.visit_class_element(element);
        }
        ControlFlow::Continue(())
    }

    fn visit_class_element(&mut self, element: &'ast ClassElement) -> ControlFlow<Self::BreakTy> {
        if let ClassElement::MethodDefinition(method) = element {
            if let ClassElementName::PropertyName(name) = method.name() {
                let _ = name.visit_with(self);
            }
            return ControlFlow::Continue(());
        }
        let _ = element.visit_with(self);
        ControlFlow::Continue(())
    }

    fn visit_object_method_definition(
        &mut self,
        method: &'ast ObjectMethodDefinition,
    ) -> ControlFlow<Self::BreakTy> {
        let _ = method.name().visit_with(self);
        ControlFlow::Continue(())
    }
}

pub(crate) fn async_generator_resumable_plan(body: &FunctionBody) -> ResumablePlanIr {
    let mut collector = AsyncGeneratorSuspensionCollector::default();
    let _ = body.visit_with(&mut collector);
    collector.states.finish()
}

pub(crate) fn linear_generator_plan(body: &FunctionBody) -> Option<GeneratorPlanIr> {
    let mut suspension_points = Vec::new();
    let mut current_state = 0u32;
    for item in body.statements() {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                return None;
            }
            continue;
        };
        let yield_expression = match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression)) => Some((expression, true)),
            Statement::Expression(Expression::Assign(assignment))
                if assignment.op() == AssignOp::Assign
                    && matches!(
                        assignment.lhs(),
                        AssignTarget::Identifier(_) | AssignTarget::Access(_)
                    ) =>
            {
                match assignment.rhs() {
                    Expression::Yield(expression) => Some((expression, false)),
                    _ => None,
                }
            }
            Statement::Return(statement) => match statement.target() {
                Some(Expression::Yield(expression)) => Some((expression, true)),
                _ => None,
            },
            _ => None,
        };
        if let Some((yield_expression, nested_yield_allowed)) = yield_expression {
            let yield_count =
                direct_generator_yield_count(yield_expression.target(), nested_yield_allowed)?;
            for _ in 0..yield_count {
                let suspend_state = current_state;
                current_state += 1;
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state,
                    resume_state: current_state,
                });
            }
            continue;
        }
        if let Statement::Return(statement) = statement.as_ref() {
            if let Some(target) = statement
                .target()
                .filter(|target| contains(*target, ContainsSymbol::YieldExpression))
            {
                let yield_count = staged_generator_expression_yield_count(target)?;
                for _ in 0..yield_count {
                    let suspend_state = current_state;
                    current_state += 1;
                    suspension_points.push(GeneratorSuspensionPointIr {
                        suspend_state,
                        resume_state: current_state,
                    });
                }
                continue;
            }
        }
        if let Statement::Expression(expression) = statement.as_ref() {
            if contains(expression, ContainsSymbol::YieldExpression) {
                append_discarded_generator_expression_suspensions(
                    expression,
                    &mut current_state,
                    &mut suspension_points,
                )?;
                continue;
            }
        }
        if let Statement::Block(block) = statement.as_ref() {
            if contains(block, ContainsSymbol::YieldExpression) {
                append_discarded_generator_block_suspensions(
                    block.statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )?;
                continue;
            }
        }
        if let Statement::With(with) = statement.as_ref() {
            if contains(with.expression(), ContainsSymbol::YieldExpression) {
                return None;
            }
            match with.statement() {
                Statement::Block(block) => append_discarded_generator_block_suspensions(
                    block.statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )?,
                Statement::Expression(expression) => {
                    append_discarded_generator_expression_suspensions(
                        expression,
                        &mut current_state,
                        &mut suspension_points,
                    )?;
                }
                statement if contains(statement, ContainsSymbol::YieldExpression) => return None,
                _ => {}
            }
            continue;
        }
        let loop_shape = match statement.as_ref() {
            Statement::ForLoop(loop_statement) => Some((
                loop_statement.body(),
                matches!(loop_statement.init(), Some(ForLoopInitializer::Lexical(_))),
                loop_statement,
            )),
            _ => None,
        };
        if let Some((loop_body, reject_nested_functions, loop_statement)) = loop_shape {
            if !simple_generator_loop_body_is_supported(loop_body)
                || generator_loop_has_unsupported_construct(loop_statement, reject_nested_functions)
            {
                return None;
            }
            let resume_state = current_state + 1;
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state: current_state,
                resume_state,
            });
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state: resume_state,
                resume_state,
            });
            current_state += 2;
            continue;
        }
        if let Statement::WhileLoop(loop_statement) = statement.as_ref() {
            if !simple_generator_loop_body_is_supported(loop_statement.body())
                || generator_loop_has_unsupported_construct(loop_statement, false)
            {
                return None;
            }
            let resume_state = current_state + 1;
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state: current_state,
                resume_state,
            });
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state: resume_state,
                resume_state,
            });
            current_state += 2;
            continue;
        }
        if let Statement::If(if_statement) = statement.as_ref() {
            if contains(if_statement.cond(), ContainsSymbol::YieldExpression) {
                return None;
            }
            let then_yields = simple_generator_if_branch_yield_count(if_statement.body())?;
            let else_yields = match if_statement.else_node() {
                Some(branch) => simple_generator_if_branch_yield_count(branch)?,
                None => 0,
            };
            let yield_count = then_yields + else_yields;
            if yield_count == 0 {
                continue;
            }
            for resume_offset in 1..=yield_count {
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state: current_state,
                    resume_state: current_state + resume_offset as u32,
                });
            }
            current_state += yield_count as u32 + 1;
            continue;
        }
        if let Statement::Try(try_statement) = statement.as_ref() {
            append_structured_generator_suspensions(
                try_statement.block().statement_list().statements(),
                &mut current_state,
                &mut suspension_points,
            )?;
            current_state += 1;
            if let Some(catch) = try_statement.catch() {
                append_structured_generator_suspensions(
                    catch.block().statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )?;
                current_state += 1;
            }
            if let Some(finally) = try_statement.finally() {
                append_structured_generator_suspensions(
                    finally.block().statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )?;
                current_state += 1;
            }
            continue;
        }
        if contains(statement.as_ref(), ContainsSymbol::YieldExpression) {
            return None;
        }
    }
    Some(GeneratorPlanIr {
        entry_state: 0,
        state_count: current_state + 1,
        suspension_points,
    })
}

fn append_structured_generator_suspensions(
    statements: &[StatementListItem],
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                return None;
            }
            continue;
        };
        let yield_expression = match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression)) => Some((expression, true)),
            Statement::Expression(Expression::Assign(assignment))
                if assignment.op() == AssignOp::Assign
                    && matches!(
                        assignment.lhs(),
                        AssignTarget::Identifier(_) | AssignTarget::Access(_)
                    ) =>
            {
                match assignment.rhs() {
                    Expression::Yield(expression) => Some((expression, false)),
                    _ => None,
                }
            }
            Statement::Return(statement) => match statement.target() {
                Some(Expression::Yield(expression)) => Some((expression, true)),
                _ => None,
            },
            _ => None,
        };
        if let Some((yield_expression, nested_yield_allowed)) = yield_expression {
            let yield_count =
                direct_generator_yield_count(yield_expression.target(), nested_yield_allowed)?;
            for _ in 0..yield_count {
                let suspend_state = *current_state;
                *current_state += 1;
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state,
                    resume_state: *current_state,
                });
            }
            continue;
        }
        if let Statement::Return(statement) = statement.as_ref() {
            if let Some(target) = statement
                .target()
                .filter(|target| contains(*target, ContainsSymbol::YieldExpression))
            {
                let yield_count = staged_generator_expression_yield_count(target)?;
                for _ in 0..yield_count {
                    let suspend_state = *current_state;
                    *current_state += 1;
                    suspension_points.push(GeneratorSuspensionPointIr {
                        suspend_state,
                        resume_state: *current_state,
                    });
                }
                continue;
            }
        }
        if let Statement::Expression(expression) = statement.as_ref() {
            if contains(expression, ContainsSymbol::YieldExpression) {
                append_discarded_generator_expression_suspensions(
                    expression,
                    current_state,
                    suspension_points,
                )?;
                continue;
            }
        }
        if let Statement::Block(block) = statement.as_ref() {
            if contains(block, ContainsSymbol::YieldExpression) {
                append_discarded_generator_block_suspensions(
                    block.statement_list().statements(),
                    current_state,
                    suspension_points,
                )?;
                continue;
            }
        }
        if let Statement::Try(try_statement) = statement.as_ref() {
            append_structured_generator_suspensions(
                try_statement.block().statement_list().statements(),
                current_state,
                suspension_points,
            )?;
            *current_state += 1;
            if let Some(catch) = try_statement.catch() {
                append_structured_generator_suspensions(
                    catch.block().statement_list().statements(),
                    current_state,
                    suspension_points,
                )?;
                *current_state += 1;
            }
            if let Some(finally) = try_statement.finally() {
                append_structured_generator_suspensions(
                    finally.block().statement_list().statements(),
                    current_state,
                    suspension_points,
                )?;
                *current_state += 1;
            }
            continue;
        }
        if contains(statement.as_ref(), ContainsSymbol::YieldExpression) {
            return None;
        }
    }
    Some(())
}

fn direct_generator_yield_count(
    target: Option<&Expression>,
    nested_yield_allowed: bool,
) -> Option<u32> {
    let Some(target) = target else {
        return Some(1);
    };
    if !contains(target, ContainsSymbol::YieldExpression) {
        return Some(1);
    }
    if !nested_yield_allowed {
        return None;
    }
    staged_generator_expression_yield_count(target)?.checked_add(1)
}

fn staged_generator_expression_yield_count(expression: &Expression) -> Option<u32> {
    match expression {
        Expression::Parenthesized(parenthesized) => {
            staged_generator_expression_yield_count(parenthesized.expression())
        }
        Expression::Yield(yield_expression) => {
            let nested_count = match yield_expression.target() {
                Some(target) => staged_generator_expression_yield_count(target)?,
                None => 0,
            };
            nested_count.checked_add(1)
        }
        Expression::Call(call) => {
            if contains(call.function(), ContainsSymbol::YieldExpression)
                || call
                    .args()
                    .iter()
                    .any(|arg| matches!(arg, Expression::Spread(_)))
            {
                return None;
            }
            call.args().iter().try_fold(0u32, |count, argument| {
                count.checked_add(staged_generator_expression_yield_count(argument)?)
            })
        }
        Expression::ArrayLiteral(array) => {
            array
                .as_ref()
                .iter()
                .try_fold(0u32, |count, element| match element {
                    Some(Expression::Spread(spread)) => {
                        count.checked_add(staged_generator_expression_yield_count(spread.target())?)
                    }
                    Some(element) => {
                        count.checked_add(staged_generator_expression_yield_count(element)?)
                    }
                    None => Some(count),
                })
        }
        Expression::ObjectLiteral(object) => {
            object
                .properties()
                .iter()
                .try_fold(0u32, |count, property| match property {
                    PropertyDefinition::SpreadObject(source) => {
                        count.checked_add(staged_generator_expression_yield_count(source)?)
                    }
                    PropertyDefinition::Property(PropertyName::Literal(_), value) => {
                        count.checked_add(staged_generator_expression_yield_count(value)?)
                    }
                    _ => None,
                })
        }
        expression if contains(expression, ContainsSymbol::YieldExpression) => None,
        _ => Some(0),
    }
}

fn append_discarded_generator_block_suspensions(
    statements: &[StatementListItem],
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            return None;
        };
        match statement.as_ref() {
            Statement::Expression(expression) => {
                append_discarded_generator_expression_suspensions(
                    expression,
                    current_state,
                    suspension_points,
                )?;
            }
            Statement::Block(block) => append_discarded_generator_block_suspensions(
                block.statement_list().statements(),
                current_state,
                suspension_points,
            )?,
            statement if contains(statement, ContainsSymbol::YieldExpression) => return None,
            _ => {}
        }
    }
    Some(())
}

fn append_discarded_generator_expression_suspensions(
    expression: &Expression,
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match expression {
        Expression::Parenthesized(parenthesized) => {
            append_discarded_generator_expression_suspensions(
                parenthesized.expression(),
                current_state,
                suspension_points,
            )
        }
        Expression::Yield(yield_expression) => {
            if direct_generator_yield_count(yield_expression.target(), true)? != 1 {
                return None;
            }
            let suspend_state = *current_state;
            *current_state += 1;
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state,
                resume_state: *current_state,
            });
            Some(())
        }
        Expression::ArrayLiteral(array) => {
            for element in array.as_ref().iter().flatten() {
                if matches!(element, Expression::Spread(_)) {
                    return None;
                }
                append_discarded_generator_expression_suspensions(
                    element,
                    current_state,
                    suspension_points,
                )?;
            }
            Some(())
        }
        Expression::Binary(binary) if binary.op() == BinaryOp::Comma => {
            append_discarded_generator_expression_suspensions(
                binary.lhs(),
                current_state,
                suspension_points,
            )?;
            append_discarded_generator_expression_suspensions(
                binary.rhs(),
                current_state,
                suspension_points,
            )
        }
        Expression::Binary(binary) if binary.op() == BinaryOp::Arithmetic(ArithmeticOp::Add) => {
            append_discarded_generator_expression_suspensions(
                binary.lhs(),
                current_state,
                suspension_points,
            )?;
            append_discarded_generator_expression_suspensions(
                binary.rhs(),
                current_state,
                suspension_points,
            )
        }
        Expression::Conditional(conditional) => {
            let mut condition = conditional.condition();
            while let Expression::Parenthesized(parenthesized) = condition {
                condition = parenthesized.expression();
            }
            let Expression::Yield(condition_yield) = condition else {
                return None;
            };
            if direct_generator_yield_count(condition_yield.target(), true)? != 1 {
                return None;
            }
            let condition_suspend_state = *current_state;
            *current_state += 1;
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state: condition_suspend_state,
                resume_state: *current_state,
            });

            let branch_entry_state = *current_state;
            for (resume_offset, branch) in [conditional.if_true(), conditional.if_false()]
                .into_iter()
                .enumerate()
            {
                let mut branch = branch;
                while let Expression::Parenthesized(parenthesized) = branch {
                    branch = parenthesized.expression();
                }
                let Expression::Yield(branch_yield) = branch else {
                    return None;
                };
                if direct_generator_yield_count(branch_yield.target(), true)? != 1 {
                    return None;
                }
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state: branch_entry_state,
                    resume_state: branch_entry_state + resume_offset as u32 + 1,
                });
            }
            *current_state = branch_entry_state + 3;
            Some(())
        }
        Expression::Assign(assignment)
            if assignment.op() == AssignOp::Assign
                && matches!(assignment.lhs(), AssignTarget::Identifier(_))
                && matches!(assignment.rhs(), Expression::TemplateLiteral(template) if contains(template, ContainsSymbol::YieldExpression)) =>
        {
            let Expression::TemplateLiteral(template) = assignment.rhs() else {
                return None;
            };
            for element in template.elements() {
                let TemplateElement::Expr(expression) = element else {
                    continue;
                };
                if !contains(expression, ContainsSymbol::YieldExpression) {
                    continue;
                }
                let mut expression = expression;
                while let Expression::Parenthesized(parenthesized) = expression {
                    expression = parenthesized.expression();
                }
                let Expression::Yield(yield_expression) = expression else {
                    return None;
                };
                if direct_generator_yield_count(yield_expression.target(), true)? != 1 {
                    return None;
                }
                let suspend_state = *current_state;
                *current_state += 1;
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state,
                    resume_state: *current_state,
                });
            }
            Some(())
        }
        expression if contains(expression, ContainsSymbol::YieldExpression) => None,
        _ => Some(()),
    }
}

fn simple_generator_if_branch_yield_count(branch: &Statement) -> Option<usize> {
    let statements = match branch {
        Statement::Block(block) => block.statement_list().statements(),
        _ => {
            return match branch {
                Statement::Expression(Expression::Yield(expression)) if !expression.delegate() => {
                    Some(1)
                }
                statement if contains(statement, ContainsSymbol::YieldExpression) => None,
                _ => Some(0),
            };
        }
    };
    let mut yield_count = 0usize;
    let mut has_declaration = false;
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                return None;
            }
            has_declaration = true;
            continue;
        };
        match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression)) if !expression.delegate() => {
                yield_count += 1;
            }
            statement if contains(statement, ContainsSymbol::YieldExpression) => return None,
            _ => {}
        }
    }
    (yield_count <= 1 && !(has_declaration && yield_count == 1)).then_some(yield_count)
}

/// A generator loop body lowers to
/// `StatementIr::GeneratorLoop { before_suspension, suspension_statement, after_suspension, .. }`,
/// where everything ahead of the single direct `yield` lands in
/// `before_suspension`. A lexical (`let`/`const`) declaration survives that
/// split unchanged: it lowers to `StatementIr::Lexical` /
/// `StatementIr::LexicalBlock`, which both generator-loop compilers already
/// hoist through `initialize_direct_lexical_bindings` before running the
/// segment. That is the same allowance
/// [`simple_resumable_await_loop_body_is_supported`] already makes for the
/// await-loop shape, so `for (...) { let x = f(i); yield x; }` needs no new
/// backend support — only this predicate stood in the way (ECMA-262 14.7.4 /
/// 14.3.1: the per-iteration lexical binding is created and initialized on the
/// iteration that observes it, which is exactly the segment the loop compiler
/// re-enters on each resume).
///
/// Declaration forms with no `StatementIr::Lexical` lowering — function,
/// generator, async, class, and `using`/`await using` — stay rejected, as does
/// any declaration whose initializer itself contains a `yield`.
pub(crate) fn simple_generator_loop_body_is_supported(body: &Statement) -> bool {
    let statements = match body {
        Statement::Block(block) => block.statement_list().statements(),
        _ => {
            return matches!(body, Statement::Expression(Expression::Yield(expression)) if !expression.delegate());
        }
    };
    let mut yield_count = 0usize;
    let mut has_lexical_declaration = false;
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if !generator_loop_body_declaration_is_supported(item) {
                return false;
            }
            has_lexical_declaration = true;
            continue;
        };
        match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression)) if !expression.delegate() => {
                yield_count += 1;
            }
            statement if contains(statement, ContainsSymbol::YieldExpression) => return false,
            _ => {}
        }
    }
    if yield_count != 1 {
        return false;
    }
    // A captured lexical binding makes `lower_block` materialize a
    // `BlockIr::lexical_environment`, and `split_resumable_loop_body` refuses to
    // split such a block — the loop would then silently fall back to a plain
    // `StatementIr::For` holding a `GeneratorYield`, which the generator
    // dispatcher cannot re-enter. Only closures can capture, so rejecting every
    // nested function-like construct in the body keeps the environment empty and
    // the split guaranteed.
    !has_lexical_declaration || !generator_loop_has_unsupported_construct(body, true)
}

fn generator_loop_body_declaration_is_supported(item: &StatementListItem) -> bool {
    let StatementListItem::Declaration(declaration) = item else {
        return false;
    };
    if contains(item, ContainsSymbol::YieldExpression) {
        return false;
    }
    matches!(
        declaration.as_ref(),
        Declaration::Lexical(LexicalDeclaration::Let(_) | LexicalDeclaration::Const(_))
    )
}

pub(crate) fn simple_resumable_await_loop_body_is_supported(body: &Statement) -> bool {
    if generator_loop_has_unsupported_construct(body, false) {
        return false;
    }
    let statements = match body {
        Statement::Block(block) => block.statement_list().statements(),
        statement => {
            return matches!(
                statement,
                Statement::Expression(Expression::Await(await_expression))
                    if !contains(
                        await_expression.target(),
                        ContainsSymbol::AwaitExpression
                    ) && !contains(
                        await_expression.target(),
                        ContainsSymbol::YieldExpression
                    )
            );
        }
    };
    let mut await_count = 0usize;
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::AwaitExpression)
                || contains(item, ContainsSymbol::YieldExpression)
            {
                return false;
            }
            continue;
        };
        match statement.as_ref() {
            Statement::Expression(Expression::Await(await_expression))
                if !contains(await_expression.target(), ContainsSymbol::AwaitExpression)
                    && !contains(await_expression.target(), ContainsSymbol::YieldExpression) =>
            {
                await_count += 1;
            }
            statement
                if contains(statement, ContainsSymbol::AwaitExpression)
                    || contains(statement, ContainsSymbol::YieldExpression) =>
            {
                return false;
            }
            _ => {}
        }
    }
    await_count == 1
}

struct GeneratorLoopShapeVisitor {
    reject_nested_functions: bool,
}

impl GeneratorLoopShapeVisitor {
    fn visit_nested_function(&self) -> ControlFlow<()> {
        if self.reject_nested_functions {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<'ast> Visitor<'ast> for GeneratorLoopShapeVisitor {
    type BreakTy = ();

    fn visit_break(&mut self, _statement: &'ast AstBreak) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_continue(&mut self, _statement: &'ast AstContinue) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Break(())
    }

    fn visit_function_declaration(
        &mut self,
        _function: &'ast FunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_generator_declaration(
        &mut self,
        _function: &'ast GeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_async_function_declaration(
        &mut self,
        _function: &'ast AsyncFunctionDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_async_generator_declaration(
        &mut self,
        _function: &'ast AsyncGeneratorDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_function_expression(
        &mut self,
        _function: &'ast FunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_generator_expression(
        &mut self,
        _function: &'ast GeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_async_function_expression(
        &mut self,
        _function: &'ast AsyncFunctionExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_async_generator_expression(
        &mut self,
        _function: &'ast AsyncGeneratorExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_arrow_function(
        &mut self,
        _function: &'ast ArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_async_arrow_function(
        &mut self,
        _function: &'ast AsyncArrowFunction,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_class_declaration(
        &mut self,
        _class: &'ast ClassDeclaration,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_class_expression(
        &mut self,
        _class: &'ast ClassExpression,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }

    fn visit_object_method_definition(
        &mut self,
        _method: &'ast ObjectMethodDefinition,
    ) -> ControlFlow<Self::BreakTy> {
        self.visit_nested_function()
    }
}

fn generator_loop_has_unsupported_construct<N: VisitWith + ?Sized>(
    loop_statement: &N,
    reject_nested_functions: bool,
) -> bool {
    let mut visitor = GeneratorLoopShapeVisitor {
        reject_nested_functions,
    };
    loop_statement.visit_with(&mut visitor).is_break()
}

pub(crate) fn generator_function_is_aot_supported(
    body: &FunctionBody,
    _parameters: &FormalParameterList,
) -> bool {
    linear_generator_plan(body).is_some()
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

pub(crate) fn async_arrow_function_key(function: &AsyncArrowFunction) -> String {
    let span = function.linear_span();
    format!("async-arrow:{}:{}", span.start().pos(), span.end().pos())
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

pub(crate) fn class_field_debug_key(key: &ClassFieldKeyIr) -> String {
    match key {
        ClassFieldKeyIr::Public(name) => name.clone(),
        ClassFieldKeyIr::ComputedPublic(slot) => format!("<computed:{slot}>"),
        ClassFieldKeyIr::Private(private_name_id) => private_data_key(*private_name_id),
    }
}

pub(crate) fn class_constructor_key(function: &FunctionExpression) -> String {
    format!("class-constructor:{}", function_expression_key(function))
}

pub(crate) fn class_default_constructor_key(span: boa_ast::LinearSpan) -> String {
    format!(
        "class-default-constructor:{}:{}",
        span.start().pos(),
        span.end().pos()
    )
}

pub(crate) fn class_field_initializer_key(initializer: &Expression) -> String {
    let span = initializer.span();
    format!(
        "class-field-initializer:{}:{}:{}:{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number()
    )
}

pub(crate) fn class_static_block_key(block: &StaticBlockBody) -> String {
    let span = block.statements().span();
    format!(
        "class-static-block:{}:{}:{}:{}",
        span.start().line_number(),
        span.start().column_number(),
        span.end().line_number(),
        span.end().column_number()
    )
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

pub(crate) fn generator_declaration_source_slice(
    function: &GeneratorDeclaration,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn async_function_declaration_source_slice(
    function: &AsyncFunctionDeclaration,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn async_generator_declaration_source_slice(
    function: &AsyncGeneratorDeclaration,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn async_function_expression_source_slice(
    function: &AsyncFunctionExpression,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_positions(source_text, span.start().pos(), span.end().pos())
}

pub(crate) fn async_generator_expression_source_slice(
    function: &AsyncGeneratorExpression,
    source_text: &str,
) -> String {
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

pub(crate) fn async_arrow_function_source_slice(
    function: &AsyncArrowFunction,
    source_text: &str,
) -> String {
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
