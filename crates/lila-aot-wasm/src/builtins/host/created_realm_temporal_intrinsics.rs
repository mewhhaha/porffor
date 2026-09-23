use super::*;
use crate::intrinsics::temporal::{TemporalIntrinsicFamily, TemporalIntrinsicRealm};

#[must_use = "created-Realm Temporal intrinsics must be published"]
pub(super) struct CreatedRealmTemporalIntrinsics(u32);

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_materialize_created_realm_temporal_intrinsics(
        &mut self,
        realm: RealmRecordLocal,
        realm_functions: &RealmFunctionMaterializationContext,
        object_prototype: u32,
        function: &mut Function,
    ) -> Result<CreatedRealmTemporalIntrinsics, EmitError> {
        let namespace = self.reserve_temp_local();
        let constructor = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype), None, function)?;
        function.instruction(&Instruction::LocalSet(namespace));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        for family in TemporalIntrinsicFamily::ALL {
            self.emit_alloc_plain_object_with_prototype(Some(object_prototype), None, function)?;
            function.instruction(&Instruction::LocalSet(prototype));
            self.emit_store_non_array_realm_intrinsic(
                realm.index(),
                family.prototype_slot(),
                prototype,
                function,
            );
            self.emit_temporal_intrinsic_callable(
                family.constructor(),
                &TemporalIntrinsicRealm::Created(realm_functions),
                constructor,
                function,
            )?;
            self.emit_set_function_prototype_data_with_flags(
                constructor,
                prototype,
                false,
                false,
                false,
                true,
                function,
            )?;
            self.emit_install_temporal_intrinsic_members(
                family,
                constructor,
                prototype,
                TemporalIntrinsicRealm::Created(realm_functions),
                function,
            )?;
            let name = family.constructor().native_function_name().ok_or_else(|| {
                EmitError::unsupported("missing Temporal constructor native name")
            })?;
            self.emit_object_define_local_data(namespace, name, constructor, tag, function)?;
        }
        self.emit_define_temporal_intrinsic_to_string_tag(namespace, "Temporal", function)?;
        self.release_temp_local(tag);
        self.release_temp_local(prototype);
        self.release_temp_local(constructor);
        Ok(CreatedRealmTemporalIntrinsics(namespace))
    }

    pub(super) fn emit_publish_created_realm_temporal_intrinsics(
        &mut self,
        intrinsics: CreatedRealmTemporalIntrinsics,
        global: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        let result =
            self.emit_object_define_local_data(global, "Temporal", intrinsics.0, tag, function);
        self.release_temp_local(tag);
        self.release_temp_local(intrinsics.0);
        result
    }
}
