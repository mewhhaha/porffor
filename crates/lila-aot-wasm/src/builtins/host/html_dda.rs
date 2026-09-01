use super::*;

impl<'a> FunctionBuilder<'a> {
    /// Mint the `$262.IsHTMLDDA` exotic object (B.3.6 The [[IsHTMLDDA]]
    /// Internal Slot).
    ///
    /// The returned value is an ordinary-looking function object whose flags
    /// word carries `FUNCTION_FLAG_IS_HTMLDDA`; that flag is what
    /// `emit_is_htmldda_function_i32` reads for the ToBoolean (B.3.6.1),
    /// `typeof` (B.3.6.2) and IsLooselyEqual (B.3.6.3) overrides.
    /// `emit_function_value_payload` also skips creating the `prototype` own
    /// property for this meta, so `Object.defineProperty(obj, "prototype", ...)`
    /// in `superclass-emulates-undefined.js` sees no pre-existing
    /// non-configurable property.
    pub(crate) fn compile_host_create_html_dda_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let html_dda_meta = self
            .functions
            .get(&HostBuiltinId::HTMLDDA.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing HTMLDDA host callable",
                )
            })?;
        // The annexB tests install own properties on this object
        // (`prototype`, `@@matchAll`, `@@replace`, `@@search`), so it has to be
        // extensible. `emit_function_value_payload` zeroes `HEAP_CAP_OFFSET`
        // — the very slot `emit_ordinary_is_extensible_i32` reads — whenever
        // `length_name_configurable` is false, so the host-builtin meta must
        // keep it true.
        debug_assert!(
            html_dda_meta.length_name_configurable,
            "HTMLDDA host callable must stay extensible for Object.defineProperty",
        );
        self.emit_function_value_payload(&html_dda_meta, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        Ok(())
    }

    pub(crate) fn compile_host_html_dda_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        Ok(())
    }
}
