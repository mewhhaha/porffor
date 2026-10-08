//! Ordinary Switch consumes the exact source-owned selection and body regions.
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_ordinary_generator_switch(
        &mut self,
        switch: &AstSwitch,
    ) -> (StatementIr, ValueKind) {
        // Function capture owns any defining With records. The Switch adds
        // only its CaseBlock child; normal Reference lowering keeps the
        // captured object slots and their inner-to-outer resolution order.
        let entry = self
            .plain_generator_entry_state()
            .expect("ordinary generator Switch activation");
        let Some(checked) = CheckedGeneratorSwitchSource::new(switch, entry) else {
            self.unsupported("ordinary generator Switch source has no owned selection plan");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let (switch, states) = checked.into_parts();
        self.lower_resumable_switch::<super::resumable_switch::OrdinarySwitch>(switch, states)
    }
}
