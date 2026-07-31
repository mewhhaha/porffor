//! `string` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_string_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_store_boxed_primitive_metadata(
            object_local,
            BOXED_PRIMITIVE_KIND_STRING,
            payload_local,
            tag_local,
            function,
        );
        for builtin in [
            StandardBuiltinId::StringPrototypeToString,
            StandardBuiltinId::StringPrototypeValueOf,
            StandardBuiltinId::StringPrototypeCharAt,
            StandardBuiltinId::StringPrototypeConcat,
            StandardBuiltinId::StringPrototypeCharCodeAt,
            StandardBuiltinId::StringPrototypeCodePointAt,
            StandardBuiltinId::StringPrototypeAt,
            StandardBuiltinId::StringPrototypeAnchor,
            StandardBuiltinId::StringPrototypeBig,
            StandardBuiltinId::StringPrototypeBlink,
            StandardBuiltinId::StringPrototypeBold,
            StandardBuiltinId::StringPrototypeFixed,
            StandardBuiltinId::StringPrototypeFontcolor,
            StandardBuiltinId::StringPrototypeFontsize,
            StandardBuiltinId::StringPrototypeItalics,
            StandardBuiltinId::StringPrototypeLink,
            StandardBuiltinId::StringPrototypeSmall,
            StandardBuiltinId::StringPrototypeStrike,
            StandardBuiltinId::StringPrototypeSub,
            StandardBuiltinId::StringPrototypeSubstr,
            StandardBuiltinId::StringPrototypeSubstring,
            StandardBuiltinId::StringPrototypeSup,
            StandardBuiltinId::StringPrototypeMatch,
            StandardBuiltinId::StringPrototypeMatchAll,
            StandardBuiltinId::StringPrototypeReplace,
            StandardBuiltinId::StringPrototypeReplaceAll,
            StandardBuiltinId::StringPrototypeSearch,
            StandardBuiltinId::StringPrototypeIndexOf,
            StandardBuiltinId::StringPrototypeLastIndexOf,
            StandardBuiltinId::StringPrototypeSlice,
            StandardBuiltinId::StringPrototypeSplit,
            StandardBuiltinId::StringPrototypePadStart,
            StandardBuiltinId::StringPrototypePadEnd,
            StandardBuiltinId::StringPrototypeRepeat,
            StandardBuiltinId::StringPrototypeEndsWith,
            StandardBuiltinId::StringPrototypeIncludes,
            StandardBuiltinId::StringPrototypeStartsWith,
            StandardBuiltinId::StringPrototypeNormalize,
            StandardBuiltinId::StringPrototypeLocaleCompare,
            StandardBuiltinId::StringPrototypeToLocaleLowerCase,
            StandardBuiltinId::StringPrototypeToLocaleUpperCase,
            StandardBuiltinId::StringPrototypeToLowerCase,
            StandardBuiltinId::StringPrototypeToUpperCase,
            StandardBuiltinId::StringPrototypeTrim,
            StandardBuiltinId::StringPrototypeTrimStart,
            StandardBuiltinId::StringPrototypeTrimEnd,
            StandardBuiltinId::StringPrototypeIsWellFormed,
            StandardBuiltinId::StringPrototypeToWellFormed,
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            match builtin {
                StandardBuiltinId::StringPrototypeTrimStart => {
                    self.emit_object_define_function_data_with_aliases(
                        object_local,
                        "trimStart",
                        &["trimLeft"],
                        meta,
                        function,
                    )?;
                }
                StandardBuiltinId::StringPrototypeTrimEnd => {
                    self.emit_object_define_function_data_with_aliases(
                        object_local,
                        "trimEnd",
                        &["trimRight"],
                        meta,
                        function,
                    )?;
                }
                _ => self.emit_object_define_function_data(
                    object_local,
                    builtin.string_prototype_method_name().unwrap(),
                    meta,
                    function,
                )?,
            }
        }
        let iterator_meta = self
            .functions
            .get(&StandardBuiltinId::StringPrototypeIterator.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype[Symbol.iterator]`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "Symbol.iterator",
            iterator_meta,
            function,
        )?;

        Ok(())
    }
}
