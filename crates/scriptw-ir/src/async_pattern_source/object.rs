//! Nested Object patterns consume original key, target, default and rest order.

use super::*;
use boa_ast::pattern::{ObjectPattern, ObjectPatternElement};

pub(crate) struct AsyncObjectPatternSource<'ast> {
    pattern: &'ast ObjectPattern,
}
impl<'ast> AsyncObjectPatternSource<'ast> {
    pub(crate) fn new(pattern: &'ast Pattern) -> Option<Self> {
        let Pattern::Object(pattern) = pattern else {
            return None;
        };
        if contains(pattern, ContainsSymbol::YieldExpression) {
            return None;
        }
        let source = Self { pattern };
        source.append(&mut 0, &mut Vec::new())?;
        Some(source)
    }
    pub(crate) fn pattern(&self) -> &'ast ObjectPattern {
        self.pattern
    }
    pub(crate) fn append(&self, cursor: &mut u32, awaits: &mut Vec<(u32, u32)>) -> Option<()> {
        for element in self.pattern.bindings() {
            match element {
                ObjectPatternElement::SingleName {
                    name, default_init, ..
                } => {
                    append_key(name, cursor, awaits)?;
                    append_default(default_init.as_ref(), cursor, awaits)?;
                }
                ObjectPatternElement::AssignmentPropertyAccess {
                    name,
                    access,
                    default_init,
                } => {
                    append_key(name, cursor, awaits)?;
                    append_target(access, cursor, awaits)?;
                    append_default(default_init.as_ref(), cursor, awaits)?;
                }
                ObjectPatternElement::Pattern {
                    name,
                    pattern,
                    default_init,
                } => {
                    append_key(name, cursor, awaits)?;
                    append_default(default_init.as_ref(), cursor, awaits)?;
                    append_nested(pattern, cursor, awaits)?;
                }
                ObjectPatternElement::RestProperty { .. } => {}
                ObjectPatternElement::AssignmentRestPropertyAccess { access } => {
                    append_target(access, cursor, awaits)?
                }
            }
        }
        Some(())
    }
    pub(super) fn append_from_pattern(
        pattern: &'ast Pattern,
        cursor: &mut u32,
        awaits: &mut Vec<(u32, u32)>,
    ) -> Option<()> {
        let Pattern::Object(pattern) = pattern else {
            return None;
        };
        Self { pattern }.append(cursor, awaits)
    }
}
fn append_key(source: &PropertyName, cursor: &mut u32, awaits: &mut Vec<(u32, u32)>) -> Option<()> {
    match source {
        PropertyName::Literal(_) => Some(()),
        PropertyName::Computed(source) => expression::append(source, cursor, awaits),
    }
}
