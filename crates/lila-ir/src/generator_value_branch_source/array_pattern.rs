//! Array patterns own acquisition, the complete body and IteratorClose exit.

use super::*;
use crate::generator_loop_control::GeneratorLoopSourceRange;
use crate::ArrayDestructuringOperationKindIr;
use boa_ast::pattern::{ArrayPattern, ArrayPatternElement};

pub(crate) struct GeneratorArrayPatternSource<'ast> {
    pattern: &'ast ArrayPattern,
}

/// The source's acquisition is outside its own close scope. Every body,
/// including an eager nested body, has a distinct entry and terminal state.
pub(crate) struct GeneratorArrayPatternSourceStates {
    entry: u32,
    body: GeneratorLoopSourceRange,
    exit: u32,
    suspensions: Vec<GeneratorSuspensionPointIr>,
    operations: Vec<(ArrayDestructuringOperationKindIr, u32)>,
}

impl GeneratorArrayPatternSourceStates {
    pub(crate) fn entry(&self) -> u32 {
        self.entry
    }
    pub(crate) fn body(&self) -> GeneratorLoopSourceRange {
        self.body
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn suspensions(&self) -> &[GeneratorSuspensionPointIr] {
        &self.suspensions
    }
    pub(crate) fn operations(&self) -> &[(ArrayDestructuringOperationKindIr, u32)] {
        &self.operations
    }
}

impl<'ast> GeneratorArrayPatternSource<'ast> {
    pub(crate) fn new(pattern: &'ast Pattern) -> Option<Self> {
        let Pattern::Array(pattern) = pattern else {
            return None;
        };
        let source = Self { pattern };
        source.states(0)?;
        Some(source)
    }

    pub(crate) fn pattern(&self) -> &'ast ArrayPattern {
        self.pattern
    }

    pub(crate) fn states(&self, entry: u32) -> Option<GeneratorArrayPatternSourceStates> {
        if contains(self.pattern, ContainsSymbol::AwaitExpression) {
            return None;
        }
        let body_entry = entry.checked_add(1)?;
        let mut cursor = body_entry;
        let mut suspensions = Vec::new();
        let mut operations = Vec::new();
        for element in self.pattern.bindings() {
            match element {
                ArrayPatternElement::Elision => {
                    operations.push((ArrayDestructuringOperationKindIr::Elision, cursor));
                }
                ArrayPatternElement::SingleNameRest { .. } => {
                    operations.push((ArrayDestructuringOperationKindIr::RestArray, cursor));
                }
                ArrayPatternElement::SingleName { default_init, .. } => {
                    operations.push((ArrayDestructuringOperationKindIr::StepValue, cursor));
                    append_default(default_init.as_ref(), &mut cursor, &mut suspensions)?;
                }
                ArrayPatternElement::PropertyAccess {
                    access,
                    default_init,
                } => {
                    append_target(access, &mut cursor, &mut suspensions)?;
                    operations.push((ArrayDestructuringOperationKindIr::StepValue, cursor));
                    append_default(default_init.as_ref(), &mut cursor, &mut suspensions)?;
                }
                ArrayPatternElement::Pattern {
                    pattern,
                    default_init,
                } => {
                    operations.push((ArrayDestructuringOperationKindIr::StepValue, cursor));
                    append_default(default_init.as_ref(), &mut cursor, &mut suspensions)?;
                    append_nested(pattern, &mut cursor, &mut suspensions)?;
                }
                ArrayPatternElement::PropertyAccessRest { access } => {
                    append_target(access, &mut cursor, &mut suspensions)?;
                    operations.push((ArrayDestructuringOperationKindIr::RestArray, cursor));
                }
                ArrayPatternElement::PatternRest { pattern } => {
                    operations.push((ArrayDestructuringOperationKindIr::RestArray, cursor));
                    append_nested(pattern, &mut cursor, &mut suspensions)?;
                }
            }
        }
        Some(GeneratorArrayPatternSourceStates {
            entry,
            body: GeneratorLoopSourceRange {
                entry: body_entry,
                end: cursor,
            },
            exit: cursor.checked_add(1)?,
            suspensions,
            operations,
        })
    }

    pub(crate) fn append(
        &self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        let states = self.states(*cursor)?;
        *cursor = states.exit();
        points.extend(states.suspensions);
        Some(())
    }

    /// Recursion consumes the same actual planner without repeating the whole
    /// child preflight at every parent level.
    pub(crate) fn append_from_pattern(
        pattern: &'ast Pattern,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        let Pattern::Array(pattern) = pattern else {
            return None;
        };
        Self { pattern }.append(cursor, points)
    }
}

fn append_expression(
    source: &Expression,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    GeneratorExpressionSourcePlan::new(source, GeneratorValueBranchAdmission::OrdinaryOutsideLoops)?
        .append(cursor, points)
}

fn append_target(
    source: &PropertyAccess,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match source {
        PropertyAccess::Simple(access) => {
            append_expression(access.target(), cursor, points)?;
            match access.field() {
                PropertyAccessField::Const(_) => Some(()),
                PropertyAccessField::Expr(source) => append_expression(source, cursor, points),
            }
        }
        PropertyAccess::Private(access) => append_expression(access.target(), cursor, points),
        PropertyAccess::Super(access) => match access.field() {
            PropertyAccessField::Const(_) => Some(()),
            PropertyAccessField::Expr(source) => append_expression(source, cursor, points),
        },
    }
}

fn append_default(
    source: Option<&Expression>,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    if let Some(source) = source {
        let states = GeneratorObjectPatternSource::default_states(source, *cursor)?;
        *cursor = states.exit();
        states.append_suspensions(points);
    }
    Some(())
}

fn append_nested(
    pattern: &Pattern,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match pattern {
        Pattern::Object(_) => {
            GeneratorObjectPatternSource::append_from_pattern(pattern, cursor, points)
        }
        Pattern::Array(_) => {
            if contains(pattern, ContainsSymbol::AwaitExpression) {
                return None;
            }
            if contains(pattern, ContainsSymbol::YieldExpression) {
                GeneratorArrayPatternSource::append_from_pattern(pattern, cursor, points)
            } else {
                Some(())
            }
        }
    }
}
