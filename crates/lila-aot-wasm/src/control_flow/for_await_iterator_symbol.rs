use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ForAwaitIteratorSymbol {
    AsyncIterator,
    Iterator,
}
impl ForAwaitIteratorSymbol {
    const fn symbol(self) -> lila_ir::WellKnownSymbol {
        match self {
            Self::AsyncIterator => lila_ir::WellKnownSymbol::AsyncIterator,
            Self::Iterator => lila_ir::WellKnownSymbol::Iterator,
        }
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_for_await_well_known_symbol_read(
        &mut self,
        symbol: ForAwaitIteratorSymbol,
        target: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let boxed = schema.reserve_completion(function);
        self.emit_value_to_object_locals(target, &boxed, function)?;
        self.completion().copy_from(&boxed, function);
        self.emit_propagate_current_throw_if_needed(function);
        let symbol = schema.reserve_gc_local(function).initialize(
            self.emit_well_known_symbol_reference(symbol.symbol(), function)?,
            function,
        );
        let key = crate::operations::PropertyKeyLocals::from_symbol(schema, &symbol, function);
        self.emit_object_read(boxed.value(), target, &key, result, function)?;
        key.clear(function);
        symbol.clear(function);
        boxed.clear(function);
        Ok(())
    }
}
