//! The actual abstract constructor and prototype belong to their defining Realm.

use super::super::*;
use super::IntrinsicKey;
use crate::functions::{NonArrayRealmIntrinsicSlot, RealmFunctionMaterializationContext};
use crate::gc_types::*;

impl FunctionBuilder<'_> {
    pub(crate) fn emit_initialize_abstract_module_source_intrinsic(
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
            NonArrayRealmIntrinsicSlot::AbstractModuleSourcePrototype,
            &prototype,
            function,
        );
        let constructor = self.init_builtin_constructor_object(
            StandardBuiltinId::AbstractModuleSourceConstructor,
            realm,
            &prototype,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            &prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            Some(StandardBuiltinId::AbstractModuleSourcePrototypeToStringTagGetter),
            None,
            realm,
            true,
            function,
        )?;
        constructor.clear(function);
        prototype.clear(function);
        object.clear(function);
        Ok(())
    }
}
