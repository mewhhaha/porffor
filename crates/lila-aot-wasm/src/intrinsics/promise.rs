//! Fresh intrinsic properties retain completed values and defining Realm.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter};

pub(crate) const PROMISE_PROTOTYPE_METHOD_PUBLICATIONS: [StandardBuiltinId; 3] = [
    StandardBuiltinId::PromisePrototypeThen,
    StandardBuiltinId::PromisePrototypeCatch,
    StandardBuiltinId::PromisePrototypeFinally,
];

pub(crate) const PROMISE_STATIC_METHOD_PUBLICATIONS: [StandardBuiltinId; 10] = [
    StandardBuiltinId::PromiseResolve,
    StandardBuiltinId::PromiseReject,
    StandardBuiltinId::PromiseAll,
    StandardBuiltinId::PromiseAllSettled,
    StandardBuiltinId::PromiseAllKeyed,
    StandardBuiltinId::PromiseAllSettledKeyed,
    StandardBuiltinId::PromiseAny,
    StandardBuiltinId::PromiseRace,
    StandardBuiltinId::PromiseWithResolvers,
    StandardBuiltinId::PromiseTry,
];

impl FunctionBuilder<'_> {
    pub(crate) fn install_promise_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for builtin in PROMISE_PROTOTYPE_METHOD_PUBLICATIONS {
            let name = builtin.native_function_name().ok_or_else(|| {
                EmitError::unsupported("planned Promise prototype method has no native name")
            })?;
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
            "Promise",
            false,
            false,
            true,
            function,
        )?;
        for builtin in PROMISE_STATIC_METHOD_PUBLICATIONS {
            let name = builtin.native_function_name().ok_or_else(|| {
                EmitError::unsupported("planned Promise static method has no native name")
            })?;
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
        self.emit_install_intrinsic_accessor(
            context.constructor,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Species),
            AccessorDescriptor::Getter(AccessorGetter::new(
                StandardBuiltinId::PromiseSpeciesGetter,
            )),
            context.realm,
            true,
            function,
        )?;
        Ok(())
    }
}
