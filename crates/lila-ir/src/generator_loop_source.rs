use crate::generator_loop_control::{
    checked_next_state, GeneratorLoopSourceRange, GeneratorLoopSourceStates,
};
use crate::generator_value_branch_source::{
    GeneratorExpressionSourcePlan, GeneratorPatternInitializerSource, GeneratorValueBranchAdmission,
};
use crate::*;

/// This is the sole source walk allocating classic-loop phase/conditional and
/// try-clause states. The function admission walk and lowering consume it.
#[derive(Clone, Copy)]
pub(crate) enum ClassicGeneratorLoopSource<'ast> {
    For(&'ast ForLoop),
    While(&'ast WhileLoop),
    DoWhile(&'ast DoWhileLoop),
}

impl ClassicGeneratorLoopSource<'_> {
    pub(crate) fn plan(self, entry: u32) -> Option<GeneratorLoopSourceStates> {
        let mut cursor = entry;
        let mut suspensions = Vec::new();
        let (kind, initialization, test, body, update) = match self {
            Self::For(source) => {
                let initialization = phase(&mut cursor, |cursor| {
                    if let Some(init) = source.init() {
                        append_initializer(init, cursor, &mut suspensions)?;
                    }
                    Some(())
                })?;
                let test = phase(&mut cursor, |cursor| {
                    append_optional_expression(source.condition(), cursor, &mut suspensions)
                })?;
                let body = phase(&mut cursor, |cursor| {
                    append_classic_generator_statement(source.body(), cursor, &mut suspensions)
                })?;
                let update = phase(&mut cursor, |cursor| {
                    if let Some(source) = source.final_expr() {
                        crate::lowering_helpers::append_discarded_generator_expression_suspensions(
                            source,
                            cursor,
                            &mut suspensions,
                            GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
                        )?;
                    }
                    Some(())
                })?;
                (
                    GeneratorLoopKindIr::For,
                    Some(initialization),
                    test,
                    body,
                    Some(update),
                )
            }
            Self::While(source) => {
                let test = phase(&mut cursor, |cursor| {
                    append_expression(source.condition(), cursor, &mut suspensions)
                })?;
                let body = phase(&mut cursor, |cursor| {
                    append_classic_generator_statement(source.body(), cursor, &mut suspensions)
                })?;
                (GeneratorLoopKindIr::While, None, test, body, None)
            }
            Self::DoWhile(source) => {
                let body = phase(&mut cursor, |cursor| {
                    append_classic_generator_statement(source.body(), cursor, &mut suspensions)
                })?;
                let test = phase(&mut cursor, |cursor| {
                    append_expression(source.cond(), cursor, &mut suspensions)
                })?;
                (GeneratorLoopKindIr::DoWhile, None, test, body, None)
            }
        };
        Some(GeneratorLoopSourceStates {
            kind,
            initialization,
            test,
            body,
            update,
            exit: cursor,
            suspensions,
        })
    }

    pub(crate) fn append(
        self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        if let Self::For(source) = self {
            if let Some(source) =
                crate::async_generator_source::PlainAsyncClassicLoopSource::for_generator_resource(
                    source,
                )
            {
                return source.append_generator(cursor, points);
            }
        }
        let plan = self.plan(*cursor)?;
        points.extend(plan.suspensions);
        *cursor = plan.exit;
        Some(())
    }
}

fn phase(
    cursor: &mut u32,
    append: impl FnOnce(&mut u32) -> Option<()>,
) -> Option<GeneratorLoopSourceRange> {
    let entry = *cursor;
    append(cursor)?;
    let range = GeneratorLoopSourceRange {
        entry,
        end: *cursor,
    };
    *cursor = checked_next_state(*cursor).ok()?;
    Some(range)
}

fn append_optional_expression(
    source: Option<&Expression>,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    if let Some(source) = source {
        append_expression(source, cursor, points)?;
    }
    Some(())
}

fn append_expression(
    source: &Expression,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    GeneratorExpressionSourcePlan::new(source, GeneratorValueBranchAdmission::OrdinaryOutsideLoops)?
        .append(cursor, points)
}

fn append_initializer(
    source: &ForLoopInitializer,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match source {
        ForLoopInitializer::Expression(source) => {
            crate::lowering_helpers::append_discarded_generator_expression_suspensions(
                source,
                cursor,
                points,
                GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
            )
        }
        ForLoopInitializer::Var(source) => {
            if contains(source, ContainsSymbol::YieldExpression) {
                GeneratorExpressionSourcePlan::var_declaration(source)?.append(cursor, points)?;
            }
            Some(())
        }
        ForLoopInitializer::Lexical(source) => match source.declaration() {
            LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => {
                for variable in list.as_ref() {
                    if contains(variable.binding(), ContainsSymbol::AwaitExpression) {
                        return None;
                    }
                    if let Some(initializer) = variable.init() {
                        if matches!(variable.binding(), Binding::Pattern(_))
                            && (contains(initializer, ContainsSymbol::YieldExpression)
                                || contains(variable.binding(), ContainsSymbol::YieldExpression))
                        {
                            let source = GeneratorPatternInitializerSource::new(
                                variable,
                                GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
                            )?;
                            source.append(cursor, points)?;
                        } else {
                            append_expression(initializer, cursor, points)?;
                        }
                    }
                }
                Some(())
            }
            LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_) => None,
        },
    }
}

pub(crate) struct ClassicGeneratorIfSourceStates {
    pub(crate) condition: GeneratorLoopSourceRange,
    pub(crate) then_branch: GeneratorLoopSourceRange,
    pub(crate) else_branch: GeneratorLoopSourceRange,
    pub(crate) exit: u32,
    suspensions: Vec<GeneratorSuspensionPointIr>,
}

pub(crate) fn classic_generator_if_states(
    source: &If,
    entry: u32,
) -> Option<ClassicGeneratorIfSourceStates> {
    let mut cursor = entry;
    let mut suspensions = Vec::new();
    let condition = phase(&mut cursor, |cursor| {
        append_expression(source.cond(), cursor, &mut suspensions)
    })?;
    let then_branch = phase(&mut cursor, |cursor| {
        append_classic_generator_statement(source.body(), cursor, &mut suspensions)
    })?;
    let else_branch = phase(&mut cursor, |cursor| {
        if let Some(source) = source.else_node() {
            append_classic_generator_statement(source, cursor, &mut suspensions)?;
        }
        Some(())
    })?;
    Some(ClassicGeneratorIfSourceStates {
        condition,
        then_branch,
        else_branch,
        exit: cursor,
        suspensions,
    })
}

pub(crate) fn append_classic_generator_items(
    items: &[StatementListItem],
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    if crate::async_generator_source::AsyncGeneratorResourceScopeSource::for_protocol(
        items,
        ResumableRegionProtocolIr::Generator,
    )
    .is_some()
    {
        let mut typed = Vec::new();
        crate::async_generator_source::append_resource_scope_for_protocol(
            items,
            ResumableRegionProtocolIr::Generator,
            cursor,
            &mut typed,
        )?;
        points.extend(typed.into_iter().map(|point| GeneratorSuspensionPointIr {
            suspend_state: point.suspend_state,
            resume_state: point.resume_state,
        }));
        return Some(());
    }
    for item in items {
        match item {
            StatementListItem::Declaration(_) => {
                if contains(item, ContainsSymbol::YieldExpression) {
                    GeneratorExpressionSourcePlan::declaration(
                        item,
                        GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
                    )?
                    .append(cursor, points)?;
                }
            }
            StatementListItem::Statement(statement) => {
                append_classic_generator_statement(statement, cursor, points)?
            }
        }
    }
    Some(())
}

/// Catch BindingInitialization precedes its actual Block and consumes the
/// same checked pattern tape as a suspended declaration or assignment.
pub(crate) fn append_generator_catch_parameter(
    parameter: Option<&Binding>,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    let Some(Binding::Pattern(pattern)) = parameter else {
        return Some(());
    };
    if !contains(pattern, ContainsSymbol::YieldExpression) {
        return Some(());
    }
    match pattern {
        Pattern::Array(_) => {
            crate::generator_value_branch_source::GeneratorArrayPatternSource::new(pattern)?
                .append(cursor, points)
        }
        Pattern::Object(_) => {
            crate::generator_value_branch_source::GeneratorObjectPatternSource::new(pattern)?
                .append(cursor, points)
        }
    }
}

pub(crate) fn append_classic_generator_statement(
    source: &Statement,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match source {
        Statement::Expression(source) => {
            // Discarded composites use the real source/lowering authority,
            // including expressions whose value staging has a narrower shape.
            crate::lowering_helpers::append_discarded_generator_expression_suspensions(
                source,
                cursor,
                points,
                GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
            )
        }
        Statement::Var(source) => {
            if contains(source, ContainsSymbol::YieldExpression) {
                GeneratorExpressionSourcePlan::var_declaration(source)?.append(cursor, points)?;
            }
            Some(())
        }
        Statement::Return(source) => append_optional_expression(source.target(), cursor, points),
        Statement::Throw(source) => append_expression(source.target(), cursor, points),
        Statement::Block(source) => {
            append_classic_generator_items(source.statement_list().statements(), cursor, points)
        }
        Statement::ForLoop(source) => {
            ClassicGeneratorLoopSource::For(source).append(cursor, points)
        }
        Statement::WhileLoop(source) => {
            ClassicGeneratorLoopSource::While(source).append(cursor, points)
        }
        Statement::DoWhileLoop(source) => {
            ClassicGeneratorLoopSource::DoWhile(source).append(cursor, points)
        }
        Statement::If(source) => {
            let states = classic_generator_if_states(source, *cursor)?;
            points.extend(states.suspensions);
            *cursor = states.exit;
            Some(())
        }
        Statement::Switch(source) => {
            if contains(source, ContainsSymbol::YieldExpression)
                || crate::lowering_helpers::contains_ordinary_generator_phase_owner(source)
            {
                append_generator_switch_suspensions(source, cursor, points)
            } else {
                Some(())
            }
        }
        Statement::Try(source) => {
            append_classic_generator_items(
                source.block().statement_list().statements(),
                cursor,
                points,
            )?;
            *cursor = checked_next_state(*cursor).ok()?;
            if let Some(source) = source.catch() {
                append_generator_catch_parameter(source.parameter(), cursor, points)?;
                append_classic_generator_items(
                    source.block().statement_list().statements(),
                    cursor,
                    points,
                )?;
                *cursor = checked_next_state(*cursor).ok()?;
            }
            if let Some(source) = source.finally() {
                append_classic_generator_items(
                    source.block().statement_list().statements(),
                    cursor,
                    points,
                )?;
                *cursor = checked_next_state(*cursor).ok()?;
            }
            Some(())
        }
        Statement::Labelled(source) => match source.item() {
            LabelledItem::Statement(
                source @ (Statement::ForLoop(_)
                | Statement::WhileLoop(_)
                | Statement::DoWhileLoop(_)
                | Statement::Switch(_)
                | Statement::Labelled(_)
                | Statement::With(_)),
            ) => append_classic_generator_statement(source, cursor, points),
            LabelledItem::Statement(source)
                if crate::lowering_helpers::contains_ordinary_generator_phase_owner(source) =>
            {
                append_classic_generator_statement(source, cursor, points)
            }
            LabelledItem::Statement(source)
                if !contains(source, ContainsSymbol::YieldExpression) =>
            {
                Some(())
            }
            LabelledItem::Statement(_) | LabelledItem::FunctionDeclaration(_) => None,
        },
        Statement::With(source) => {
            crate::lowering_helpers::GeneratorWithSource::new(source)?.append(cursor, points)
        }
        Statement::ForInLoop(source) => {
            crate::lowering_helpers::append_generator_for_in_suspensions(source, cursor, points)
        }
        Statement::ForOfLoop(source) => {
            crate::lowering_helpers::append_generator_for_of_suspensions(source, cursor, points)
        }
        Statement::Empty | Statement::Break(_) | Statement::Continue(_) | Statement::Debugger => {
            Some(())
        }
    }
}
