//! A native JSON evaluation value minted only from a retained parsed JSON record.
use super::record::ModuleUnitId;
pub use lila_front::JsonValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonModuleValueIr {
    module: ModuleUnitId,
    source: lila_front::ParsedJson,
}

impl JsonModuleValueIr {
    pub(super) fn new(module: ModuleUnitId, source: lila_front::ParsedJson) -> Self {
        Self { module, source }
    }
    pub const fn module(&self) -> ModuleUnitId {
        self.module
    }
    pub fn value(&self) -> &lila_front::JsonValue {
        self.source.value()
    }
}
