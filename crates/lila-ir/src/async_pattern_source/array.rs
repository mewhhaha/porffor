//! Acquisition and IteratorClose delimit every actual async Array pattern.

use super::*;
use boa_ast::pattern::{ArrayPattern, ArrayPatternElement};

pub(crate) struct AsyncArrayPatternSource<'ast> {
    pattern: &'ast ArrayPattern,
}

pub(crate) struct AsyncArrayPatternSourceStates {
    entry: u32,
    body_entry: u32,
    body_end: u32,
    exit: u32,
    awaits: Vec<(u32, u32)>,
    operations: Vec<(ArrayDestructuringOperationKindIr, u32)>,
}

impl AsyncArrayPatternSourceStates {
    pub(crate) fn entry(&self) -> u32 {
        self.entry
    }
    pub(crate) fn body_entry(&self) -> u32 {
        self.body_entry
    }
    pub(crate) fn body_end(&self) -> u32 {
        self.body_end
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn awaits(&self) -> &[(u32, u32)] {
        &self.awaits
    }
    pub(crate) fn operations(&self) -> &[(ArrayDestructuringOperationKindIr, u32)] {
        &self.operations
    }
}

impl<'ast> AsyncArrayPatternSource<'ast> {
    pub(crate) fn new(pattern: &'ast Pattern) -> Option<Self> {
        let Pattern::Array(pattern) = pattern else {
            return None;
        };
        if contains(pattern, ContainsSymbol::YieldExpression) {
            return None;
        }
        let source = Self { pattern };
        source.states(0)?;
        Some(source)
    }
    pub(crate) fn pattern(&self) -> &'ast ArrayPattern {
        self.pattern
    }
    pub(crate) fn states(&self, entry: u32) -> Option<AsyncArrayPatternSourceStates> {
        let body_entry = entry.checked_add(1)?;
        let mut cursor = body_entry;
        let mut awaits = Vec::new();
        let mut operations = Vec::new();
        for element in self.pattern.bindings() {
            match element {
                ArrayPatternElement::Elision => {
                    operations.push((ArrayDestructuringOperationKindIr::Elision, cursor))
                }
                ArrayPatternElement::SingleNameRest { .. } => {
                    operations.push((ArrayDestructuringOperationKindIr::RestArray, cursor))
                }
                ArrayPatternElement::SingleName { default_init, .. } => {
                    operations.push((ArrayDestructuringOperationKindIr::StepValue, cursor));
                    append_default(default_init.as_ref(), &mut cursor, &mut awaits)?;
                }
                ArrayPatternElement::PropertyAccess {
                    access,
                    default_init,
                } => {
                    append_target(access, &mut cursor, &mut awaits)?;
                    operations.push((ArrayDestructuringOperationKindIr::StepValue, cursor));
                    append_default(default_init.as_ref(), &mut cursor, &mut awaits)?;
                }
                ArrayPatternElement::Pattern {
                    pattern,
                    default_init,
                } => {
                    operations.push((ArrayDestructuringOperationKindIr::StepValue, cursor));
                    append_default(default_init.as_ref(), &mut cursor, &mut awaits)?;
                    append_nested(pattern, &mut cursor, &mut awaits)?;
                }
                ArrayPatternElement::PropertyAccessRest { access } => {
                    append_target(access, &mut cursor, &mut awaits)?;
                    operations.push((ArrayDestructuringOperationKindIr::RestArray, cursor));
                }
                ArrayPatternElement::PatternRest { pattern } => {
                    operations.push((ArrayDestructuringOperationKindIr::RestArray, cursor));
                    append_nested(pattern, &mut cursor, &mut awaits)?;
                }
            }
        }
        Some(AsyncArrayPatternSourceStates {
            entry,
            body_entry,
            body_end: cursor,
            exit: cursor.checked_add(1)?,
            awaits,
            operations,
        })
    }
    pub(crate) fn append(&self, cursor: &mut u32, awaits: &mut Vec<(u32, u32)>) -> Option<()> {
        let states = self.states(*cursor)?;
        *cursor = states.exit;
        awaits.extend(states.awaits);
        Some(())
    }
    pub(super) fn append_from_pattern(
        pattern: &'ast Pattern,
        cursor: &mut u32,
        awaits: &mut Vec<(u32, u32)>,
    ) -> Option<()> {
        let Pattern::Array(pattern) = pattern else {
            return None;
        };
        Self { pattern }.append(cursor, awaits)
    }
}
