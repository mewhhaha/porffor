use super::global_environment::{GlobalBindingFailure, GLOBAL_ENV_REALM_OFFSET};
use super::named_environment::{
    NamedEnvironmentKind, NAMED_BINDING_DELETABLE_OFFSET, NAMED_BINDING_PRESENT_OFFSET,
};
use super::*;

#[derive(Clone, Copy)]
enum EnvironmentReferenceKind {
    Unresolvable,
    Declarative,
    GlobalLexical,
    WithObject,
    GlobalObject,
}

impl EnvironmentReferenceKind {
    const fn code(self) -> i64 {
        match self {
            Self::Unresolvable => 0,
            Self::Declarative => 1,
            Self::GlobalLexical => 2,
            Self::WithObject => 3,
            Self::GlobalObject => 4,
        }
    }
}

/// A Reference keeps the selected record and source name across RHS evaluation.
/// Eval can change that record's binding table, so PutValue resolves its name
/// again within this record, without searching a newly changed parent chain.
#[must_use]
pub(crate) struct EnvironmentIdentifierReference {
    key: u32,
    kind: u32,
    record: u32,
    entry: u32,
    base_payload: u32,
    base_tag: u32,
    strictness: Strictness,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_resolve_environment_identifier(
        &mut self,
        name_local: u32,
        strictness: Strictness,
        function: &mut Function,
    ) -> Result<EnvironmentIdentifierReference, EmitError> {
        let reference = EnvironmentIdentifierReference {
            key: self.reserve_temp_local(),
            kind: self.reserve_temp_local(),
            record: self.reserve_temp_local(),
            entry: self.reserve_temp_local(),
            base_payload: self.reserve_temp_local(),
            base_tag: self.reserve_temp_local(),
            strictness,
        };
        let parent_local = self.reserve_temp_local();
        let record_kind_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(name_local));
        function.instruction(&Instruction::LocalSet(reference.key));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(reference.record));
        for local in [
            reference.kind,
            reference.entry,
            reference.base_payload,
            reference.base_tag,
        ] {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(local));
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            reference.record,
            ENV_PARENT_OFFSET,
            parent_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_global_lexical_entry_to_local(reference.key, reference.entry, function);
        function.instruction(&Instruction::LocalGet(reference.entry));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            reference.record,
            GLOBAL_ENV_REALM_OFFSET,
            reference.base_payload,
            function,
        );
        self.load_i64_to_local_from_offset(
            reference.base_payload,
            HEAP_REALM_GLOBAL_THIS_OFFSET,
            reference.base_payload,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(reference.base_tag));
        self.emit_object_has_property_i32(
            reference.base_payload,
            reference.base_tag,
            reference.key,
            present_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::GlobalObject.code(),
        ));
        function.instruction(&Instruction::LocalSet(reference.kind));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::GlobalLexical.code(),
        ));
        function.instruction(&Instruction::LocalSet(reference.kind));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            reference.record,
            ENV_RECORD_KIND_OFFSET,
            record_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(record_kind_local));
        function.instruction(&Instruction::I64Const(
            NamedEnvironmentKind::WithObject.code() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            reference.record,
            ENV_WITH_OBJECT_OFFSET,
            reference.entry,
            function,
        );
        self.load_i64_to_local_from_offset(
            reference.entry,
            ENV_SLOT_PAYLOAD_OFFSET,
            reference.base_payload,
            function,
        );
        self.load_i64_to_local_from_offset(
            reference.entry,
            ENV_SLOT_TAG_OFFSET,
            reference.base_tag,
            function,
        );
        self.emit_with_environment_has_binding(
            reference.base_payload,
            reference.base_tag,
            reference.key,
            present_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::WithObject.code(),
        ));
        function.instruction(&Instruction::LocalSet(reference.kind));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_find_own_named_binding(
            reference.record,
            reference.key,
            reference.entry,
            function,
        );
        function.instruction(&Instruction::LocalGet(reference.entry));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Declarative.code(),
        ));
        function.instruction(&Instruction::LocalSet(reference.kind));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::LocalSet(reference.record));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(present_local);
        self.release_temp_local(record_kind_local);
        self.release_temp_local(parent_local);
        Ok(reference)
    }

    fn emit_with_environment_has_binding(
        &mut self,
        object_local: u32,
        object_tag_local: u32,
        name_local: u32,
        present_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let exclusions_local = self.reserve_temp_local();
        let exclusions_tag_local = self.reserve_temp_local();
        let blocked_local = self.reserve_temp_local();
        let blocked_tag_local = self.reserve_temp_local();
        self.emit_object_has_property_i32(
            object_local,
            object_tag_local,
            name_local,
            present_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.unscopables"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            exclusions_local,
            exclusions_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(exclusions_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            exclusions_local,
            exclusions_tag_local,
            exclusions_local,
            exclusions_tag_local,
            name_local,
            blocked_local,
            blocked_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(blocked_tag_local, blocked_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(present_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(blocked_tag_local);
        self.release_temp_local(blocked_local);
        self.release_temp_local(exclusions_tag_local);
        self.release_temp_local(exclusions_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn release_environment_identifier_reference(
        &mut self,
        reference: EnvironmentIdentifierReference,
    ) {
        self.release_temp_local(reference.base_tag);
        self.release_temp_local(reference.base_payload);
        self.release_temp_local(reference.entry);
        self.release_temp_local(reference.record);
        self.release_temp_local(reference.kind);
        self.release_temp_local(reference.key);
    }

    pub(crate) fn emit_environment_identifier_call_base(
        &mut self,
        reference: &EnvironmentIdentifierReference,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::WithObject.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reference.base_payload));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(reference.base_tag));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::End);
    }
}

pub(crate) enum EnvironmentIdentifierRead {
    Value,
    Typeof,
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_environment_identifier_get(
        &mut self,
        reference: &EnvironmentIdentifierReference,
        read: EnvironmentIdentifierRead,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Unresolvable.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        match read {
            EnvironmentIdentifierRead::Value => {
                self.emit_throw_global_binding_error(
                    GlobalBindingFailure::Unresolvable,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            EnvironmentIdentifierRead::Typeof => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
        }
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Declarative.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::GlobalLexical.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_global_lexical_read(reference.entry, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        let present_local = self.reserve_temp_local();
        self.emit_object_has_property_i32(
            reference.base_payload,
            reference.base_tag,
            reference.key,
            present_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        match reference.strictness {
            Strictness::Strict => {
                self.emit_throw_global_binding_error(
                    GlobalBindingFailure::Unresolvable,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            Strictness::Sloppy => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
        }
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_object_read(
            reference.base_payload,
            reference.base_tag,
            reference.base_payload,
            reference.base_tag,
            reference.key,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(present_local);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_environment_identifier_put(
        &mut self,
        reference: &EnvironmentIdentifierReference,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Declarative.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_set_named_environment_binding(
            reference.record,
            reference.key,
            reference.strictness,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::GlobalLexical.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_global_lexical_write(reference.entry, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        if reference.strictness.throws_on_failed_set() {
            function.instruction(&Instruction::LocalGet(reference.kind));
            function.instruction(&Instruction::I64Const(
                EnvironmentReferenceKind::Unresolvable.code(),
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_global_binding_error(
                GlobalBindingFailure::UnresolvableAssignment,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
        }
        let key_tag_local = self.reserve_temp_local();
        let success_local = self.reserve_temp_local();
        // Object Environment Record SetMutableBinding rechecks the held
        // binding object after RHS effects; unresolvable sloppy PutValue
        // goes straight to the global object's Set instead.
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Unresolvable.code(),
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_has_property_i32(
            reference.base_payload,
            reference.base_tag,
            reference.key,
            success_local,
            function,
        )?;
        if reference.strictness.throws_on_failed_set() {
            function.instruction(&Instruction::LocalGet(success_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_global_binding_error(
                GlobalBindingFailure::UnresolvableAssignment,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_ordinary_set_result_via_helper(
            reference.base_payload,
            reference.base_tag,
            reference.base_payload,
            reference.base_tag,
            reference.key,
            key_tag_local,
            payload_local,
            tag_local,
            success_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        if reference.strictness.throws_on_failed_set() {
            function.instruction(&Instruction::LocalGet(success_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Cannot assign to read only property",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_propagate_current_completion_if_throw(function);
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(success_local);
        self.release_temp_local(key_tag_local);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_environment_identifier_delete(
        &mut self,
        reference: &EnvironmentIdentifierReference,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Unresolvable.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::GlobalLexical.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(reference.kind));
        function.instruction(&Instruction::I64Const(
            EnvironmentReferenceKind::Declarative.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_find_own_named_binding(
            reference.record,
            reference.key,
            reference.entry,
            function,
        );
        function.instruction(&Instruction::LocalGet(reference.entry));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            reference.entry,
            NAMED_BINDING_DELETABLE_OFFSET,
            result_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.store_i64_const_at_offset(reference.entry, NAMED_BINDING_PRESENT_OFFSET, 0, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_object_delete(
            reference.base_payload,
            reference.base_tag,
            reference.key,
            result_local,
            function,
        )?;
        self.emit_propagate_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        Ok(())
    }
}
