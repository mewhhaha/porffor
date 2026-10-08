//! Date prototype publication preserves method order and UTC/GMT identity.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

impl FunctionBuilder<'_> {
    pub(crate) fn install_date_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("now", StandardBuiltinId::DateNow),
            ("parse", StandardBuiltinId::DateParse),
            ("UTC", StandardBuiltinId::DateUtc),
        ] {
            self.emit_install_intrinsic_method(
                context.constructor,
                IntrinsicKey::Name(name),
                builtin,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        for (name, builtin) in [
            ("getTime", StandardBuiltinId::DatePrototypeGetTime),
            ("setTime", StandardBuiltinId::DatePrototypeSetTime),
            ("valueOf", StandardBuiltinId::DatePrototypeValueOf),
            ("getFullYear", StandardBuiltinId::DatePrototypeGetFullYear),
            (
                "getUTCFullYear",
                StandardBuiltinId::DatePrototypeGetUtcFullYear,
            ),
            ("getMonth", StandardBuiltinId::DatePrototypeGetMonth),
            ("getUTCMonth", StandardBuiltinId::DatePrototypeGetUtcMonth),
            ("getDate", StandardBuiltinId::DatePrototypeGetDate),
            ("getUTCDate", StandardBuiltinId::DatePrototypeGetUtcDate),
            ("getDay", StandardBuiltinId::DatePrototypeGetDay),
            ("getUTCDay", StandardBuiltinId::DatePrototypeGetUtcDay),
            ("getHours", StandardBuiltinId::DatePrototypeGetHours),
            ("getUTCHours", StandardBuiltinId::DatePrototypeGetUtcHours),
            ("getMinutes", StandardBuiltinId::DatePrototypeGetMinutes),
            (
                "getUTCMinutes",
                StandardBuiltinId::DatePrototypeGetUtcMinutes,
            ),
            ("getSeconds", StandardBuiltinId::DatePrototypeGetSeconds),
            (
                "getUTCSeconds",
                StandardBuiltinId::DatePrototypeGetUtcSeconds,
            ),
            (
                "getMilliseconds",
                StandardBuiltinId::DatePrototypeGetMilliseconds,
            ),
            (
                "getUTCMilliseconds",
                StandardBuiltinId::DatePrototypeGetUtcMilliseconds,
            ),
            (
                "getTimezoneOffset",
                StandardBuiltinId::DatePrototypeGetTimezoneOffset,
            ),
            ("getYear", StandardBuiltinId::DatePrototypeGetYear),
            ("setYear", StandardBuiltinId::DatePrototypeSetYear),
            ("setFullYear", StandardBuiltinId::DatePrototypeSetFullYear),
            (
                "setUTCFullYear",
                StandardBuiltinId::DatePrototypeSetUtcFullYear,
            ),
            ("setMonth", StandardBuiltinId::DatePrototypeSetMonth),
            ("setUTCMonth", StandardBuiltinId::DatePrototypeSetUtcMonth),
            ("setDate", StandardBuiltinId::DatePrototypeSetDate),
            ("setUTCDate", StandardBuiltinId::DatePrototypeSetUtcDate),
            ("setHours", StandardBuiltinId::DatePrototypeSetHours),
            ("setUTCHours", StandardBuiltinId::DatePrototypeSetUtcHours),
            ("setMinutes", StandardBuiltinId::DatePrototypeSetMinutes),
            (
                "setUTCMinutes",
                StandardBuiltinId::DatePrototypeSetUtcMinutes,
            ),
            ("setSeconds", StandardBuiltinId::DatePrototypeSetSeconds),
            (
                "setUTCSeconds",
                StandardBuiltinId::DatePrototypeSetUtcSeconds,
            ),
            (
                "setMilliseconds",
                StandardBuiltinId::DatePrototypeSetMilliseconds,
            ),
            (
                "setUTCMilliseconds",
                StandardBuiltinId::DatePrototypeSetUtcMilliseconds,
            ),
            ("toISOString", StandardBuiltinId::DatePrototypeToIsoString),
            ("toJSON", StandardBuiltinId::DatePrototypeToJson),
            ("toDateString", StandardBuiltinId::DatePrototypeToDateString),
            (
                "toLocaleDateString",
                StandardBuiltinId::DatePrototypeToLocaleDateString,
            ),
            (
                "toLocaleString",
                StandardBuiltinId::DatePrototypeToLocaleString,
            ),
            (
                "toLocaleTimeString",
                StandardBuiltinId::DatePrototypeToLocaleTimeString,
            ),
            (
                "toTemporalInstant",
                StandardBuiltinId::DatePrototypeToTemporalInstant,
            ),
            ("toTimeString", StandardBuiltinId::DatePrototypeToTimeString),
            ("toString", StandardBuiltinId::DatePrototypeToString),
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
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToPrimitive),
            StandardBuiltinId::DatePrototypeToPrimitive,
            context.realm,
            false,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method_aliases(
            context.prototype,
            &[
                IntrinsicKey::Name("toUTCString"),
                IntrinsicKey::Name("toGMTString"),
            ],
            StandardBuiltinId::DatePrototypeToUtcString,
            context.realm,
            true,
            true,
            function,
        )?;
        Ok(())
    }
}
