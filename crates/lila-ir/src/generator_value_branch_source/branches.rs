//! Complete selected value branches retain their actual source operands.
use super::*;
use crate::generator_loop_control::GeneratorLoopSourceRange;

/// A complete region accepts only an expression admitted by its state planner.
pub(crate) struct CheckedGeneratorRegionValue<'ast> {
    source: &'ast Expression,
}

impl<'ast> CheckedGeneratorRegionValue<'ast> {
    fn new(source: &'ast Expression) -> Option<Self> {
        GeneratorExpressionSourcePlan::new(
            source,
            GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
        )?;
        Some(Self { source })
    }

    pub(crate) fn source(&self) -> &'ast Expression {
        self.source
    }
}

pub(crate) enum GeneratorValueBranchKind<'ast> {
    Conditional {
        selector: StagedGeneratorSelector<'ast>,
        then_arm: CheckedGeneratorRegionValue<'ast>,
        else_arm: CheckedGeneratorRegionValue<'ast>,
    },
    Logical {
        op: LogicalOp,
        left: StagedGeneratorSelector<'ast>,
        rhs: CheckedGeneratorRegionValue<'ast>,
    },
}

pub(crate) struct GeneratorValueBranchSource<'ast> {
    kind: GeneratorValueBranchKind<'ast>,
}

impl<'ast> GeneratorValueBranchSource<'ast> {
    pub(crate) fn new(source: &'ast Expression) -> Option<Self> {
        if !contains(source, ContainsSymbol::YieldExpression) {
            return None;
        }
        let kind = match source {
            Expression::Conditional(source) => GeneratorValueBranchKind::Conditional {
                selector: StagedGeneratorSelector::new(source.condition())?,
                then_arm: CheckedGeneratorRegionValue::new(source.if_true())?,
                else_arm: CheckedGeneratorRegionValue::new(source.if_false())?,
            },
            Expression::Binary(source) => {
                let BinaryOp::Logical(op) = source.op() else {
                    return None;
                };
                GeneratorValueBranchKind::Logical {
                    op,
                    left: StagedGeneratorSelector::new(source.lhs())?,
                    rhs: CheckedGeneratorRegionValue::new(source.rhs())?,
                }
            }
            _ => return None,
        };
        Some(Self { kind })
    }

    /// Entry belongs to the branch after its selector has completed.
    pub(crate) fn states(&self, entry: u32) -> Option<GeneratorValueRegionStates> {
        let (then_arm, else_arm) = match &self.kind {
            GeneratorValueBranchKind::Conditional {
                then_arm, else_arm, ..
            } => (Some(then_arm.source()), Some(else_arm.source())),
            GeneratorValueBranchKind::Logical { op, rhs, .. } => match op {
                LogicalOp::And | LogicalOp::Coalesce => (Some(rhs.source()), None),
                LogicalOp::Or => (None, Some(rhs.source())),
            },
        };
        GeneratorValueRegionStates::new(entry, then_arm, else_arm)
    }

    pub(crate) fn selector_source(&self) -> &'ast Expression {
        match &self.kind {
            GeneratorValueBranchKind::Conditional { selector, .. } => selector.source(),
            GeneratorValueBranchKind::Logical { left, .. } => left.source(),
        }
    }

    pub(super) fn append(
        self,
        cursor: &mut u32,
        points: &mut Vec<GeneratorSuspensionPointIr>,
    ) -> Option<()> {
        let (selector, then_arm, else_arm) = match self.kind {
            GeneratorValueBranchKind::Conditional {
                selector,
                then_arm,
                else_arm,
            } => (selector, Some(then_arm), Some(else_arm)),
            GeneratorValueBranchKind::Logical { op, left, rhs } => match op {
                LogicalOp::And | LogicalOp::Coalesce => (left, Some(rhs), None),
                LogicalOp::Or => (left, None, Some(rhs)),
            },
        };
        selector.plan.append(cursor, points)?;
        let states = GeneratorValueRegionStates::new(
            *cursor,
            then_arm.as_ref().map(CheckedGeneratorRegionValue::source),
            else_arm.as_ref().map(CheckedGeneratorRegionValue::source),
        )?;
        points.extend(states.suspensions);
        *cursor = states.exit;
        Some(())
    }

    pub(crate) fn into_kind(self) -> GeneratorValueBranchKind<'ast> {
        self.kind
    }
}

/// Source-owned arm ranges include eager/skipped arms and a distinct common
/// exit, so a resume cannot enter the other selected arm.
pub(crate) struct GeneratorValueRegionStates {
    entry: u32,
    then_arm: GeneratorLoopSourceRange,
    else_arm: GeneratorLoopSourceRange,
    exit: u32,
    suspensions: Vec<GeneratorSuspensionPointIr>,
}

impl GeneratorValueRegionStates {
    pub(super) fn new(
        entry: u32,
        then_source: Option<&Expression>,
        else_source: Option<&Expression>,
    ) -> Option<Self> {
        let mut cursor = entry.checked_add(1)?;
        let mut suspensions = Vec::new();
        let mut arm = |source: Option<&Expression>| -> Option<GeneratorLoopSourceRange> {
            let start = cursor;
            if let Some(source) = source {
                GeneratorExpressionSourcePlan::new(
                    source,
                    GeneratorValueBranchAdmission::OrdinaryOutsideLoops,
                )?
                .append(&mut cursor, &mut suspensions)?;
            }
            let range = GeneratorLoopSourceRange {
                entry: start,
                end: cursor,
            };
            cursor = cursor.checked_add(1)?;
            Some(range)
        };
        let then_arm = arm(then_source)?;
        let else_arm = arm(else_source)?;
        // The function's final state count must fit before publishing points.
        cursor.checked_add(1)?;
        Some(Self {
            entry,
            then_arm,
            else_arm,
            exit: cursor,
            suspensions,
        })
    }

    pub(crate) fn entry(&self) -> u32 {
        self.entry
    }
    pub(crate) fn then_arm(&self) -> GeneratorLoopSourceRange {
        self.then_arm
    }
    pub(crate) fn else_arm(&self) -> GeneratorLoopSourceRange {
        self.else_arm
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }

    pub(super) fn append_suspensions(self, points: &mut Vec<GeneratorSuspensionPointIr>) {
        points.extend(self.suspensions);
    }
}
