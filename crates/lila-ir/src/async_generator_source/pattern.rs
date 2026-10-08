//! Mixed patterns consume the one actual expression and suspension allocator.

use super::*;

mod array;
mod object;

pub(crate) use array::{AsyncGeneratorArrayPatternSource, AsyncGeneratorArrayPatternSourceStates};
pub(crate) use object::AsyncGeneratorObjectPatternSource;

pub(crate) enum AsyncGeneratorPatternSource<'ast> {
    Array(AsyncGeneratorArrayPatternSource<'ast>),
    Object(AsyncGeneratorObjectPatternSource<'ast>),
}

impl<'ast> AsyncGeneratorPatternSource<'ast> {
    pub(crate) fn new(source: &'ast Pattern) -> Option<Self> {
        match source {
            Pattern::Array(_) => AsyncGeneratorArrayPatternSource::new(source).map(Self::Array),
            Pattern::Object(pattern) => {
                let mut states = ResumableStateAllocator::at(0);
                object::append(pattern, &mut states).ok()?;
                Some(Self::Object(
                    AsyncGeneratorObjectPatternSource::from_pattern(pattern),
                ))
            }
        }
    }
}

pub(super) fn append_if_suspended(
    source: &Pattern,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    if has_suspension(source) {
        append(source, states)?;
    }
    Ok(())
}

pub(super) fn append(
    source: &Pattern,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    match source {
        Pattern::Array(source) => array::append(source, states).map(|_| ()),
        Pattern::Object(source) => object::append(source, states),
    }
}

pub(super) fn append_target(
    source: &PropertyAccess,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    match source {
        PropertyAccess::Simple(source) => {
            expression::append(source.target(), states)?;
            match source.field() {
                PropertyAccessField::Const(_) => Ok(()),
                PropertyAccessField::Expr(source) => expression::append(source, states),
            }
        }
        PropertyAccess::Private(source) => expression::append(source.target(), states),
        PropertyAccess::Super(source) => match source.field() {
            PropertyAccessField::Const(_) => Ok(()),
            PropertyAccessField::Expr(source) => expression::append(source, states),
        },
    }
}

pub(super) fn append_default(
    source: Option<&Expression>,
    states: &mut ResumableStateAllocator,
) -> Result<(), AsyncGeneratorSourceError> {
    if let Some(source) = source.filter(|source| has_suspension(*source)) {
        expression::append_branches(None, Some(source), None, states)?;
    }
    Ok(())
}

pub(crate) fn default_states(
    source: &Expression,
    entry: u32,
) -> Option<AsyncGeneratorIfSourceStates> {
    expression::branch_parts_states(None, Some(source), None, entry)
}
