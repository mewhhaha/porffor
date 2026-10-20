//! Actual object-pattern operands share one ordered suspension authority.

use super::*;
use boa_ast::pattern::{ObjectPattern, ObjectPatternElement};

/// Actual nested Arrays use their checked suspension owner when needed;
/// eager Arrays retain the ordinary IteratorRecord/IteratorClose consumer.
pub(crate) struct GeneratorObjectPatternSource<'ast> {
    pattern: &'ast ObjectPattern,
}

impl<'ast> GeneratorObjectPatternSource<'ast> {
    pub(crate) fn new(pattern: &'ast Pattern) -> Option<Self> {
        let Pattern::Object(pattern) = pattern else {
            return None;
        };
        if contains(pattern, ContainsSymbol::AwaitExpression) {
            return None;
        }
        let source = Self { pattern };
        // This is the same full plan subsequently consumed by publication.
        source.append(&mut 0, &mut Vec::new())?;
        Some(source)
    }

    pub(crate) fn pattern(&self) -> &'ast ObjectPattern {
        self.pattern
    }

    pub(crate) fn append_from_pattern(
        pattern: &'ast Pattern,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        let Pattern::Object(pattern) = pattern else {
            return None;
        };
        if contains(pattern, ContainsSymbol::AwaitExpression) {
            return None;
        }
        Self { pattern }.append(cursor, points)
    }

    pub(crate) fn default_states(
        source: &Expression,
        entry: u32,
    ) -> Option<GeneratorValueRegionStates> {
        GeneratorValueRegionStates::new(entry, Some(source), None)
    }

    pub(crate) fn append(
        &self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        for element in self.pattern.bindings() {
            match element {
                ObjectPatternElement::SingleName {
                    name, default_init, ..
                } => {
                    append_key(name, cursor, points)?;
                    append_default(default_init.as_ref(), cursor, points)?;
                }
                ObjectPatternElement::AssignmentPropertyAccess {
                    name,
                    access,
                    default_init,
                } => {
                    append_key(name, cursor, points)?;
                    append_target(access, cursor, points)?;
                    append_default(default_init.as_ref(), cursor, points)?;
                }
                ObjectPatternElement::Pattern {
                    name,
                    pattern,
                    default_init,
                } => {
                    append_key(name, cursor, points)?;
                    append_default(default_init.as_ref(), cursor, points)?;
                    match pattern {
                        Pattern::Object(pattern) => Self { pattern }.append(cursor, points)?,
                        Pattern::Array(_) => {
                            if contains(pattern, ContainsSymbol::AwaitExpression) {
                                return None;
                            }
                            if contains(pattern, ContainsSymbol::YieldExpression) {
                                GeneratorArrayPatternSource::append_from_pattern(
                                    pattern, cursor, points,
                                )?;
                            }
                        }
                    }
                }
                ObjectPatternElement::RestProperty { .. } => {}
                ObjectPatternElement::AssignmentRestPropertyAccess { access } => {
                    append_target(access, cursor, points)?;
                }
            }
        }
        Some(())
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

fn append_key(
    source: &PropertyName,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match source {
        PropertyName::Literal(_) => Some(()),
        PropertyName::Computed(source) => append_expression(source, cursor, points),
    }
}

fn append_target(
    source: &PropertyAccess,
    cursor: &mut u32,
    points: &mut Vec<GeneratorSuspensionPointIr>,
) -> Option<()> {
    match source {
        PropertyAccess::Simple(source) => {
            append_expression(source.target(), cursor, points)?;
            match source.field() {
                PropertyAccessField::Const(_) => Some(()),
                PropertyAccessField::Expr(source) => append_expression(source, cursor, points),
            }
        }
        PropertyAccess::Private(source) => append_expression(source.target(), cursor, points),
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
