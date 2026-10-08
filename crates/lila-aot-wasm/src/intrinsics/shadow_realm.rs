//! Each Realm owns its ShadowRealm constructor, prototype and native methods.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::functions::{NonArrayRealmIntrinsicSlot, RealmFunctionMaterializationContext};
use crate::gc_types::ValueLocals;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_initialize_shadow_realm_intrinsic(
        &mut self,
        realm: &RealmFunctionMaterializationContext,
        object_prototype: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let object = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(object_prototype), function)?,
            function,
        );
        let prototype = schema.reserve_value_local(function);
        prototype.set_reference(&object, schema, function);
        self.emit_store_non_array_realm_intrinsic(
            realm.realm(),
            NonArrayRealmIntrinsicSlot::ShadowRealmPrototype,
            &prototype,
            function,
        );
        let constructor = self.init_builtin_constructor_object(
            StandardBuiltinId::ShadowRealmConstructor,
            realm,
            &prototype,
            function,
        )?;
        constructor.clear(function);
        prototype.clear(function);
        object.clear(function);
        Ok(())
    }

    pub(crate) fn install_shadow_realm_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for &(name, builtin) in lila_ir::SHADOW_REALM_PROTOTYPE_METHODS {
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
            IntrinsicKey::Symbol(WellKnownSymbol::ToStringTag),
            lila_ir::SHADOW_REALM_NAME,
            false,
            false,
            true,
            function,
        )
    }
}
