//! `binary_data` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_array_buffer_constructor_intrinsics(
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

        if matches!(builtin, StandardBuiltinId::ArrayBufferConstructor) {
            let is_view_meta = self
                .functions
                .get(&StandardBuiltinId::ArrayBufferIsView.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `ArrayBuffer.isView`",
                    )
                })?;
            self.emit_object_define_function_data(object_local, "isView", is_view_meta, function)?;
        }

        let key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let species_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayBufferSpeciesGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `ArrayBuffer[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(species_meta, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            object_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;

        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(object_local));
        for (name, builtin) in [(
            "byteLength",
            if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
                StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
            } else {
                StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
            },
        )] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(getter_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(getter_tag_local));
            self.emit_object_append_accessor_property_with_flags(
                object_local,
                key_local,
                Some((getter_payload_local, getter_tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
            let grow_meta = self
                .functions
                .get(&StandardBuiltinId::SharedArrayBufferPrototypeGrow.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `SharedArrayBuffer.prototype.grow`",
                    )
                })?;
            self.emit_object_define_function_data(object_local, "grow", grow_meta, function)?;
            for (name, builtin) in [
                (
                    "maxByteLength",
                    StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter,
                ),
                (
                    "growable",
                    StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter,
                ),
            ] {
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(getter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(getter_tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    object_local,
                    key_local,
                    Some((getter_payload_local, getter_tag_local)),
                    None,
                    false,
                    true,
                    function,
                )?;
            }
        } else {
            for (name, builtin) in [
                (
                    "detached",
                    StandardBuiltinId::ArrayBufferPrototypeDetachedGetter,
                ),
                (
                    "maxByteLength",
                    StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter,
                ),
                (
                    "resizable",
                    StandardBuiltinId::ArrayBufferPrototypeResizableGetter,
                ),
            ] {
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(getter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(getter_tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    object_local,
                    key_local,
                    Some((getter_payload_local, getter_tag_local)),
                    None,
                    false,
                    true,
                    function,
                )?;
            }
        }
        let slice_builtin = if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
            StandardBuiltinId::SharedArrayBufferPrototypeSlice
        } else {
            StandardBuiltinId::ArrayBufferPrototypeSlice
        };
        let slice_meta = self
            .functions
            .get(&slice_builtin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `ArrayBuffer.prototype.slice`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "slice", slice_meta, function)?;
        if matches!(builtin, StandardBuiltinId::ArrayBufferConstructor) {
            let resize_meta = self
                .functions
                .get(&StandardBuiltinId::ArrayBufferPrototypeResize.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `ArrayBuffer.prototype.resize`",
                    )
                })?;
            self.emit_object_define_function_data(object_local, "resize", resize_meta, function)?;
            for (name, builtin) in [
                ("transfer", StandardBuiltinId::ArrayBufferPrototypeTransfer),
                (
                    "transferToFixedLength",
                    StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength,
                ),
                (
                    "transferToImmutable",
                    StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable,
                ),
                (
                    "sliceToImmutable",
                    StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable,
                ),
            ] {
                let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
                self.emit_object_define_function_data(object_local, name, meta, function)?;
            }
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(
            if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
                "SharedArrayBuffer"
            } else {
                "ArrayBuffer"
            },
        )));
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            getter_payload_local,
            getter_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(key_local);

        Ok(())
    }

    pub(crate) fn install_data_view_constructor_intrinsics(
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

        let prototype_object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            ("buffer", StandardBuiltinId::DataViewPrototypeBufferGetter),
            (
                "byteLength",
                StandardBuiltinId::DataViewPrototypeByteLengthGetter,
            ),
            (
                "byteOffset",
                StandardBuiltinId::DataViewPrototypeByteOffsetGetter,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(getter_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(getter_tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((getter_payload_local, getter_tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        for (name, builtin) in [
            ("getUint8", StandardBuiltinId::DataViewPrototypeGetUint8),
            ("setUint8", StandardBuiltinId::DataViewPrototypeSetUint8),
            ("getInt8", StandardBuiltinId::DataViewPrototypeGetInt8),
            ("setInt8", StandardBuiltinId::DataViewPrototypeSetInt8),
            ("getUint16", StandardBuiltinId::DataViewPrototypeGetUint16),
            ("setUint16", StandardBuiltinId::DataViewPrototypeSetUint16),
            ("getInt16", StandardBuiltinId::DataViewPrototypeGetInt16),
            ("setInt16", StandardBuiltinId::DataViewPrototypeSetInt16),
            ("getUint32", StandardBuiltinId::DataViewPrototypeGetUint32),
            ("setUint32", StandardBuiltinId::DataViewPrototypeSetUint32),
            ("getInt32", StandardBuiltinId::DataViewPrototypeGetInt32),
            ("setInt32", StandardBuiltinId::DataViewPrototypeSetInt32),
            ("getFloat16", StandardBuiltinId::DataViewPrototypeGetFloat16),
            ("setFloat16", StandardBuiltinId::DataViewPrototypeSetFloat16),
            ("getFloat32", StandardBuiltinId::DataViewPrototypeGetFloat32),
            ("setFloat32", StandardBuiltinId::DataViewPrototypeSetFloat32),
            ("getFloat64", StandardBuiltinId::DataViewPrototypeGetFloat64),
            ("setFloat64", StandardBuiltinId::DataViewPrototypeSetFloat64),
            (
                "getBigInt64",
                StandardBuiltinId::DataViewPrototypeGetBigInt64,
            ),
            (
                "setBigInt64",
                StandardBuiltinId::DataViewPrototypeSetBigInt64,
            ),
            (
                "getBigUint64",
                StandardBuiltinId::DataViewPrototypeGetBigUint64,
            ),
            (
                "setBigUint64",
                StandardBuiltinId::DataViewPrototypeSetBigUint64,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(prototype_object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("DataView")));
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            getter_payload_local,
            getter_tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_object_local);

        Ok(())
    }
}
