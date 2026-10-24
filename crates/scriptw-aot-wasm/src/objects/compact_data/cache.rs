//! A monomorphic cache owns only immutable shape/key metadata and a slot. Every
//! use reloads the receiver's live storage and SoA values after intervening code.
use super::*;

impl FunctionBuilder<'_> {
    fn emit_own_data_cache_site(
        &mut self,
        site: I64Local,
        key: &ValueLocals,
        f: &mut Function,
    ) -> GcLocal<OwnDataReadCache> {
        let schema = self.runtime_schema();
        let head = schema
            .reserve_gc_local(f)
            .initialize(self.current_program().own_data_read_caches(schema, f), f);
        let cursor = schema
            .reserve_gc_local(f)
            .initialize(head.load(schema, f), f);
        let found = schema
            .reserve_gc_local::<OwnDataReadCache, Nullable>(f)
            .initialize_null(schema, f);
        let actual = schema.reserve_i64_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let repeat = self.open_frame(ControlFrameKind::Loop, f);
        cursor.load(schema, f).is_null(f);
        self.emit_branch_if_to_target(exit, f);
        schema
            .field(OwnDataReadCacheSchema::SITE)
            .read(&cursor, schema, f)
            .store_i64(actual, f);
        site.load(f);
        actual.load(f);
        f.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, f);
        found.replace(cursor.load(schema, f), f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        found.load(schema, f).is_null(f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_branch_if_to_target(exit, f);
        cursor.replace(
            schema
                .field(OwnDataReadCacheSchema::NEXT)
                .read(&cursor, schema, f)
                .reference(),
            f,
        );
        self.emit_branch_to_target(repeat, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        found.load(schema, f).is_null(f);
        self.open_frame(ControlFrameKind::If, f);
        let name = schema
            .reserve_gc_local(f)
            .initialize(key.cast_reference::<StringValue>(schema, f), f);
        let cache = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<OwnDataReadCache>().construct(
                (
                    GcOperand::i64_local(site),
                    GcOperand::reference(&name, schema),
                    GcOperand::null(schema),
                    GcOperand::i32(0),
                    GcOperand::reference(&head, schema),
                ),
                f,
            ),
            f,
        );
        self.current_program()
            .publish_own_data_read_cache(&cache, schema, f);
        found.replace(cache.load(schema, f).nullable(), f);
        cache.clear(f);
        name.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let result = schema
            .reserve_gc_local(f)
            .initialize(found.load(schema, f).require_non_null(f), f);
        found.clear(f);
        cursor.clear(f);
        head.clear(f);
        schema.release_i64_local(actual, f);
        result
    }

    pub(crate) fn compile_cached_property_read_helper(&mut self) -> Result<Function, EmitError> {
        let mut f = self.begin_helper_body(RuntimeHelperId::CachedPropertyRead);
        let p =
            self.helper_parameters::<crate::runtime_helpers::CachedPropertyReadParameters>(&mut f);
        let schema = self.runtime_schema();
        let result = schema.reserve_completion(&mut f);
        let handled = schema.reserve_i32_local(&mut f);
        let exit = self.open_frame(ControlFrameKind::Block, &mut f);
        // PropertyRead retains its raw receiver, while canonical GetV has
        // already performed ToObject. Keep the raw nullish error on the former.
        self.compile_nullish_tagged_i32(p.target.tag(), &mut f)?;
        self.open_frame(ControlFrameKind::If, &mut f);
        self.emit_throw_runtime_error(
            scriptw_ir::NativeErrorKind::TypeError,
            RuntimeErrorMessage::CANNOT_READ_PROPERTIES_OF_NULL_OR_UNDEFINED,
            &result,
            &mut f,
        )?;
        self.emit_branch_to_target(exit, &mut f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        f.instruction(&Instruction::I32Const(0));
        handled.store(&mut f);
        // Only compiler-owned static string read sites call this kernel. Keep a
        // value-tag guard here so no Symbol/object can ever enter a strong cache.
        p.key.value().tag().load(&mut f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, &mut f);
        p.target.reference().load(&mut f);
        f.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<OrdinaryObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, &mut f);
        let object = schema.reserve_gc_local(&mut f).initialize(
            p.target.cast_reference::<OrdinaryObject>(schema, &mut f),
            &mut f,
        );
        let base = schema.reserve_gc_local(&mut f).initialize(
            schema
                .field(OrdinaryObjectSchema::PROPERTIES)
                .read(&object, schema, &mut f)
                .reference(),
            &mut f,
        );
        base.is_subtype::<CompactDataStorage>(schema, &mut f);
        self.open_frame(ControlFrameKind::If, &mut f);
        let storage = schema.reserve_gc_local(&mut f).initialize(
            base.load(schema, &mut f)
                .cast_subtype::<CompactDataStorage>(schema, &mut f),
            &mut f,
        );
        // Generic/exotic receivers never allocate or search cache metadata.
        // Registration starts only after proving a live compact receiver.
        let cache = self.emit_own_data_cache_site(p.site, p.key.value(), &mut f);
        let shape = schema.reserve_gc_local(&mut f).initialize(
            schema
                .field(CompactDataStorageSchema::SHAPE)
                .read(&storage, schema, &mut f)
                .reference(),
            &mut f,
        );
        let cached_shape = schema.reserve_gc_local(&mut f).initialize(
            schema
                .field(OwnDataReadCacheSchema::SHAPE)
                .read(&cache, schema, &mut f)
                .reference(),
            &mut f,
        );
        let cached_key = schema.reserve_gc_local(&mut f).initialize(
            schema
                .field(OwnDataReadCacheSchema::KEY)
                .read(&cache, schema, &mut f)
                .reference(),
            &mut f,
        );
        let slot = schema.reserve_i32_local(&mut f);
        cached_key.load(schema, &mut f);
        p.key.value().reference().load(&mut f);
        f.instruction(&Instruction::RefEq);
        shape.load(schema, &mut f);
        cached_shape.load(schema, &mut f);
        f.instruction(&Instruction::RefEq);
        f.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, &mut f);
        schema
            .field(OwnDataReadCacheSchema::SLOT)
            .read(&cache, schema, &mut f)
            .store(slot, &mut f);
        f.instruction(&Instruction::I32Const(1));
        handled.store(&mut f);
        f.instruction(&Instruction::Else);
        self.emit_compact_slot_probe(&storage, &p.key, slot, handled, &mut f)?;
        handled.load(&mut f);
        self.open_frame(ControlFrameKind::If, &mut f);
        schema.field(OwnDataReadCacheSchema::SHAPE).write(
            &cache,
            GcOperand::nullable_reference(&shape, schema),
            schema,
            &mut f,
        );
        schema.field(OwnDataReadCacheSchema::SLOT).write(
            &cache,
            GcOperand::i32_local(slot),
            schema,
            &mut f,
        );
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        handled.load(&mut f);
        self.open_frame(ControlFrameKind::If, &mut f);
        let value = schema.reserve_value_local(&mut f);
        self.emit_compact_value_at(&storage, slot, &value, &mut f);
        result.set_normal(&value, &mut f);
        value.clear(&mut f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        schema.release_i32_local(slot, &mut f);
        cached_shape.clear(&mut f);
        cached_key.clear(&mut f);
        shape.clear(&mut f);
        storage.clear(&mut f);
        cache.clear(&mut f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        base.clear(&mut f);
        object.clear(&mut f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        handled.load(&mut f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, &mut f);
        self.emit_dynamic_property_read_with_key_locals(
            &p.target,
            &p.receiver,
            &p.key,
            &result,
            &mut f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        result.emit(&mut f);
        result.clear(&mut f);
        schema.release_i32_local(handled, &mut f);
        p.release(&mut f);
        f.instruction(&Instruction::End);
        Ok(self.finish_function(f))
    }
}
