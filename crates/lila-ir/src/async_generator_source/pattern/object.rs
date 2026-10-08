use super::*;
use boa_ast::pattern::{ObjectPattern, ObjectPatternElement};

pub(crate) struct AsyncGeneratorObjectPatternSource<'ast> {
    pattern: &'ast ObjectPattern,
}
impl<'ast> AsyncGeneratorObjectPatternSource<'ast> {
    pub(super) fn from_pattern(pattern: &'ast ObjectPattern) -> Self {
        Self { pattern }
    }
    pub(crate) fn pattern(&self) -> &'ast ObjectPattern {
        self.pattern
    }
    pub(crate) fn end_state(&self, entry: u32) -> Option<u32> {
        let mut states = ResumableStateAllocator::at(entry);
        append(self.pattern, &mut states).ok()?;
        Some(states.current())
    }
}
pub(super) fn append(
    source: &ObjectPattern,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    for element in source.bindings() {
        match element {
            ObjectPatternElement::SingleName {
                name, default_init, ..
            } => {
                append_key(name, states)?;
                append_default(default_init.as_ref(), states)?;
            }
            ObjectPatternElement::AssignmentPropertyAccess {
                name,
                access,
                default_init,
            } => {
                append_key(name, states)?;
                append_target(access, states)?;
                append_default(default_init.as_ref(), states)?;
            }
            ObjectPatternElement::Pattern {
                name,
                pattern,
                default_init,
            } => {
                append_key(name, states)?;
                append_default(default_init.as_ref(), states)?;
                super::append(pattern, states)?;
            }
            ObjectPatternElement::RestProperty { .. } => {}
            ObjectPatternElement::AssignmentRestPropertyAccess { access } => {
                append_target(access, states)?
            }
        }
    }
    Ok(())
}
fn append_key(
    source: &PropertyName,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    match source {
        PropertyName::Literal(_) => Ok(()),
        PropertyName::Computed(source) => expression::append(source, states),
    }
}
