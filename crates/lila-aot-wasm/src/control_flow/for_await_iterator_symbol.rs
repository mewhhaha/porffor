use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ForAwaitIteratorSymbol {
    AsyncIterator,
    Iterator,
}

impl ForAwaitIteratorSymbol {
    const fn name(self) -> &'static str {
        match self {
            Self::AsyncIterator => "Symbol.asyncIterator",
            Self::Iterator => "Symbol.iterator",
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// Read a well-known-symbol method off a `for await (… of …)` head value.
    ///
    /// `PropertyKeyIr::StaticString("Symbol.asyncIterator")` is *not* the
    /// well-known symbol: `compile_object_key_to_locals` lowers a static string
    /// key to `strings.payload(name)` tagged `String`, so it looks up the
    /// ordinary string property `"Symbol.asyncIterator"` and always misses.
    /// A symbol key has to carry `PROPERTY_KEY_SYMBOL_MARKER`, which the
    /// `StringExpr` path ORs in when the key expression is `Symbol`-kinded.
    /// This mirrors `emit_generator_delegate_property_read`, the `yield*`
    /// equivalent, and keeps primitive receivers (strings, numbers) working by
    /// going through the dynamic read.
    pub(super) fn emit_for_await_well_known_symbol_read(
        &mut self,
        symbol: ForAwaitIteratorSymbol,
        target_payload_local: u32,
        target_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = symbol.name();
        let target = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags()
                    .without(ValueKind::Undefined)
                    .without(ValueKind::Null),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            ExprIr::Undefined,
        );
        let symbol_key = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Symbol),
            ExprIr::String(key.to_string()),
        );
        self.compile_property_read_from_locals(
            &target,
            &PropertyKeyIr::StringExpr(Box::new(symbol_key)),
            target_payload_local,
            target_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )
    }
}
