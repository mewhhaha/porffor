//! Delete's terminal property Reference stays separate from prefix Gets/Calls.
use crate::{OptionalChainOperationIr, PropertyKeyIr, Strictness, TypedExpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidDeleteOptionalPropertyChainIr {
    NonPropertyTerminal,
}

/// Only the checked constructor can consume a complete chain and retain its
/// actual final property key. The terminal operation never performs GetValue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteOptionalPropertyChainIr {
    target: Box<TypedExpr>,
    prefix: Vec<OptionalChainOperationIr>,
    key: PropertyKeyIr,
    shorted: bool,
    strictness: Strictness,
}

impl DeleteOptionalPropertyChainIr {
    pub(crate) fn new(
        target: TypedExpr,
        mut complete_chain: Vec<OptionalChainOperationIr>,
        strictness: Strictness,
    ) -> Result<Self, InvalidDeleteOptionalPropertyChainIr> {
        let Some(OptionalChainOperationIr::Property { key, shorted }) = complete_chain.pop() else {
            return Err(InvalidDeleteOptionalPropertyChainIr::NonPropertyTerminal);
        };
        Ok(Self {
            target: Box::new(target),
            prefix: complete_chain,
            key,
            shorted,
            strictness,
        })
    }

    pub fn target(&self) -> &TypedExpr {
        &self.target
    }
    pub fn prefix(&self) -> &[OptionalChainOperationIr] {
        &self.prefix
    }
    pub fn key(&self) -> &PropertyKeyIr {
        &self.key
    }
    pub fn shorted(&self) -> bool {
        self.shorted
    }
    pub fn strictness(&self) -> Strictness {
        self.strictness
    }
}
