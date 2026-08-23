use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_break(&mut self, brk: &AstBreak) -> (StatementIr, ValueKind) {
        if let Some(label) = brk.label() {
            let label = self.interner.resolve_expect(label).to_string();
            if self.labels.iter().rev().any(|active| active.name == label) {
                return (
                    StatementIr::Break { label: Some(label) },
                    ValueKind::Undefined,
                );
            }
            self.unsupported("break to unknown label");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        if self.breakable_depth == 0 {
            self.unsupported("break outside loop or switch");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        (StatementIr::Break { label: None }, ValueKind::Undefined)
    }

    pub(super) fn lower_continue(&mut self, cont: &AstContinue) -> (StatementIr, ValueKind) {
        if let Some(label) = cont.label() {
            let label = self.interner.resolve_expect(label).to_string();
            let Some(active) = self.labels.iter().rev().find(|active| active.name == label) else {
                self.unsupported("continue to unknown label");
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            if active.kind != LabelTargetKind::Loop {
                self.unsupported("continue to non-loop label");
                return (StatementIr::Empty, ValueKind::Undefined);
            }
            return (
                StatementIr::Continue { label: Some(label) },
                ValueKind::Undefined,
            );
        }
        if self.loop_depth == 0 {
            self.unsupported("continue outside loop");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        (StatementIr::Continue { label: None }, ValueKind::Undefined)
    }
}
