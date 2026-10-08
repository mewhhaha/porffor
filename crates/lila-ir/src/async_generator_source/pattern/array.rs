use super::*;
use boa_ast::pattern::{ArrayPattern, ArrayPatternElement};

pub(crate) struct AsyncGeneratorArrayPatternSource<'ast> {
    pattern: &'ast ArrayPattern,
}

pub(crate) struct AsyncGeneratorArrayPatternSourceStates {
    entry: u32,
    body: AsyncGeneratorSourceRange,
    exit: u32,
    suspensions: Vec<ResumableSuspensionPointIr>,
    operations: Vec<(ArrayDestructuringOperationKindIr, u32)>,
}
impl AsyncGeneratorArrayPatternSourceStates {
    pub(crate) fn entry(&self) -> u32 {
        self.entry
    }
    pub(crate) fn body(&self) -> AsyncGeneratorSourceRange {
        self.body
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn suspensions(&self) -> &[ResumableSuspensionPointIr] {
        &self.suspensions
    }
    pub(crate) fn operations(&self) -> &[(ArrayDestructuringOperationKindIr, u32)] {
        &self.operations
    }
}
impl<'ast> AsyncGeneratorArrayPatternSource<'ast> {
    pub(crate) fn new(source: &'ast Pattern) -> Option<Self> {
        let Pattern::Array(pattern) = source else {
            return None;
        };
        let source = Self { pattern };
        source.states(0)?;
        Some(source)
    }
    pub(crate) fn pattern(&self) -> &'ast ArrayPattern {
        self.pattern
    }
    pub(crate) fn states(&self, entry: u32) -> Option<AsyncGeneratorArrayPatternSourceStates> {
        let mut states = ResumableStateAllocator::at(entry);
        let (body, operations) = append(self.pattern, &mut states).ok()?;
        let exit = states.current();
        let (suspensions, _) = states.into_tape();
        Some(AsyncGeneratorArrayPatternSourceStates {
            entry,
            body,
            exit,
            suspensions,
            operations,
        })
    }
}

pub(super) fn append(
    source: &ArrayPattern,
    states: &mut ResumableStateAllocator,
) -> Result<
    (
        AsyncGeneratorSourceRange,
        Vec<(ArrayDestructuringOperationKindIr, u32)>,
    ),
    AsyncGeneratorSourceError,
> {
    states.reserve()?;
    let entry = states.current();
    let mut operations = Vec::new();
    states.with_resume_environment(ResumableResumeEnvironmentIr::InvocationOuter, |states| {
        for element in source.bindings() {
            match element {
                ArrayPatternElement::Elision => {
                    operations.push((ArrayDestructuringOperationKindIr::Elision, states.current()))
                }
                ArrayPatternElement::SingleNameRest { .. } => operations.push((
                    ArrayDestructuringOperationKindIr::RestArray,
                    states.current(),
                )),
                ArrayPatternElement::SingleName { default_init, .. } => {
                    operations.push((
                        ArrayDestructuringOperationKindIr::StepValue,
                        states.current(),
                    ));
                    append_default(default_init.as_ref(), states)?;
                }
                ArrayPatternElement::PropertyAccess {
                    access,
                    default_init,
                } => {
                    append_target(access, states)?;
                    operations.push((
                        ArrayDestructuringOperationKindIr::StepValue,
                        states.current(),
                    ));
                    append_default(default_init.as_ref(), states)?;
                }
                ArrayPatternElement::Pattern {
                    pattern,
                    default_init,
                } => {
                    operations.push((
                        ArrayDestructuringOperationKindIr::StepValue,
                        states.current(),
                    ));
                    append_default(default_init.as_ref(), states)?;
                    super::append(pattern, states)?;
                }
                ArrayPatternElement::PropertyAccessRest { access } => {
                    append_target(access, states)?;
                    operations.push((
                        ArrayDestructuringOperationKindIr::RestArray,
                        states.current(),
                    ));
                }
                ArrayPatternElement::PatternRest { pattern } => {
                    operations.push((
                        ArrayDestructuringOperationKindIr::RestArray,
                        states.current(),
                    ));
                    super::append(pattern, states)?;
                }
            }
        }
        Ok(())
    })?;
    let body = AsyncGeneratorSourceRange {
        entry,
        end: states.current(),
    };
    states.reserve()?;
    Ok((body, operations))
}
