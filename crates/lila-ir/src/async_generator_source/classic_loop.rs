use super::*;

#[derive(Clone, Copy)]
pub(crate) enum AsyncGeneratorClassicLoopSource<'ast> {
    For(&'ast ForLoop),
    While(&'ast WhileLoop),
    DoWhile(&'ast DoWhileLoop),
}
impl<'ast> AsyncGeneratorClassicLoopSource<'ast> {
    pub(crate) fn for_loop(source: &'ast ForLoop) -> Option<Self> {
        Self::For(source).checked()
    }
    pub(crate) fn while_loop(source: &'ast WhileLoop) -> Option<Self> {
        Self::While(source).checked()
    }
    pub(crate) fn do_while(source: &'ast DoWhileLoop) -> Option<Self> {
        Self::DoWhile(source).checked()
    }
    fn checked(self) -> Option<Self> {
        self.states(0)?;
        Some(self)
    }
    pub(crate) fn states(self, entry: u32) -> Option<AsyncGeneratorLoopSourceStates> {
        let mut states = ResumableStateAllocator::at(entry);
        let (kind, initialization, test, body, update, resource) =
            self.append_phases(&mut states).ok()?;
        let exit = states.current();
        let (suspensions, _) = states.into_tape();
        Some(AsyncGeneratorLoopSourceStates {
            execution: ResumableRegionProtocolIr::AsyncGenerator,
            kind,
            initialization,
            test,
            body,
            update,
            resource,
            exit,
            suspensions,
        })
    }
    pub(super) fn append(
        self,
        states: &mut ResumableStateAllocator,
    ) -> Result<(), AsyncGeneratorSourceError> {
        self.append_phases(states).map(|_| ())
    }

    fn append_phases(
        self,
        states: &mut ResumableStateAllocator,
    ) -> Result<LoopPhases, AsyncGeneratorSourceError> {
        let mut registrations = Vec::new();
        let (kind, initialization, test, body, update) = match self {
            Self::For(source) => {
                let initialization = phase(states, |states| {
                    append_initializer(source.init(), states, &mut registrations)
                })?;
                let test = phase(states, |states| append_optional(source.condition(), states))?;
                let body = phase(states, |states| {
                    statement::append(
                        source.body(),
                        states,
                        AsyncGeneratorSourceDomain::FunctionBody,
                    )
                })?;
                let update = phase(states, |states| {
                    append_optional(source.final_expr(), states)
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
                let test = phase(states, |states| {
                    expression::append(source.condition(), states)
                })?;
                let body = phase(states, |states| {
                    statement::append(
                        source.body(),
                        states,
                        AsyncGeneratorSourceDomain::FunctionBody,
                    )
                })?;
                (GeneratorLoopKindIr::While, None, test, body, None)
            }
            Self::DoWhile(source) => {
                let body = phase(states, |states| {
                    statement::append(
                        source.body(),
                        states,
                        AsyncGeneratorSourceDomain::FunctionBody,
                    )
                })?;
                let test = phase(states, |states| expression::append(source.cond(), states))?;
                (GeneratorLoopKindIr::DoWhile, None, test, body, None)
            }
        };
        let resource = resource::finish_scoped(
            initialization.map_or(test.entry(), |range| range.entry()),
            update.map_or_else(
                || {
                    if kind == GeneratorLoopKindIr::DoWhile {
                        test.end()
                    } else {
                        body.end()
                    }
                },
                |range| range.end(),
            ),
            registrations,
            states,
        )?;
        Ok((kind, initialization, test, body, update, resource))
    }
}

type LoopPhases = (
    GeneratorLoopKindIr,
    Option<AsyncGeneratorSourceRange>,
    AsyncGeneratorSourceRange,
    AsyncGeneratorSourceRange,
    Option<AsyncGeneratorSourceRange>,
    Option<AsyncGeneratorScopedResourceSourceStates>,
);

fn append_optional(
    source: Option<&Expression>,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    if let Some(source) = source {
        expression::append(source, states)?;
    }
    Ok(())
}
fn append_initializer(
    source: Option<&ForLoopInitializer>,
    states: &mut ResumableStateAllocator,
    registrations: &mut Vec<AsyncGeneratorResourceRegistrationSource>,
) -> Result<(), AsyncGeneratorSourceError> {
    match source {
        None => Ok(()),
        Some(ForLoopInitializer::Expression(source)) => expression::append(source, states),
        Some(ForLoopInitializer::Var(source)) => {
            statement::append_variables(source.0.as_ref(), states)
        }
        Some(ForLoopInitializer::Lexical(source)) => match source.declaration() {
            LexicalDeclaration::Let(list) | LexicalDeclaration::Const(list) => {
                statement::append_variables(list.as_ref(), states)
            }
            LexicalDeclaration::Using(list) => resource::append_registration_variables(
                ResourceDisposalHintIr::Sync,
                list.as_ref(),
                states,
                registrations,
            ),
            LexicalDeclaration::AwaitUsing(list) => resource::append_registration_variables(
                ResourceDisposalHintIr::Async,
                list.as_ref(),
                states,
                registrations,
            ),
        },
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AsyncGeneratorIfSource<'ast> {
    source: &'ast If,
}
impl<'ast> AsyncGeneratorIfSource<'ast> {
    pub(crate) fn new(source: &'ast If) -> Option<Self> {
        let owner = Self { source };
        owner.states(0)?;
        Some(owner)
    }
    pub(crate) fn source(self) -> &'ast If {
        self.source
    }
    pub(crate) fn states(self, entry: u32) -> Option<AsyncGeneratorIfSourceStates> {
        let mut states = ResumableStateAllocator::at(entry);
        let (condition, then_branch, else_branch) =
            append_if_phases(self.source, &mut states).ok()?;
        let exit = states.current();
        let (suspensions, enclosing_scope_resume_states) = states.into_tape();
        Some(AsyncGeneratorIfSourceStates {
            condition,
            then_branch,
            else_branch,
            exit,
            suspensions,
            enclosing_scope_resume_states,
        })
    }
}

pub(super) fn append_if(
    source: &If,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    append_if_phases(source, states).map(|_| ())
}

fn append_if_phases(
    source: &If,
    states: &mut ResumableStateAllocator,
) -> Result<
    (
        AsyncGeneratorSourceRange,
        AsyncGeneratorSourceRange,
        AsyncGeneratorSourceRange,
    ),
    AsyncGeneratorSourceError,
> {
    let condition = phase(states, |states| expression::append(source.cond(), states))?;
    let then_branch = phase(states, |states| {
        statement::append(
            source.body(),
            states,
            AsyncGeneratorSourceDomain::FunctionBody,
        )
    })?;
    let else_branch = phase(states, |states| {
        if let Some(source) = source.else_node() {
            statement::append(source, states, AsyncGeneratorSourceDomain::FunctionBody)?;
        }
        Ok(())
    })?;
    Ok((condition, then_branch, else_branch))
}
