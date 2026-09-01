use super::*;

pub(super) enum CreatedRealmIteratorNextTarget {
    Array,
    String,
    Map,
    Set,
}

impl CreatedRealmIteratorNextTarget {
    fn builtin(&self) -> StandardBuiltinId {
        match self {
            Self::Array => StandardBuiltinId::ArrayIteratorNext,
            Self::String => StandardBuiltinId::StringIteratorNext,
            Self::Map => StandardBuiltinId::MapIteratorNext,
            Self::Set => StandardBuiltinId::SetIteratorNext,
        }
    }
}

pub(super) struct CreatedRealmIteratorNextPublicationContext<'a> {
    realm_functions: &'a RealmFunctionMaterializationContext,
    type_error_prototype_local: u32,
    array_iterator_prototype_local: u32,
    string_iterator_prototype_local: u32,
    map_iterator_prototype_local: u32,
    set_iterator_prototype_local: u32,
}

impl<'a> CreatedRealmIteratorNextPublicationContext<'a> {
    pub(super) fn new(
        realm_functions: &'a RealmFunctionMaterializationContext,
        type_error_prototype_local: u32,
        array_iterator_prototype_local: u32,
        string_iterator_prototype_local: u32,
        map_iterator_prototype_local: u32,
        set_iterator_prototype_local: u32,
    ) -> Self {
        Self {
            realm_functions,
            type_error_prototype_local,
            array_iterator_prototype_local,
            string_iterator_prototype_local,
            map_iterator_prototype_local,
            set_iterator_prototype_local,
        }
    }

    fn prototype_local(&self, target: &CreatedRealmIteratorNextTarget) -> u32 {
        match target {
            CreatedRealmIteratorNextTarget::Array => self.array_iterator_prototype_local,
            CreatedRealmIteratorNextTarget::String => self.string_iterator_prototype_local,
            CreatedRealmIteratorNextTarget::Map => self.map_iterator_prototype_local,
            CreatedRealmIteratorNextTarget::Set => self.set_iterator_prototype_local,
        }
    }
}

#[must_use = "created-Realm iterator next functions must be published"]
pub(super) struct CreatedRealmIteratorNext {
    prototype_local: u32,
    function_local: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_materialize_created_realm_iterator_next(
        &mut self,
        target: CreatedRealmIteratorNextTarget,
        context: &CreatedRealmIteratorNextPublicationContext<'_>,
        function: &mut Function,
    ) -> Result<CreatedRealmIteratorNext, EmitError> {
        let builtin = target.builtin();
        let prototype_local = context.prototype_local(&target);
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
        let function_local = self.reserve_temp_local();
        self.emit_function_value_payload_in_realm(
            &meta,
            context.realm_functions,
            function_local,
            function,
        )?;
        self.store_i64_local_at_offset(
            function_local,
            HEAP_FUNCTION_ENV_HANDLE_OFFSET,
            function_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_local,
            HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
            context.type_error_prototype_local,
            function,
        );
        Ok(CreatedRealmIteratorNext {
            prototype_local,
            function_local,
        })
    }

    pub(super) fn emit_publish_created_realm_iterator_next(
        &mut self,
        iterator_next: CreatedRealmIteratorNext,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CreatedRealmIteratorNext {
            prototype_local,
            function_local,
        } = iterator_next;
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_local_data(
            prototype_local,
            "next",
            function_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(function_local);
        Ok(())
    }
}
