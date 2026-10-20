use super::*;
use crate::async_generator_source::AsyncGeneratorSwitchSource;

impl ScriptLowerer<'_> {
    pub(super) fn lower_async_generator_switch(
        &mut self,
        switch: &AstSwitch,
    ) -> (StatementIr, ValueKind) {
        let Some(source) = AsyncGeneratorSwitchSource::new(switch) else {
            self.unsupported("async-generator Switch source has no complete selection owner");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let Some(states) = self
            .async_generator_entry_state()
            .and_then(|entry| source.states(entry))
        else {
            self.unsupported("async-generator Switch requires complete mixed source ranges");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        self.lower_resumable_switch::<super::resumable_switch::MixedSwitch>(source.source(), states)
    }
}
