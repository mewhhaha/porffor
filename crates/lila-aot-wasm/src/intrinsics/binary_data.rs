//! Intrinsic members retain property order and exact shared function identity.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

impl FunctionBuilder<'_> {
    pub(crate) fn install_array_buffer_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let shared = context.builtin == StandardBuiltinId::SharedArrayBufferConstructor;
        if !shared {
            self.emit_install_intrinsic_method(
                context.constructor,
                IntrinsicKey::Name("isView"),
                StandardBuiltinId::ArrayBufferIsView,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        self.emit_install_intrinsic_accessor(
            context.constructor,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Species),
            Some(StandardBuiltinId::ArrayBufferSpeciesGetter),
            None,
            context.realm,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("byteLength"),
            Some(if shared {
                StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
            } else {
                StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
            }),
            None,
            context.realm,
            true,
            function,
        )?;
        if shared {
            self.emit_install_intrinsic_method(
                context.prototype,
                IntrinsicKey::Name("grow"),
                StandardBuiltinId::SharedArrayBufferPrototypeGrow,
                context.realm,
                true,
                true,
                function,
            )?;
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
                self.emit_install_intrinsic_accessor(
                    context.prototype,
                    IntrinsicKey::Name(name),
                    Some(builtin),
                    None,
                    context.realm,
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
                self.emit_install_intrinsic_accessor(
                    context.prototype,
                    IntrinsicKey::Name(name),
                    Some(builtin),
                    None,
                    context.realm,
                    true,
                    function,
                )?;
            }
        }
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Name("slice"),
            if shared {
                StandardBuiltinId::SharedArrayBufferPrototypeSlice
            } else {
                StandardBuiltinId::ArrayBufferPrototypeSlice
            },
            context.realm,
            true,
            true,
            function,
        )?;
        if !shared {
            for (name, builtin) in [
                ("resize", StandardBuiltinId::ArrayBufferPrototypeResize),
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
                self.emit_install_intrinsic_method(
                    context.prototype,
                    IntrinsicKey::Name(name),
                    builtin,
                    context.realm,
                    true,
                    true,
                    function,
                )?;
            }
        }
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            if shared {
                "SharedArrayBuffer"
            } else {
                "ArrayBuffer"
            },
            false,
            false,
            true,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_data_view_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
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
            self.emit_install_intrinsic_accessor(
                context.prototype,
                IntrinsicKey::Name(name),
                Some(builtin),
                None,
                context.realm,
                true,
                function,
            )?;
        }
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
            self.emit_install_intrinsic_method(
                context.prototype,
                IntrinsicKey::Name(name),
                builtin,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "DataView",
            false,
            false,
            true,
            function,
        )?;
        Ok(())
    }
}
