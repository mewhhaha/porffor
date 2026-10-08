//! Numeric prototype state is completed during allocation, before publication.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

impl FunctionBuilder<'_> {
    pub(crate) fn install_big_int_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "BigInt",
            false,
            false,
            true,
            function,
        )?;
        for (name, builtin) in [
            ("toString", StandardBuiltinId::BigIntPrototypeToString),
            (
                "toLocaleString",
                StandardBuiltinId::BigIntPrototypeToLocaleString,
            ),
            ("valueOf", StandardBuiltinId::BigIntPrototypeValueOf),
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
        for (name, builtin) in [
            ("asIntN", StandardBuiltinId::BigIntAsIntN),
            ("asUintN", StandardBuiltinId::BigIntAsUintN),
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
        Ok(())
    }

    pub(crate) fn install_number_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, value) in [
            ("NaN", f64::NAN),
            ("POSITIVE_INFINITY", f64::INFINITY),
            ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
            ("MAX_VALUE", f64::MAX),
            ("MIN_VALUE", f64::from_bits(1)),
            ("EPSILON", f64::EPSILON),
            ("MAX_SAFE_INTEGER", 9007199254740991.0),
            ("MIN_SAFE_INTEGER", -9007199254740991.0),
        ] {
            self.emit_install_intrinsic_number(
                context.constructor,
                name,
                value,
                false,
                false,
                false,
                function,
            )?;
        }
        for builtin in [
            StandardBuiltinId::NumberIsInteger,
            StandardBuiltinId::NumberIsSafeInteger,
            StandardBuiltinId::NumberIsFinite,
            StandardBuiltinId::NumberIsNaN,
        ] {
            self.emit_install_intrinsic_method(
                context.constructor,
                IntrinsicKey::Name(builtin.native_function_name().ok_or_else(|| {
                    EmitError::unsupported("planned Number static method has no native name")
                })?),
                builtin,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        for builtin in [HostBuiltinId::ParseInt, HostBuiltinId::ParseFloat] {
            let schema = self.runtime_schema();
            let callable =
                self.emit_intrinsic_canonical_host_callable(builtin, context.realm, function)?;
            let value = schema.reserve_value_local(function);
            value.set_reference(&callable, schema, function);
            self.emit_install_intrinsic_data(
                context.constructor,
                IntrinsicKey::Name(builtin.as_str()),
                &value,
                true,
                false,
                true,
                function,
            )?;
            value.clear(function);
            callable.clear(function);
        }
        for builtin in [
            StandardBuiltinId::NumberPrototypeToFixed,
            StandardBuiltinId::NumberPrototypeToExponential,
            StandardBuiltinId::NumberPrototypeToPrecision,
            StandardBuiltinId::NumberPrototypeToString,
            StandardBuiltinId::NumberPrototypeToLocaleString,
            StandardBuiltinId::NumberPrototypeValueOf,
        ] {
            self.emit_install_intrinsic_method(
                context.prototype,
                IntrinsicKey::Name(builtin.native_function_name().ok_or_else(|| {
                    EmitError::unsupported("planned Number prototype method has no native name")
                })?),
                builtin,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        Ok(())
    }

    pub(crate) fn install_boolean_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("toString", StandardBuiltinId::BooleanPrototypeToString),
            ("valueOf", StandardBuiltinId::BooleanPrototypeValueOf),
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
        Ok(())
    }
}
