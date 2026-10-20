//! Intrinsic methods preserve their canonical property installation order.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter, AccessorSetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_object_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("groupBy", StandardBuiltinId::ObjectGroupBy),
            ("fromEntries", StandardBuiltinId::ObjectFromEntries),
            ("assign", StandardBuiltinId::ObjectAssign),
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
            (
                "hasOwnProperty",
                StandardBuiltinId::ObjectPrototypeHasOwnProperty,
            ),
            (
                "__defineGetter__",
                StandardBuiltinId::ObjectPrototypeDefineGetter,
            ),
            (
                "__defineSetter__",
                StandardBuiltinId::ObjectPrototypeDefineSetter,
            ),
            (
                "__lookupGetter__",
                StandardBuiltinId::ObjectPrototypeLookupGetter,
            ),
            (
                "__lookupSetter__",
                StandardBuiltinId::ObjectPrototypeLookupSetter,
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
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("__proto__"),
            AccessorDescriptor::GetterAndSetter {
                getter: AccessorGetter::new(StandardBuiltinId::ObjectPrototypeProtoGetter),
                setter: AccessorSetter::new(StandardBuiltinId::ObjectPrototypeProtoSetter),
            },
            context.realm,
            true,
            function,
        )?;
        for (name, builtin) in [
            (
                "propertyIsEnumerable",
                StandardBuiltinId::ObjectPrototypePropertyIsEnumerable,
            ),
            (
                "isPrototypeOf",
                StandardBuiltinId::ObjectPrototypeIsPrototypeOf,
            ),
            ("toString", StandardBuiltinId::ObjectPrototypeToString),
            (
                "toLocaleString",
                StandardBuiltinId::ObjectPrototypeToLocaleString,
            ),
            ("valueOf", StandardBuiltinId::ObjectPrototypeValueOf),
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
            ("create", StandardBuiltinId::ObjectCreate),
            ("getPrototypeOf", StandardBuiltinId::ObjectGetPrototypeOf),
            ("setPrototypeOf", StandardBuiltinId::ObjectSetPrototypeOf),
            ("defineProperty", StandardBuiltinId::ObjectDefineProperty),
            (
                "getOwnPropertyDescriptor",
                StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
            ),
            (
                "getOwnPropertyDescriptors",
                StandardBuiltinId::ObjectGetOwnPropertyDescriptors,
            ),
            (
                "getOwnPropertyNames",
                StandardBuiltinId::ObjectGetOwnPropertyNames,
            ),
            (
                "getOwnPropertySymbols",
                StandardBuiltinId::ObjectGetOwnPropertySymbols,
            ),
            ("keys", StandardBuiltinId::ObjectKeys),
            ("values", StandardBuiltinId::ObjectValues),
            ("entries", StandardBuiltinId::ObjectEntries),
            ("hasOwn", StandardBuiltinId::ObjectHasOwn),
            (
                "defineProperties",
                StandardBuiltinId::ObjectDefineProperties,
            ),
            ("is", StandardBuiltinId::ObjectIs),
            ("isSealed", StandardBuiltinId::ObjectIsSealed),
            ("isFrozen", StandardBuiltinId::ObjectIsFrozen),
            ("seal", StandardBuiltinId::ObjectSeal),
            ("freeze", StandardBuiltinId::ObjectFreeze),
            ("isExtensible", StandardBuiltinId::ObjectIsExtensible),
            (
                "preventExtensions",
                StandardBuiltinId::ObjectPreventExtensions,
            ),
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
}
