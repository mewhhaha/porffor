use super::*;

/// The completed capture inventory proves full ordinary syntax and names have
/// been validated. Only this consuming owner admits width/lowering/publication.
#[must_use]
pub(in crate::builtins::regexp::compiler) struct CompletedRegExpPattern {
    captures: CompletedCaptureInventory,
}
impl CompletedRegExpPattern {
    pub(super) fn new(captures: CompletedCaptureInventory) -> Self {
        Self { captures }
    }
    pub(in crate::builtins::regexp::compiler) fn emit_group_count(&self, function: &mut Function) {
        self.captures.emit_group_count(function);
    }
    pub(in crate::builtins::regexp::compiler) fn into_capture_inventory(
        self,
    ) -> CompletedCaptureInventory {
        self.captures
    }
}
