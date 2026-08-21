use super::*;

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

pub(crate) enum StaticStringGeneratorLoopBody {
    FromCharCode,
    FromCharCodeUnlessRegexpMatch(Regex),
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
        StatementIr::GeneratorYield { .. } | StatementIr::AsyncAwait { .. }
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

pub(crate) fn async_generator_resumable_plan(body: &FunctionBody) -> ResumablePlanIr {
    let mut collector = AsyncGeneratorSuspensionCollector::default();
    let _ = body.visit_with(&mut collector);
    collector.states.finish()
}

/// Why a generator body has no linear suspension plan.
///
/// # Why this exists
///
/// [`linear_generator_plan`] answered `Option`, and its single consumer in
/// `lower_declaration` reported the rejection as
/// `unsupported("function or class declaration")`. That string is wrong twice:
/// the declaration is a *generator*, not a function or class, and the reason is
/// the shape of its yields, not the kind of declaration. Every generator this
/// compiler refuses therefore collapsed into one `detail_hash` shared with
/// genuinely unrelated declarations, which is the worst possible outcome for a
/// sweep whose whole purpose is grouping failures into families.
///
/// Measured example, and the pair this was written against:
/// `annexB/built-ins/RegExp/RegExp-control-escape-russian-letter.js` and
/// `RegExp-invalid-control-escape-character-class.js` both declare
///
/// ```ignore
/// function* invalidControls() {
///   for (var alpha = 0x0410; alpha <= 0x042F; alpha++) { yield String.fromCharCode(alpha); }
///   // ... and then a third loop whose yield is nested inside an `if`:
///   for (alpha = 0x00; alpha <= 0x7F; alpha++) {
///     let letter = String.fromCharCode(alpha);
///     if (!letter.match(/[0-9A-Za-z_\$(|)\[\]\/\\^]/)) { yield letter; }
///   }
/// }
/// ```
///
/// and both reported `unsupported in lila wasm-aot first slice: function or
/// class declaration`. The actual refusal is [`Self::LoopBodyYieldNotDirect`],
/// raised by `simple_generator_loop_body_is_supported` on the third loop: its
/// body's `if` is a statement that *contains* a yield without *being* one.
///
/// # Scope
///
/// This is a diagnostic type only. It changes no acceptance decision: the set of
/// bodies [`linear_generator_plan`] accepts is byte-for-byte the set it accepted
/// before, because the wrapper is `.ok()` over the same walk. Widening the walk
/// to count yields per *path* rather than per statement list is filed as a
/// batch-8 candidate and is deliberately not done here —
/// `simple_generator_loop_body_is_supported`'s own comment records that
/// accepting a body whose block carries a lexical environment does not fail
/// loudly, it produces a loop the generator dispatcher cannot re-enter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeneratorPlanRejection {
    /// A nested declaration (a function, class or lexical declaration at the top
    /// level of the body) contains a yield.
    YieldInDeclaration,
    /// A statement-level `yield`, or `x = yield …`, whose operand itself hides
    /// further suspensions the linear walk cannot order.
    YieldOperandNotDirect,
    /// `return <expression containing a yield>` whose expression cannot be
    /// staged into a sequence of suspensions.
    ReturnOperandNotStageable,
    /// An expression statement whose value is discarded but whose yields cannot
    /// be flattened into a sequence.
    DiscardedYieldExpression,
    /// A bare block whose yields cannot be flattened into a sequence.
    DiscardedYieldBlock,
    /// `with (<expression containing a yield>)`.
    YieldInWithHead,
    /// A `with` body that is neither a block nor an expression statement and
    /// contains a yield.
    YieldInWithBody,
    /// A `for`/`while` body that is not exactly one direct `yield` statement —
    /// most often because the yield sits inside an `if`, a nested block or a
    /// `try` within the body, or because the body yields more than once.
    LoopBodyYieldNotDirect,
    /// A `for`/`while` carrying `break`, `continue`, or a nested function that
    /// would capture a per-iteration binding.
    LoopControlFlow,
    /// `if (<condition containing a yield>)`.
    YieldInIfCondition,
    /// An `if` branch whose yields are not a countable direct sequence.
    IfBranchYieldNotDirect,
    /// A `try`, `catch` or `finally` block whose yields are not a countable
    /// direct sequence.
    YieldInTryStatement,
    /// Any other statement kind that contains a yield: `switch`, labelled
    /// statements, `do`-`while`, `for`-`in`, `for`-`of`, and so on.
    YieldInUnsupportedStatement,
}

impl GeneratorPlanRejection {
    /// The reported reason. Every message names the *generator body* and the
    /// shape that failed, so a sweep groups these into families instead of into
    /// one bucket labelled "function or class declaration".
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::YieldInDeclaration => {
                "generator body: a nested declaration contains a yield, which has no linear \
                 suspension plan"
            }
            Self::YieldOperandNotDirect => {
                "generator body: a yield operand hides further suspensions, which has no linear \
                 suspension plan"
            }
            Self::ReturnOperandNotStageable => {
                "generator body: a `return` operand containing a yield cannot be staged into a \
                 linear suspension plan"
            }
            Self::DiscardedYieldExpression => {
                "generator body: a discarded expression statement's yields cannot be flattened \
                 into a linear suspension plan"
            }
            Self::DiscardedYieldBlock => {
                "generator body: a block's yields cannot be flattened into a linear suspension \
                 plan"
            }
            Self::YieldInWithHead => {
                "generator body: a yield in a `with` head has no linear suspension plan"
            }
            Self::YieldInWithBody => {
                "generator body: a yield in this `with` body shape has no linear suspension plan"
            }
            Self::LoopBodyYieldNotDirect => {
                "generator body: a loop body whose yield is nested inside another statement \
                 (an `if`, a block or a `try`), or which yields more than once, has no linear \
                 suspension plan"
            }
            Self::LoopControlFlow => {
                "generator body: a loop carrying `break`, `continue` or a capturing nested \
                 function has no linear suspension plan"
            }
            Self::YieldInIfCondition => {
                "generator body: a yield in an `if` condition has no linear suspension plan"
            }
            Self::IfBranchYieldNotDirect => {
                "generator body: an `if` branch whose yields are not a direct sequence has no \
                 linear suspension plan"
            }
            Self::YieldInTryStatement => {
                "generator body: a `try`/`catch`/`finally` block whose yields are not a direct \
                 sequence has no linear suspension plan"
            }
            Self::YieldInUnsupportedStatement => {
                "generator body: a yield inside a statement kind with no resumable lowering \
                 (`switch`, a label, `do`-`while`, `for`-`in`, `for`-`of`) has no linear \
                 suspension plan"
            }
        }
    }
}

/// The acceptance answer, unchanged. Every existing caller asks only whether a
/// plan exists; only `lower_declaration`'s rejection arm needs the reason, and
/// it calls [`linear_generator_plan_with_reason`] directly. Keeping this a thin
/// `.ok()` wrapper is what makes "the accepted set did not move" a property of
/// the code rather than a claim in a note.
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
                return Err(GeneratorPlanRejection::YieldInDeclaration);
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
                direct_generator_yield_count(yield_expression.target(), nested_yield_allowed)
                    .ok_or(GeneratorPlanRejection::YieldOperandNotDirect)?;
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
                let yield_count = staged_generator_expression_yield_count(target)
                    .ok_or(GeneratorPlanRejection::ReturnOperandNotStageable)?;
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
                )
                .ok_or(GeneratorPlanRejection::DiscardedYieldExpression)?;
                continue;
            }
        }
        if let Statement::Block(block) = statement.as_ref() {
            if contains(block, ContainsSymbol::YieldExpression) {
                append_discarded_generator_block_suspensions(
                    block.statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )
                .ok_or(GeneratorPlanRejection::DiscardedYieldBlock)?;
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
                )
                .ok_or(GeneratorPlanRejection::DiscardedYieldBlock)?,
                Statement::Expression(expression) => {
                    append_discarded_generator_expression_suspensions(
                        expression,
                        &mut current_state,
                        &mut suspension_points,
                    )
                    .ok_or(GeneratorPlanRejection::DiscardedYieldExpression)?;
                }
                statement if contains(statement, ContainsSymbol::YieldExpression) => {
                    return Err(GeneratorPlanRejection::YieldInWithBody)
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
            // Split into two answers rather than one `||`. This is the arm the
            // two `annexB` `invalidControls` cases take, and "the loop body's
            // yield is nested inside an `if`" and "the loop carries a break" are
            // different families with different fixes.
            if !simple_generator_loop_body_is_supported(loop_body) {
                return Err(GeneratorPlanRejection::LoopBodyYieldNotDirect);
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
            let then_yields = simple_generator_if_branch_yield_count(if_statement.body())
                .ok_or(GeneratorPlanRejection::IfBranchYieldNotDirect)?;
            let else_yields = match if_statement.else_node() {
                Some(branch) => simple_generator_if_branch_yield_count(branch)
                    .ok_or(GeneratorPlanRejection::IfBranchYieldNotDirect)?,
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
            )
            .ok_or(GeneratorPlanRejection::YieldInTryStatement)?;
            current_state += 1;
            if let Some(catch) = try_statement.catch() {
                append_structured_generator_suspensions(
                    catch.block().statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )
                .ok_or(GeneratorPlanRejection::YieldInTryStatement)?;
                current_state += 1;
            }
            if let Some(finally) = try_statement.finally() {
                append_structured_generator_suspensions(
                    finally.block().statement_list().statements(),
                    &mut current_state,
                    &mut suspension_points,
                )
                .ok_or(GeneratorPlanRejection::YieldInTryStatement)?;
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

/// Whether a `for`-`of` head can take the resumable array-index walk
/// (`ScriptLowerer::lower_async_for_of_array_with_body_await`), and if not, the
/// one premise that failed.
///
/// # Why this is a closed type and not the `bool` it replaces
///
/// It used to be `has_unsupported_binding_form: bool`, computed as
///
/// ```ignore
/// !for_of_environment_is_storage_only(lexical_environment.as_ref())
///     || pattern_initializer.is_some()
///     || assignment_pattern_initializer.is_some()
///     || access_initializer.is_some()
/// ```
///
/// and OR-ed at the use site with an array-typing test and an
/// `Option<u32>` entry state, so **four** independent premises produced one
/// message: `async for-of with a body await requires an array iterable and a
/// plain binding`.
///
/// That message is wrong for the family it actually rejects. Measured on the
/// batch-7 baseline sweep, `built-ins/Array/fromAsync` is 95 cases, 93 passed,
/// and both failures carry that string under one `detail_hash`
/// (`10438609855492019567`):
///
/// ```ignore
/// asyncTest(async function () {
///   for (const v of [true, "", Symbol(), 1, 1n, {}]) {
///     await assert.throwsAsync(TypeError,
///       () => Array.fromAsync({ [Symbol.asyncIterator]: v }),
///       `@@asyncIterator = ${typeof v}`);
///   }
/// });
/// ```
///
/// The iterable IS an array literal and the binding IS a plain `const v`. Both
/// stated premises hold. The real disqualifier is the third, unnamed one: the
/// arrow captures `v`, so ECMA-262 14.7.5.7 per-iteration bindings materialise
/// an *iteration environment*, and this specialization cannot reproduce one.
/// Reproduced directly at the CLI on the same head:
///
/// ```ignore
/// async function f() {
///   const out = [];
///   for (const v of [1, 2, 3]) { out.push(() => v); await 0; }
/// }
/// // -> unsupported in lila wasm-aot first slice: async for-of with a body
/// //    await requires an array iterable and a plain binding
/// ```
///
/// A wrong reason is worse than no reason: it sent triage at an
/// iterable-typing problem that was not there. Each variant below names exactly
/// one premise, and [`Self::into_plan`] must either produce the closed runtime
/// plan or a message, so a future variant cannot be added without stating what
/// it means.
///
/// # What each variant costs to lift
///
/// `PlainStorageOnly` covers both "no head environment at all" and "storage
/// slots only, no runtime environment object". Both are reproducible here: the
/// lowering declares those slots as `StatementIr::Lexical` bindings, which every
/// loop compiler marks uninitialized before running the loop init — the same
/// observable TDZ (14.7.5.5 ForIn/OfHeadEvaluation, 8.6.2).
///
/// `CapturedPerIterationBinding` is liftable only by carrying the analyzed
/// environment through `StatementIr::GeneratorLoop`. Its backend must allocate
/// a fresh record for every entered iteration, persist the active pointer across
/// suspension, and restore the parent before the update or next test. Hoisting
/// the binding into one activation slot instead would give every closure the
/// *same* cell and silently break per-iteration semantics. See
/// `docs/rust-rewrite/contracts/resumable-loop-per-iteration-environment.md`.
/// # Why the iterable's type is a variant here and not a test at the call site
///
/// Two of the messages below assert that the *iterable type* is fine, which was
/// once a premise this type could not see: it was tested separately at the sole
/// call site, and the doc here carried a prose "caller obligation" saying that
/// site must test typing **first**. That is exactly the substitution AGENTS.md's
/// "Code Invariants Before Test Invariants" warns against — a second call site
/// would have compiled cleanly while emitting a message asserting a premise it
/// never checked, in a type whose entire thesis is that a wrong reason routes
/// triage away from the defect.
///
/// So the premise moved into the type. [`Self::NonArrayIterable`] is tested
/// first inside [`Self::classify`], which makes the load-bearing order
/// unrepresentable-wrong rather than documented: `for (const c of "ab") { f = ()
/// => c; await 0; }` is captured **and** non-array at once, and answering
/// "captured" for it would certify a String iterable as fine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncForOfArrayWalkForm {
    /// One plain storage name over an array iterable, and no runtime
    /// environment record to reproduce.
    PlainStorageOnly,
    /// The iterable can be something other than an array, so the walk would
    /// have to be the `@@iterator` protocol instead of an index walk. Tested
    /// first, because it is the cheapest and most universal premise.
    NonArrayIterable,
    /// The head destructures, or assigns into a property/member target, so the
    /// per-iteration value is not one storage name.
    DestructuringOrPropertyTarget,
    /// A closure in the body captures the loop binding, so 14.7.5.7 requires a
    /// fresh environment record per iteration.
    CapturedPerIterationBinding(LexicalEnvironmentIr),
    /// A closure captures a name the head puts in TDZ while the iterable is
    /// evaluated, so the TDZ scope needs a real environment record too. This is
    /// a different problem from the iteration environment and is left rejected
    /// until it is measured on its own.
    CapturedTdzBinding,
}

impl AsyncForOfArrayWalkForm {
    /// The only constructor.
    ///
    /// `iterable_is_array` is the `KindSet` subset test on the iterable, passed
    /// in rather than assumed — see the type's doc for why it is a parameter and
    /// not a caller obligation. `binds_pattern_or_property_target` is the
    /// disjunction of the head's three non-identifier initializer forms
    /// (`pattern_initializer`, `assignment_pattern_initializer`,
    /// `access_initializer`).
    ///
    /// The arm order is the classification order: typing, then binding shape,
    /// then captured environments. Do not reorder — the TDZ-capture error
    /// states that the array and plain-binding premises above it hold.
    pub(crate) fn classify(
        iterable_is_array: bool,
        environment: Option<&ForInOfEnvironmentIr>,
        binds_pattern_or_property_target: bool,
    ) -> Self {
        if !iterable_is_array {
            return Self::NonArrayIterable;
        }
        if binds_pattern_or_property_target {
            return Self::DestructuringOrPropertyTarget;
        }
        let Some(environment) = environment else {
            return Self::PlainStorageOnly;
        };
        if let Some(iteration_environment) = &environment.iteration_environment {
            return Self::CapturedPerIterationBinding(iteration_environment.clone());
        }
        if environment.tdz_environment.is_some() {
            return Self::CapturedTdzBinding;
        }
        Self::PlainStorageOnly
    }

    /// Converts every supported source shape into the required closed IR plan,
    /// or returns the one failed premise. A captured iteration binding is a
    /// supported plan only because the variant owns its analyzed environment;
    /// there is no path that can lose the layout and silently select storage.
    pub(crate) fn into_plan(self) -> Result<ResumableLoopIterationEnvironmentIr, &'static str> {
        match self {
            Self::PlainStorageOnly => Ok(ResumableLoopIterationEnvironmentIr::StorageOnly),
            Self::CapturedPerIterationBinding(environment) => Ok(
                ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment),
            ),
            Self::NonArrayIterable => Err(
                "async for-of with a body await requires an array iterable, and this one \
                 can be something else; every other iterable keeps the @@iterator protocol, \
                 whose own suspension points this index walk does not have",
            ),
            Self::DestructuringOrPropertyTarget => Err(
                "async for-of with a body await requires a plain single-name binding, \
                 and this head destructures or assigns into a property target",
            ),
            Self::CapturedTdzBinding => Err(
                "async for-of with a body await cannot materialize the head's TDZ \
                 environment record, and a closure captures a name the head puts in TDZ; \
                 the iterable is an array and the head binds one plain name",
            ),
        }
    }
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
