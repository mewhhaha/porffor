//! Plain async patterns retain their original operands and checked continuation tape.

use super::*;
use crate::ArrayDestructuringOperationKindIr;

#[path = "async_pattern_source/array.rs"]
mod array;
#[path = "async_pattern_source/expression.rs"]
mod expression;
#[path = "async_pattern_source/object.rs"]
mod object;

pub(crate) use array::{AsyncArrayPatternSource, AsyncArrayPatternSourceStates};
pub(crate) use expression::append as append_async_expression_states;
pub(crate) use object::AsyncObjectPatternSource;

/// The actual pattern decides which complete consumer owns its continuation.
pub(crate) enum AsyncPatternSource<'ast> {
    Array(AsyncArrayPatternSource<'ast>),
    Object(AsyncObjectPatternSource<'ast>),
}

impl<'ast> AsyncPatternSource<'ast> {
    pub(crate) fn new(pattern: &'ast Pattern) -> Option<Self> {
        match pattern {
            Pattern::Array(_) => AsyncArrayPatternSource::new(pattern).map(Self::Array),
            Pattern::Object(_) => AsyncObjectPatternSource::new(pattern).map(Self::Object),
        }
    }
    pub(crate) fn append(&self, cursor: &mut u32, awaits: &mut Vec<(u32, u32)>) -> Option<()> {
        match self {
            Self::Array(source) => source.append(cursor, awaits),
            Self::Object(source) => source.append(cursor, awaits),
        }
    }
}

pub(crate) struct AsyncPatternDefaultStates {
    entry: u32,
    then_entry: u32,
    then_end: u32,
    else_entry: u32,
    exit: u32,
    awaits: Vec<(u32, u32)>,
    resumable: bool,
}

impl AsyncPatternDefaultStates {
    pub(crate) fn new(source: &Expression, entry: u32) -> Option<Self> {
        let resumable = contains(source, ContainsSymbol::AwaitExpression);
        let then_entry = if resumable {
            entry.checked_add(1)?
        } else {
            entry
        };
        let mut then_end = then_entry;
        let mut awaits = Vec::new();
        expression::append(source, &mut then_end, &mut awaits)?;
        let else_entry = if resumable {
            then_end.checked_add(1)?
        } else {
            entry
        };
        let exit = if resumable {
            else_entry.checked_add(1)?
        } else {
            entry
        };
        Some(Self {
            entry,
            then_entry,
            then_end,
            else_entry,
            exit,
            awaits,
            resumable,
        })
    }
    pub(crate) fn entry(&self) -> u32 {
        self.entry
    }
    pub(crate) fn then_entry(&self) -> u32 {
        self.then_entry
    }
    pub(crate) fn then_end(&self) -> u32 {
        self.then_end
    }
    pub(crate) fn else_entry(&self) -> u32 {
        self.else_entry
    }
    pub(crate) fn exit(&self) -> u32 {
        self.exit
    }
    pub(crate) fn resumable(&self) -> bool {
        self.resumable
    }
    fn append(self, cursor: &mut u32, awaits: &mut Vec<(u32, u32)>) {
        *cursor = self.exit;
        awaits.extend(self.awaits);
    }
}

fn append_default(
    source: Option<&Expression>,
    cursor: &mut u32,
    awaits: &mut Vec<(u32, u32)>,
) -> Option<()> {
    if let Some(source) = source {
        AsyncPatternDefaultStates::new(source, *cursor)?.append(cursor, awaits);
    }
    Some(())
}

fn append_target(
    source: &PropertyAccess,
    cursor: &mut u32,
    awaits: &mut Vec<(u32, u32)>,
) -> Option<()> {
    match source {
        PropertyAccess::Simple(source) => {
            expression::append(source.target(), cursor, awaits)?;
            match source.field() {
                PropertyAccessField::Const(_) => Some(()),
                PropertyAccessField::Expr(source) => expression::append(source, cursor, awaits),
            }
        }
        PropertyAccess::Private(source) => expression::append(source.target(), cursor, awaits),
        PropertyAccess::Super(source) => match source.field() {
            PropertyAccessField::Const(_) => Some(()),
            PropertyAccessField::Expr(source) => expression::append(source, cursor, awaits),
        },
    }
}

fn append_nested(pattern: &Pattern, cursor: &mut u32, awaits: &mut Vec<(u32, u32)>) -> Option<()> {
    match pattern {
        Pattern::Object(_) => {
            AsyncObjectPatternSource::append_from_pattern(pattern, cursor, awaits)
        }
        Pattern::Array(_) => AsyncArrayPatternSource::append_from_pattern(pattern, cursor, awaits),
    }
}
