use super::*;
use boa_ast::StatementList;

/// The normal Number/BigInt results an already-inferred primitive can produce
/// through ToNumeric. An untracked object can observably produce either kind.
pub(crate) fn numeric_domain(primitive: Option<&ValueInfo>) -> (bool, bool) {
    match primitive {
        Some(info) => {
            let has_number = [
                ValueKind::Undefined,
                ValueKind::Null,
                ValueKind::Boolean,
                ValueKind::Number,
                ValueKind::String,
            ]
            .into_iter()
            .any(|kind| info.possible_kinds.contains(kind));
            let has_bigint = info.possible_kinds.contains(ValueKind::BigInt);
            (has_number, has_bigint)
        }
        None => (true, true),
    }
}

/// Splices out the scope wrappers that hoisting an `await`/`yield` operand puts
/// around a loop body, so the suspension becomes a direct statement again.
///
/// `t += await p` lowers to
/// `LexicalBlock([Lexical $async.await.0, AsyncAwait, Expression(t += …)])`, and
/// `const v = await p` adds a second `Lexical` after the suspension. Both hide
/// the `AsyncAwait` one level down, where `split_resumable_loop_body` cannot see
/// it — the loop would then fall back to a straight-line `StatementIr::For`
/// holding a suspension the resumable dispatcher can never re-enter.
///
/// Only blocks that actually contain a suspension are flattened, so ordinary
/// nested scopes keep their structure. Flattening is safe for the ones that do:
/// `StatementIr::LexicalBlock` is a flat statement list in every backend (it
/// allocates bindings, it does not materialize an environment), and the names it
/// binds are already uniquified by the lowerer, so hoisting them into the loop
/// body scope cannot collide (ECMA-262 14.7 / 8.6 per-iteration bindings).
pub(crate) fn flatten_suspending_lexical_blocks(statements: Vec<StatementIr>) -> Vec<StatementIr> {
    if !statements.iter().any(block_contains_direct_suspension) {
        return statements;
    }
    let mut flattened = Vec::with_capacity(statements.len());
    for statement in statements {
        match statement {
            StatementIr::LexicalBlock(inner) if inner.iter().any(is_direct_suspension) => {
                flattened.extend(flatten_suspending_lexical_blocks(inner));
            }
            StatementIr::Block(block)
                if block.lexical_environment.is_none()
                    && block.statements.iter().any(is_direct_suspension) =>
            {
                flattened.extend(flatten_suspending_lexical_blocks(block.statements));
            }
            statement => flattened.push(statement),
        }
    }
    flattened
}

fn is_direct_suspension(statement: &StatementIr) -> bool {
    matches!(
        statement,
        StatementIr::GeneratorYield { .. }
            | StatementIr::AsyncAwait { .. }
            | StatementIr::AsyncModuleInstantiation
            | StatementIr::ResumableClassDefinition(_)
    )
}

fn block_contains_direct_suspension(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::LexicalBlock(inner) => inner.iter().any(is_direct_suspension),
        StatementIr::Block(block) if block.lexical_environment.is_none() => {
            block.statements.iter().any(is_direct_suspension)
        }
        _ => false,
    }
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
    // Nested array/object patterns and object rest properties all bind names, so the
    // walk has to recurse through both pattern shapes (ECMA-262 8.6 BoundNames).
    //
    // BoundNames is a purely syntactic function of the *binding* positions: a
    // computed property key (`{ [k]: v }`) contributes no bound name and does not
    // change the names bound by the rest of the pattern, so the key shape is
    // deliberately not inspected here. Whether a key can be *lowered* is decided by
    // the pattern lowering, not by this walk.
    fn collect<'a>(
        pattern: &'a Pattern,
        identifiers: &mut Vec<&'a boa_ast::expression::Identifier>,
    ) -> Option<()> {
        match pattern {
            Pattern::Object(pattern) => {
                for element in pattern.bindings() {
                    match element {
                        ObjectPatternElement::SingleName { ident, .. } => {
                            identifiers.push(ident);
                        }
                        ObjectPatternElement::RestProperty { ident } => identifiers.push(ident),
                        ObjectPatternElement::Pattern { pattern, .. } => {
                            collect(pattern, identifiers)?;
                        }
                        ObjectPatternElement::AssignmentPropertyAccess { .. }
                        | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => return None,
                    }
                }
            }
            Pattern::Array(pattern) => {
                for element in pattern.bindings() {
                    match element {
                        ArrayPatternElement::SingleName { ident, .. }
                        | ArrayPatternElement::SingleNameRest { ident } => identifiers.push(ident),
                        ArrayPatternElement::Pattern { pattern, .. }
                        | ArrayPatternElement::PatternRest { pattern } => {
                            collect(pattern, identifiers)?;
                        }
                        ArrayPatternElement::Elision => {}
                        ArrayPatternElement::PropertyAccess { .. }
                        | ArrayPatternElement::PropertyAccessRest { .. } => return None,
                    }
                }
            }
        }
        Some(())
    }

    let identifiers = match binding {
        Binding::Identifier(identifier) => vec![identifier],
        Binding::Pattern(pattern) => {
            let mut identifiers = Vec::new();
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

/// True when every element of an object binding pattern is a plain
/// `{ key: name }` / `{ name }` element with a literal property name, i.e. the
/// shape the statement-per-binding lowering can emit directly. Nested patterns
/// (`{ a: [b] }`) and rest properties (`{ a, ...rest }`) need the semantic
/// `ObjectDestructure` node instead.
pub(crate) fn object_pattern_binds_only_single_names(bindings: &[ObjectPatternElement]) -> bool {
    bindings.iter().all(|element| {
        matches!(
            element,
            ObjectPatternElement::SingleName {
                name: PropertyName::Literal(_),
                ..
            }
        )
    })
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

    /// Burn one state without recording a suspension point, so the next
    /// `suspend` starts from a state nothing else resumes into.
    fn reserve(&mut self) {
        self.current_state += 1;
    }

    fn reserve_async_disposable_finalizer(&mut self) {
        for _ in 0..AsyncDisposableFinalizerPlanIr::IMPLICIT_STATE_COUNT {
            self.reserve();
        }
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

    fn visit_statement_list(
        &mut self,
        statement_list: &'ast StatementList,
    ) -> ControlFlow<Self::BreakTy> {
        let async_disposable_scope_count = statement_list
            .statements()
            .iter()
            .filter(|item| async_generator_await_using_is_admitted(item))
            .count();
        for item in statement_list.statements() {
            self.visit_statement_list_item(item)?;
        }
        for _ in 0..async_disposable_scope_count {
            self.states.reserve_async_disposable_finalizer();
        }
        ControlFlow::Continue(())
    }

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
            // The iterator-close await must suspend in a state of its own. The
            // allocator otherwise chains, so `ForAwaitClose.suspend_state` would
            // land on whatever the previous point resumed into — for a body with
            // no suspension that is `ForAwaitNext.resume_state`, i.e. the state
            // the loop resumes in after awaiting `next()`. The backend derives
            // `value_resume_state` and `close_resume_state` from exactly those
            // two fields, so the collision made a `next()` resume replay the
            // close path instead. Reserving one state keeps the four states of a
            // for-await loop distinct, matching the plain-async layout
            // (`entry`, `entry+1`, `entry+2`, `entry+3`).
            self.states.reserve();
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

fn async_generator_await_using_is_admitted(item: &StatementListItem) -> bool {
    let StatementListItem::Declaration(declaration) = item else {
        return false;
    };
    let Declaration::Lexical(LexicalDeclaration::AwaitUsing(list)) = declaration.as_ref() else {
        return false;
    };
    list.as_ref().iter().all(|variable| {
        matches!(variable.binding(), Binding::Identifier(_))
            && variable.init().is_some_and(|initializer| {
                !contains(initializer, ContainsSymbol::AwaitExpression)
                    && !contains(initializer, ContainsSymbol::YieldExpression)
            })
    })
}

pub(crate) fn async_generator_resumable_plan(body: &FunctionBody) -> ResumablePlanIr {
    let mut collector = AsyncGeneratorSuspensionCollector::default();
    let _ = collector.visit_statement_list(body.statement_list());
    collector.states.finish()
}

/// A source suspension shape that the generator state plan cannot represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeneratorPlanRejection {
    /// A nested declaration (a function, class or lexical declaration at the top
    /// level of the body) contains a yield outside a staged initializer.
    YieldInDeclaration,
    /// A lexical declaration's initializer, or a class declaration's heritage or
    /// computed element name, contains a yield with no staged evaluation order.
    YieldInDeclarationOperand(StagedYieldRejection),
    /// `x = yield …`, whose operand itself hides further suspensions the
    /// resumed assignment cannot order.
    YieldOperandNotDirect,
    /// A statement-level `yield` whose operand contains a yield with no staged
    /// evaluation order.
    YieldOperandNotStageable(StagedYieldRejection),
    /// `return <expression containing a yield>` whose expression cannot be
    /// staged into a sequence of suspensions.
    ReturnOperandNotStageable(StagedYieldRejection),
    /// An expression statement whose value is discarded but whose yields cannot
    /// be staged.
    DiscardedYieldExpression(StagedYieldRejection),
    /// A bare block whose yields cannot be flattened into a sequence.
    DiscardedYieldBlock,
    /// `with (<expression containing a yield>)`.
    YieldInWithHead,
    /// A `with` body that is neither a block nor an expression statement and
    /// contains a yield.
    YieldInWithBody,
    /// A `for`/`while` body that is not one direct yield or a conditional
    /// containing one direct yield in exactly one branch.
    LoopBodyYieldNotDirect,
    /// A `for`/`while` carrying `break`, `continue`, or a nested function that
    /// would capture a per-iteration binding.
    LoopControlFlow,
    /// `if (<condition containing a yield>)`.
    YieldInIfCondition,
    /// A `try`, `catch` or `finally` block whose yields are not a countable
    /// direct sequence.
    YieldInTryStatement,
    /// Any other statement kind that contains a yield: `switch`, labelled
    /// statements, `do`-`while`, `for`-`in`, and so on.
    YieldInUnsupportedStatement,
}

impl GeneratorPlanRejection {
    /// The reported reason. Every message names the *generator body* and the
    /// shape that failed, so a sweep groups these into families instead of into
    /// one bucket labelled "function or class declaration". A rejected staged
    /// expression appends the form that has no staged evaluation order.
    pub(crate) fn message(self) -> String {
        let (shape, form) = match self {
            Self::YieldInDeclaration => (
                "generator body: a nested declaration contains a yield, which has no linear \
                 suspension plan",
                None,
            ),
            Self::YieldInDeclarationOperand(form) => (
                "generator body: a nested declaration contains a yield, which has no linear \
                 suspension plan",
                Some(form),
            ),
            Self::YieldOperandNotDirect => (
                "generator body: a yield operand hides further suspensions, which has no linear \
                 suspension plan",
                None,
            ),
            Self::YieldOperandNotStageable(form) => (
                "generator body: a yield operand hides further suspensions, which has no linear \
                 suspension plan",
                Some(form),
            ),
            Self::ReturnOperandNotStageable(form) => (
                "generator body: a `return` operand containing a yield cannot be staged into a \
                 linear suspension plan",
                Some(form),
            ),
            Self::DiscardedYieldExpression(form) => (
                "generator body: a discarded expression statement's yields cannot be flattened \
                 into a linear suspension plan",
                Some(form),
            ),
            Self::DiscardedYieldBlock => (
                "generator body: a block's yields cannot be flattened into a linear suspension \
                 plan",
                None,
            ),
            Self::YieldInWithHead => (
                "generator body: a yield in a `with` head has no linear suspension plan",
                None,
            ),
            Self::YieldInWithBody => (
                "generator body: a yield in this `with` body shape has no linear suspension plan",
                None,
            ),
            Self::LoopBodyYieldNotDirect => (
                "generator body: a loop body requiring multiple suspension positions or an \
                 unsupported nested yield has no linear suspension plan",
                None,
            ),
            Self::LoopControlFlow => (
                "generator body: a loop carrying `break`, `continue` or a capturing nested \
                 function has no linear suspension plan",
                None,
            ),
            Self::YieldInIfCondition => (
                "generator body: a yield in an `if` condition has no linear suspension plan",
                None,
            ),
            Self::YieldInTryStatement => (
                "generator body: a `try`/`catch`/`finally` block whose yields are not a direct \
                 sequence has no linear suspension plan",
                None,
            ),
            Self::YieldInUnsupportedStatement => (
                "generator body: a yield inside a statement kind with no resumable lowering \
                 (`switch`, a label, `do`-`while`, `for`-`in`) has no linear \
                 suspension plan",
                None,
            ),
        };
        match form {
            Some(form) => format!("{shape}: {}", form.message()),
            None => shape.to_string(),
        }
    }
}

pub(crate) fn linear_generator_plan(body: &FunctionBody) -> Option<GeneratorPlanIr> {
    linear_generator_plan_with_reason(body).ok()
}

pub(crate) fn linear_generator_plan_with_reason(
    body: &FunctionBody,
) -> Result<GeneratorPlanIr, GeneratorPlanRejection> {
    let mut suspension_points = Vec::new();
    let mut current_state = 0u32;
    for item in body.statements() {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                append_staged_generator_declaration_suspensions(
                    item,
                    &mut current_state,
                    &mut suspension_points,
                )?;
            }
            continue;
        };
        let yield_expression = match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression)) => Some((expression, true)),
            Statement::Expression(Expression::Assign(assignment))
                if assignment.op() == AssignOp::Assign
                    && matches!(
                        assignment.lhs(),
                        AssignTarget::Identifier(_)
                            | AssignTarget::Access(PropertyAccess::Simple(_))
                    )
                    && !contains(assignment.lhs(), ContainsSymbol::YieldExpression) =>
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
            append_direct_generator_yield_suspensions(
                yield_expression.target(),
                nested_yield_allowed,
                &mut current_state,
                &mut suspension_points,
            )?;
            continue;
        }
        if let Statement::Return(statement) = statement.as_ref() {
            if let Some(target) = statement
                .target()
                .filter(|target| contains(*target, ContainsSymbol::YieldExpression))
            {
                plan_staged_generator_expression(
                    target,
                    &mut StagedSuspensionPlan::new(&mut current_state, &mut suspension_points),
                )
                .map_err(GeneratorPlanRejection::ReturnOperandNotStageable)?;
                continue;
            }
        }
        if let Statement::Expression(expression) = statement.as_ref() {
            if contains(expression, ContainsSymbol::YieldExpression) {
                append_discarded_generator_expression_suspensions(
                    expression,
                    &mut current_state,
                    &mut suspension_points,
                )
                .map_err(GeneratorPlanRejection::DiscardedYieldExpression)?;
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
                return Err(GeneratorPlanRejection::YieldInWithHead);
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
                    )
                    .map_err(GeneratorPlanRejection::DiscardedYieldExpression)?;
                }
                statement if contains(statement, ContainsSymbol::YieldExpression) => {
                    return Err(GeneratorPlanRejection::YieldInWithBody);
                }
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
            if !simple_generator_loop_body_is_supported(loop_body) {
                if !contains(loop_body, ContainsSymbol::YieldExpression) {
                    continue;
                }
                if generator_loop_has_unsupported_construct(loop_statement, reject_nested_functions)
                {
                    return Err(GeneratorPlanRejection::LoopControlFlow);
                }
                append_nested_generator_statement_suspensions(
                    statement.as_ref(),
                    &mut current_state,
                    &mut suspension_points,
                )?;
                continue;
            }
            if generator_loop_has_unsupported_construct(loop_statement, reject_nested_functions) {
                return Err(GeneratorPlanRejection::LoopControlFlow);
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
            if !simple_generator_loop_body_is_supported(loop_statement.body()) {
                return Err(GeneratorPlanRejection::LoopBodyYieldNotDirect);
            }
            if generator_loop_has_unsupported_construct(loop_statement, false) {
                return Err(GeneratorPlanRejection::LoopControlFlow);
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
                return Err(GeneratorPlanRejection::YieldInIfCondition);
            }
            let then_yields = simple_generator_if_branch_yield_count(if_statement.body());
            let else_yields = match if_statement.else_node() {
                Some(branch) => simple_generator_if_branch_yield_count(branch),
                None => Some(0),
            };
            let (Some(then_yields), Some(else_yields)) = (then_yields, else_yields) else {
                append_nested_generator_statement_suspensions(
                    statement.as_ref(),
                    &mut current_state,
                    &mut suspension_points,
                )?;
                continue;
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
        if let Statement::ForOfLoop(for_of) = statement.as_ref() {
            if !for_of.r#await() && contains(for_of.body(), ContainsSymbol::YieldExpression) {
                append_nested_generator_statement_suspensions(
                    statement.as_ref(),
                    &mut current_state,
                    &mut suspension_points,
                )?;
                continue;
            }
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
            return Err(GeneratorPlanRejection::YieldInUnsupportedStatement);
        }
    }
    Ok(GeneratorPlanIr {
        entry_state: 0,
        state_count: current_state + 1,
        suspension_points,
    })
}

/// Reserve a distinct entry and exit for each structured owner. A child may
/// resume anywhere in its interval; its parent must not re-evaluate the loop
/// head or branch condition when that happens.
fn append_nested_generator_statement_suspensions(
    statement: &Statement,
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Result<(), GeneratorPlanRejection> {
    match statement {
        Statement::Block(block) => {
            for item in block.statement_list().statements() {
                match item {
                    StatementListItem::Statement(statement) => {
                        append_nested_generator_statement_suspensions(
                            statement,
                            current_state,
                            suspension_points,
                        )?;
                    }
                    item if contains(item, ContainsSymbol::YieldExpression) => {
                        return Err(GeneratorPlanRejection::YieldInDeclaration);
                    }
                    _ => {}
                }
            }
        }
        Statement::Expression(Expression::Yield(yield_expression)) => {
            append_direct_generator_yield_suspensions(
                yield_expression.target(),
                true,
                current_state,
                suspension_points,
            )?;
        }
        Statement::ForOfLoop(for_of) if !for_of.r#await() => {
            if contains(for_of.iterable(), ContainsSymbol::YieldExpression)
                || contains(for_of.initializer(), ContainsSymbol::YieldExpression)
            {
                return Err(GeneratorPlanRejection::YieldInUnsupportedStatement);
            }
            if generator_loop_has_unsupported_construct(for_of, false) {
                return Err(GeneratorPlanRejection::LoopControlFlow);
            }
            if !contains(for_of.body(), ContainsSymbol::YieldExpression) {
                return Ok(());
            }
            *current_state += 1;
            append_nested_generator_statement_suspensions(
                for_of.body(),
                current_state,
                suspension_points,
            )?;
            *current_state += 1;
        }
        Statement::ForLoop(for_loop) => {
            if for_loop
                .init()
                .is_some_and(|init| contains(init, ContainsSymbol::YieldExpression))
                || for_loop
                    .condition()
                    .is_some_and(|test| contains(test, ContainsSymbol::YieldExpression))
                || for_loop
                    .final_expr()
                    .is_some_and(|update| contains(update, ContainsSymbol::YieldExpression))
            {
                return Err(GeneratorPlanRejection::YieldInUnsupportedStatement);
            }
            if generator_loop_has_unsupported_construct(for_loop, false) {
                return Err(GeneratorPlanRejection::LoopControlFlow);
            }
            if !contains(for_loop.body(), ContainsSymbol::YieldExpression) {
                return Ok(());
            }
            *current_state += 1;
            append_nested_generator_statement_suspensions(
                for_loop.body(),
                current_state,
                suspension_points,
            )?;
            *current_state += 1;
        }
        Statement::If(branch) => {
            if contains(branch.cond(), ContainsSymbol::YieldExpression) {
                return Err(GeneratorPlanRejection::YieldInIfCondition);
            }
            if !contains(branch.body(), ContainsSymbol::YieldExpression)
                && !branch.else_node().is_some_and(|else_branch| {
                    contains(else_branch, ContainsSymbol::YieldExpression)
                })
            {
                return Ok(());
            }
            *current_state += 1;
            append_nested_generator_statement_suspensions(
                branch.body(),
                current_state,
                suspension_points,
            )?;
            *current_state += 1;
            if let Some(else_branch) = branch.else_node() {
                append_nested_generator_statement_suspensions(
                    else_branch,
                    current_state,
                    suspension_points,
                )?;
            }
            *current_state += 1;
        }
        statement if contains(statement, ContainsSymbol::YieldExpression) => {
            return Err(GeneratorPlanRejection::YieldInUnsupportedStatement);
        }
        _ => {}
    }
    Ok(())
}

fn append_structured_generator_suspensions(
    statements: &[StatementListItem],
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Result<(), GeneratorPlanRejection> {
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                append_staged_generator_declaration_suspensions(
                    item,
                    current_state,
                    suspension_points,
                )?;
            }
            continue;
        };
        let yield_expression = match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression)) => Some((expression, true)),
            Statement::Expression(Expression::Assign(assignment))
                if assignment.op() == AssignOp::Assign
                    && matches!(
                        assignment.lhs(),
                        AssignTarget::Identifier(_)
                            | AssignTarget::Access(PropertyAccess::Simple(_))
                    )
                    && !contains(assignment.lhs(), ContainsSymbol::YieldExpression) =>
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
            append_direct_generator_yield_suspensions(
                yield_expression.target(),
                nested_yield_allowed,
                current_state,
                suspension_points,
            )?;
            continue;
        }
        if let Statement::Return(statement) = statement.as_ref() {
            if let Some(target) = statement
                .target()
                .filter(|target| contains(*target, ContainsSymbol::YieldExpression))
            {
                plan_staged_generator_expression(
                    target,
                    &mut StagedSuspensionPlan::new(current_state, suspension_points),
                )
                .map_err(GeneratorPlanRejection::ReturnOperandNotStageable)?;
                continue;
            }
        }
        if let Statement::Expression(expression) = statement.as_ref() {
            if contains(expression, ContainsSymbol::YieldExpression) {
                append_discarded_generator_expression_suspensions(
                    expression,
                    current_state,
                    suspension_points,
                )
                .map_err(GeneratorPlanRejection::DiscardedYieldExpression)?;
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
            return Err(GeneratorPlanRejection::YieldInTryStatement);
        }
    }
    Ok(())
}

/// A statement-level `yield`, `x = yield` or `return yield`: its own operand
/// stages first, then the yield itself suspends linearly.
fn append_direct_generator_yield_suspensions(
    target: Option<&Expression>,
    nested_yield_allowed: bool,
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Result<(), GeneratorPlanRejection> {
    if let Some(target) = target.filter(|target| contains(*target, ContainsSymbol::YieldExpression))
    {
        if !nested_yield_allowed {
            return Err(GeneratorPlanRejection::YieldOperandNotDirect);
        }
        plan_staged_generator_expression(
            target,
            &mut StagedSuspensionPlan::new(current_state, suspension_points),
        )
        .map_err(GeneratorPlanRejection::YieldOperandNotStageable)?;
    }
    let suspend_state = *current_state;
    *current_state += 1;
    suspension_points.push(GeneratorSuspensionPointIr {
        suspend_state,
        resume_state: *current_state,
    });
    Ok(())
}

/// A lexical declaration with identifier bindings, or a class declaration,
/// whose initializers or class operands contain a yield.
fn append_staged_generator_declaration_suspensions(
    item: &StatementListItem,
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Result<(), GeneratorPlanRejection> {
    let StatementListItem::Declaration(declaration) = item else {
        return Err(GeneratorPlanRejection::YieldInDeclaration);
    };
    let mut plan = StagedSuspensionPlan::new(current_state, suspension_points);
    match declaration.as_ref() {
        Declaration::ClassDeclaration(class) => {
            plan_linear_class_operands(class.super_ref(), class.elements(), &mut plan)
                .map_err(GeneratorPlanRejection::YieldInDeclarationOperand)
        }
        Declaration::Lexical(
            lexical @ (LexicalDeclaration::Let(_) | LexicalDeclaration::Const(_)),
        ) => {
            for variable in lexical.variable_list().as_ref() {
                if !matches!(variable.binding(), Binding::Identifier(_)) {
                    return Err(GeneratorPlanRejection::YieldInDeclaration);
                }
                if let Some(init) = variable.init() {
                    plan_staged_generator_expression(init, &mut plan)
                        .map_err(GeneratorPlanRejection::YieldInDeclarationOperand)?;
                }
            }
            Ok(())
        }
        _ => Err(GeneratorPlanRejection::YieldInDeclaration),
    }
}

pub(crate) fn class_evaluation_expressions<'a>(
    heritage: Option<&'a Expression>,
    elements: &'a [ClassElement],
) -> impl Iterator<Item = &'a Expression> {
    heritage
        .into_iter()
        .chain(elements.iter().filter_map(|element| {
            let name = match element {
                ClassElement::MethodDefinition(method) => match method.name() {
                    ClassElementName::PropertyName(name) => name,
                    ClassElementName::PrivateName(_) => return None,
                },
                ClassElement::FieldDefinition(field)
                | ClassElement::StaticFieldDefinition(field)
                | ClassElement::AccessorFieldDefinition(field)
                | ClassElement::StaticAccessorFieldDefinition(field) => field.name(),
                ClassElement::PrivateFieldDefinition(_)
                | ClassElement::PrivateStaticFieldDefinition(_)
                | ClassElement::StaticBlock(_) => return None,
            };
            match name {
                PropertyName::Computed(expression) => Some(expression),
                PropertyName::Literal(_) => None,
            }
        }))
}

fn append_discarded_generator_block_suspensions(
    statements: &[StatementListItem],
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Result<(), GeneratorPlanRejection> {
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                append_staged_generator_declaration_suspensions(
                    item,
                    current_state,
                    suspension_points,
                )?;
            }
            continue;
        };
        match statement.as_ref() {
            Statement::Expression(expression) => {
                append_discarded_generator_expression_suspensions(
                    expression,
                    current_state,
                    suspension_points,
                )
                .map_err(GeneratorPlanRejection::DiscardedYieldExpression)?;
            }
            Statement::Block(block) => append_discarded_generator_block_suspensions(
                block.statement_list().statements(),
                current_state,
                suspension_points,
            )?,
            statement if contains(statement, ContainsSymbol::YieldExpression) => {
                return Err(GeneratorPlanRejection::DiscardedYieldBlock);
            }
            _ => {}
        }
    }
    Ok(())
}

/// `(yield a) ? yield b : yield c` with three direct yields: the one
/// discarded conditional shape that lowers to a two-branch `GeneratorIf`
/// (see `ScriptLowerer::lower_discarded_generator_expression`). Every other
/// discarded conditional stages as a value.
pub(crate) fn discarded_conditional_has_direct_yield_branches(
    conditional: &boa_ast::expression::operator::Conditional,
) -> bool {
    let direct_yield = |expression: &Expression| {
        let mut expression = expression;
        while let Expression::Parenthesized(parenthesized) = expression {
            expression = parenthesized.expression();
        }
        matches!(expression, Expression::Yield(yield_expression)
            if !yield_expression
                .target()
                .is_some_and(|target| contains(target, ContainsSymbol::YieldExpression)))
    };
    direct_yield(conditional.condition())
        && direct_yield(conditional.if_true())
        && direct_yield(conditional.if_false())
}

fn append_discarded_generator_expression_suspensions(
    expression: &Expression,
    current_state: &mut u32,
    suspension_points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Result<(), StagedYieldRejection> {
    match expression {
        Expression::Parenthesized(parenthesized) => {
            append_discarded_generator_expression_suspensions(
                parenthesized.expression(),
                current_state,
                suspension_points,
            )
        }
        Expression::ArrayLiteral(array)
            if !array
                .as_ref()
                .iter()
                .flatten()
                .any(|element| matches!(element, Expression::Spread(_))) =>
        {
            for element in array.as_ref().iter().flatten() {
                append_discarded_generator_expression_suspensions(
                    element,
                    current_state,
                    suspension_points,
                )?;
            }
            Ok(())
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
        Expression::Conditional(conditional)
            if discarded_conditional_has_direct_yield_branches(conditional) =>
        {
            let condition_suspend_state = *current_state;
            *current_state += 1;
            suspension_points.push(GeneratorSuspensionPointIr {
                suspend_state: condition_suspend_state,
                resume_state: *current_state,
            });

            let branch_entry_state = *current_state;
            for resume_offset in 1..=2 {
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state: branch_entry_state,
                    resume_state: branch_entry_state + resume_offset,
                });
            }
            *current_state = branch_entry_state + 3;
            Ok(())
        }
        Expression::Assign(assignment)
            if assignment.op() == AssignOp::Assign
                && matches!(assignment.lhs(), AssignTarget::Identifier(_))
                && matches!(assignment.rhs(), Expression::TemplateLiteral(template) if contains(template, ContainsSymbol::YieldExpression)) =>
        {
            let Expression::TemplateLiteral(template) = assignment.rhs() else {
                unreachable!("guarded template literal assignment");
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
                    return Err(StagedYieldRejection::UnstagedForm);
                };
                if yield_expression
                    .target()
                    .is_some_and(|target| contains(target, ContainsSymbol::YieldExpression))
                {
                    return Err(StagedYieldRejection::UnstagedForm);
                }
                let suspend_state = *current_state;
                *current_state += 1;
                suspension_points.push(GeneratorSuspensionPointIr {
                    suspend_state,
                    resume_state: *current_state,
                });
            }
            Ok(())
        }
        // Everything else, including `a + b`, calls, assignments and
        // destructuring, is evaluated for its value and the value discarded.
        expression => plan_staged_generator_expression(
            expression,
            &mut StagedSuspensionPlan::new(current_state, suspension_points),
        ),
    }
}

pub(crate) fn simple_generator_if_branch_yield_count(branch: &Statement) -> Option<usize> {
    let statements = match branch {
        Statement::Block(block) => block.statement_list().statements(),
        _ => {
            return match branch {
                Statement::Expression(Expression::Yield(expression))
                    if !expression.delegate()
                        && !expression.target().is_some_and(|target| {
                            contains(target, ContainsSymbol::YieldExpression)
                        }) =>
                {
                    Some(1)
                }
                statement if contains(statement, ContainsSymbol::YieldExpression) => None,
                _ => Some(0),
            };
        }
    };
    let mut yield_count = 0usize;
    let mut has_declaration = false;
    let mut declarations_are_supported = true;
    for item in statements {
        let StatementListItem::Statement(statement) = item else {
            if contains(item, ContainsSymbol::YieldExpression) {
                return None;
            }
            has_declaration = true;
            declarations_are_supported &= generator_loop_body_declaration_is_supported(item);
            continue;
        };
        match statement.as_ref() {
            Statement::Expression(Expression::Yield(expression))
                if !expression.delegate()
                    && !expression.target().is_some_and(|target| {
                        contains(target, ContainsSymbol::YieldExpression)
                    }) =>
            {
                yield_count += 1;
            }
            statement if contains(statement, ContainsSymbol::YieldExpression) => return None,
            _ => {}
        }
    }
    if yield_count == 0 {
        return Some(0);
    }
    if yield_count != 1
        || has_declaration
            && (!declarations_are_supported
                || generator_loop_has_unsupported_construct(branch, true))
    {
        return None;
    }
    Some(1)
}

/// One suspension position per loop iteration, optionally guarded by an `if`.
/// Captured body bindings still require a resumable lexical environment.
pub(crate) fn simple_generator_loop_body_is_supported(body: &Statement) -> bool {
    let statements = match body {
        Statement::Block(block) => block.statement_list().statements(),
        _ => {
            return generator_loop_statement_yield_count(body) == Some(1);
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
        let Some(statement_yields) = generator_loop_statement_yield_count(statement) else {
            return false;
        };
        yield_count += statement_yields;
    }
    if yield_count != 1 {
        return false;
    }
    // A captured lexical binding needs an Environment Record that persists
    // across suspension. This loop form carries uncaptured activation slots.
    !has_lexical_declaration || !generator_loop_has_unsupported_construct(body, true)
}

fn generator_loop_statement_yield_count(statement: &Statement) -> Option<usize> {
    match statement {
        Statement::Expression(Expression::Yield(expression))
            if !expression.delegate()
                && !expression
                    .target()
                    .is_some_and(|target| contains(target, ContainsSymbol::YieldExpression)) =>
        {
            Some(1)
        }
        Statement::If(branch) if !contains(branch.cond(), ContainsSymbol::YieldExpression) => {
            let then_count = simple_generator_if_branch_yield_count(branch.body())?;
            let else_count = branch
                .else_node()
                .map(simple_generator_if_branch_yield_count)
                .unwrap_or(Some(0))?;
            let count = then_count + else_count;
            (count <= 1).then_some(count)
        }
        statement if contains(statement, ContainsSymbol::YieldExpression) => None,
        _ => Some(0),
    }
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
    await_count > 0
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

/// `break`/`continue` anywhere inside a resumable loop body is rejected: the
/// body is re-entered one iteration per invocation, so a branch out of it has no
/// wasm control frame to land in.
pub(crate) fn generator_loop_has_unsupported_control<N: VisitWith + ?Sized>(
    loop_statement: &N,
    reject_nested_functions: bool,
) -> bool {
    generator_loop_has_unsupported_construct(loop_statement, reject_nested_functions)
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

pub(crate) const fn object_method_protocol(kind: MethodDefinitionKind) -> ObjectMethodProtocolIr {
    match kind {
        MethodDefinitionKind::Ordinary => {
            ObjectMethodProtocolIr::Method(FunctionExecutionKind::Ordinary)
        }
        MethodDefinitionKind::Generator => {
            ObjectMethodProtocolIr::Method(FunctionExecutionKind::Generator)
        }
        MethodDefinitionKind::Async => ObjectMethodProtocolIr::Method(FunctionExecutionKind::Async),
        MethodDefinitionKind::AsyncGenerator => {
            ObjectMethodProtocolIr::Method(FunctionExecutionKind::AsyncGenerator)
        }
        MethodDefinitionKind::Get => ObjectMethodProtocolIr::Getter,
        MethodDefinitionKind::Set => ObjectMethodProtocolIr::Setter,
    }
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

// `tdz_binding_storage_name` lived here. It is now
// `binding_lifecycle::TdzPlaceholderName::for_source_name`, the sole constructor
// of the `$tdz.` name domain; a bare `String` is no longer accepted where a
// placeholder name is wanted.

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

fn source_slice_from_utf16_span(source_text: &str, span: boa_ast::LinearSpan) -> String {
    source_text[source_byte_range_from_utf16_span(source_text, span)].to_string()
}

pub(crate) fn source_byte_range_from_utf16_span(
    source_text: &str,
    span: boa_ast::LinearSpan,
) -> std::ops::Range<usize> {
    let mut utf16_offset = 0;
    let mut start_byte = None;
    for (byte_offset, width) in source_text
        .char_indices()
        .map(|(byte_offset, character)| (byte_offset, character.len_utf16()))
        .chain(std::iter::once((source_text.len(), 0)))
    {
        if utf16_offset == span.start().pos() {
            start_byte = Some(byte_offset);
        }
        if utf16_offset == span.end().pos() {
            let start_byte = start_byte.expect("parser source span starts at a UTF-16 boundary");
            return start_byte..byte_offset;
        }
        utf16_offset += width;
    }
    panic!("parser source span {span:?} must end within the source text");
}

pub(crate) fn function_source_slice(function: &FunctionDeclaration, source_text: &str) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn generator_declaration_source_slice(
    function: &GeneratorDeclaration,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn async_function_declaration_source_slice(
    function: &AsyncFunctionDeclaration,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn async_generator_declaration_source_slice(
    function: &AsyncGeneratorDeclaration,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn async_function_expression_source_slice(
    function: &AsyncFunctionExpression,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn async_generator_expression_source_slice(
    function: &AsyncGeneratorExpression,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn function_expression_source_slice(
    function: &FunctionExpression,
    source_text: &str,
) -> String {
    if let Some(span) = function.linear_span() {
        return source_slice_from_utf16_span(source_text, span);
    }
    String::new()
}

pub(crate) fn generator_expression_source_slice(
    function: &GeneratorExpression,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn arrow_function_source_slice(function: &ArrowFunction, source_text: &str) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn async_arrow_function_source_slice(
    function: &AsyncArrowFunction,
    source_text: &str,
) -> String {
    let span = function.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn object_method_source_slice(
    method: &ObjectMethodDefinition,
    source_text: &str,
) -> String {
    let span = method.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn class_method_source_slice(
    method: &ClassMethodDefinition,
    source_text: &str,
) -> String {
    let span = method.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn class_expression_source_slice(class: &ClassExpression, source_text: &str) -> String {
    let span = class.linear_span();
    source_slice_from_utf16_span(source_text, span)
}

pub(crate) fn class_declaration_source_slice(
    class: &ClassDeclaration,
    source_text: &str,
) -> String {
    let span = class.linear_span();
    source_slice_from_utf16_span(source_text, span)
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

pub(crate) fn contains_async_property_assignment(expression: &Expression) -> bool {
    struct PropertyAssignment;
    impl<'ast> Visitor<'ast> for PropertyAssignment {
        type BreakTy = ();

        fn visit_expression(&mut self, expression: &'ast Expression) -> ControlFlow<()> {
            if !contains(expression, ContainsSymbol::AwaitExpression) {
                return ControlFlow::Continue(());
            }
            if matches!(expression, Expression::Assign(assignment)
                if assignment.op() == AssignOp::Assign
                    && matches!(assignment.lhs(), AssignTarget::Access(PropertyAccess::Simple(_))))
            {
                return ControlFlow::Break(());
            }
            expression.visit_with(self)
        }
    }
    PropertyAssignment.visit_expression(expression).is_break()
}

/// True when hoisting the `await`s out of `expression` would change *which* of
/// them run.
///
/// An async body suspends only in statement position: the dispatcher re-enters
/// the function and resumes at the statement matching the stored state, so the
/// lowerer rewrites `await x` into a `let` plus an `AsyncAwait` statement
/// placed *before* the statement that used it. That prefix runs
/// unconditionally and in order, so it can only carry suspensions the
/// expression itself always reaches.
///
/// The right operand of `&&`/`||`/`??` (and their compound assignments), both
/// arms of `?:`, and every link after a short-circuiting `?.` are reached only
/// on some paths. An `await` in one of those must stay where it is, which for
/// now means the statement refuses rather than silently awaiting on a path the
/// program never takes.
///
/// Anything else evaluates its operands unconditionally, left to right, so the
/// walk recurses through it. Forms that are not recognised are reported as
/// conditional whenever they contain an `await` at all, so a shape this
/// function has not been taught about refuses instead of miscompiling.
pub(crate) fn await_is_conditionally_reached(expression: &Expression) -> bool {
    if !contains(expression, ContainsSymbol::AwaitExpression) {
        return false;
    }
    match expression {
        Expression::Parenthesized(parenthesized) => {
            await_is_conditionally_reached(parenthesized.expression())
        }
        Expression::Await(await_expression) => {
            await_is_conditionally_reached(await_expression.target())
        }
        Expression::Unary(unary) => await_is_conditionally_reached(unary.target()),
        Expression::Update(update) => match update.target() {
            UpdateTarget::Identifier(_) => false,
            UpdateTarget::PropertyAccess(access) => {
                property_access_await_is_conditionally_reached(access)
            }
            UpdateTarget::WebCompatCall(call) => {
                call.args().iter().any(await_is_conditionally_reached)
            }
        },
        Expression::Binary(binary) => match binary.op() {
            // 13.13/13.14: the right operand is evaluated only when the left
            // one does not already decide the result.
            BinaryOp::Logical(_) => {
                contains(binary.rhs(), ContainsSymbol::AwaitExpression)
                    || await_is_conditionally_reached(binary.lhs())
            }
            _ => {
                await_is_conditionally_reached(binary.lhs())
                    || await_is_conditionally_reached(binary.rhs())
            }
        },
        Expression::BinaryInPrivate(binary) => await_is_conditionally_reached(binary.rhs()),
        Expression::Conditional(conditional) => {
            contains(conditional.if_true(), ContainsSymbol::AwaitExpression)
                || contains(conditional.if_false(), ContainsSymbol::AwaitExpression)
                || await_is_conditionally_reached(conditional.condition())
        }
        Expression::Assign(assign) => match assign.op() {
            AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
                contains(assign.rhs(), ContainsSymbol::AwaitExpression)
                    || assign_target_await_is_conditionally_reached(assign.lhs())
            }
            _ => {
                assign_target_await_is_conditionally_reached(assign.lhs())
                    || await_is_conditionally_reached(assign.rhs())
            }
        },
        Expression::Call(call) => {
            await_is_conditionally_reached(call.function())
                || call.args().iter().any(await_is_conditionally_reached)
        }
        Expression::New(new_expression) => {
            await_is_conditionally_reached(new_expression.constructor())
        }
        Expression::SuperCall(call) => call.arguments().iter().any(await_is_conditionally_reached),
        Expression::PropertyAccess(access) => {
            property_access_await_is_conditionally_reached(access)
        }
        // Every link after the first `?.` is skipped when the target is
        // nullish, so an `await` anywhere in the chain is path-dependent.
        Expression::Optional(optional) => {
            optional
                .chain()
                .iter()
                .any(|operation| contains(operation, ContainsSymbol::AwaitExpression))
                || await_is_conditionally_reached(optional.target())
        }
        Expression::ArrayLiteral(array) => array
            .as_ref()
            .iter()
            .flatten()
            .any(await_is_conditionally_reached),
        Expression::ObjectLiteral(object) => object
            .properties()
            .iter()
            .any(object_property_await_is_conditionally_reached),
        Expression::Spread(spread) => await_is_conditionally_reached(spread.target()),
        Expression::TemplateLiteral(template) => template
            .elements()
            .iter()
            .filter_map(|element| match element {
                TemplateElement::Expr(expression) => Some(expression),
                TemplateElement::String(_) => None,
            })
            .any(await_is_conditionally_reached),
        Expression::TaggedTemplate(template) => {
            await_is_conditionally_reached(template.tag())
                || template.exprs().iter().any(await_is_conditionally_reached)
        }
        Expression::ImportCall(call) => await_is_conditionally_reached(call.argument()),
        Expression::ClassExpression(class) => {
            class_evaluation_expressions(class.super_ref(), class.elements())
                .any(await_is_conditionally_reached)
        }
        // `contains` proved an `await` is in there, and this walk cannot show
        // it is always reached.
        _ => true,
    }
}

fn property_access_await_is_conditionally_reached(access: &PropertyAccess) -> bool {
    match access {
        PropertyAccess::Simple(access) => {
            await_is_conditionally_reached(access.target())
                || match access.field() {
                    PropertyAccessField::Const(_) => false,
                    PropertyAccessField::Expr(key) => await_is_conditionally_reached(key),
                }
        }
        PropertyAccess::Private(access) => await_is_conditionally_reached(access.target()),
        PropertyAccess::Super(access) => match access.field() {
            PropertyAccessField::Const(_) => false,
            PropertyAccessField::Expr(key) => await_is_conditionally_reached(key),
        },
    }
}

fn assign_target_await_is_conditionally_reached(target: &AssignTarget) -> bool {
    match target {
        AssignTarget::Identifier(_) => false,
        AssignTarget::Access(access) => property_access_await_is_conditionally_reached(access),
        AssignTarget::Pattern(pattern) => contains(pattern, ContainsSymbol::AwaitExpression),
        AssignTarget::WebCompatCall(call) => call.args().iter().any(await_is_conditionally_reached),
    }
}

fn object_property_await_is_conditionally_reached(property: &PropertyDefinition) -> bool {
    match property {
        PropertyDefinition::IdentifierReference(_) => false,
        PropertyDefinition::Property(name, value) => {
            property_name_await_is_conditionally_reached(name)
                || await_is_conditionally_reached(value)
        }
        PropertyDefinition::SpreadObject(source) => await_is_conditionally_reached(source),
        PropertyDefinition::MethodDefinition(method) => {
            property_name_await_is_conditionally_reached(method.name())
        }
        PropertyDefinition::CoverInitializedName(_, _) => true,
    }
}

fn property_name_await_is_conditionally_reached(name: &PropertyName) -> bool {
    match name {
        PropertyName::Literal(_) => false,
        PropertyName::Computed(key) => await_is_conditionally_reached(key),
    }
}
