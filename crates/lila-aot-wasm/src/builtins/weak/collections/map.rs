use super::*;
impl FunctionBuilder<'_> {
    fn emit_weak_map_put(
        &mut self,
        record: &GcLocal<WeakMapObject>,
        resource: &GcLocal<HostResource>,
        key: &ValueLocals,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(value, f), f);
        self.weak_import(GcHostImport::WeakMapSet)?
            .put_weak_map(record, resource, key, &stored, s, f);
        stored.clear(f);
        Ok(())
    }
    pub(in super::super) fn emit_weak_map_operation(
        &mut self,
        op: MapOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let record = self.emit_weak_receiver::<WeakMapObject>(WeakFamily::Map, f)?;
        let resource = s.reserve_gc_local(f).initialize(
            s.field(WeakMapObjectSchema::RESOURCE)
                .read(&record, s, f)
                .reference(),
            f,
        );
        let key = s.reserve_value_local(f);
        let argument = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let found = s.reserve_i32_local(f);
        self.emit_builtin_arg_to_value(0, &key, f);
        self.emit_builtin_arg_to_value(1, &argument, f);
        value.set_undefined(f);
        found.set_constant(0, f);
        self.emit_whole_can_be_held_weakly_i32(&key, f);
        match op {
            MapOperation::Get | MapOperation::Has | MapOperation::Delete => {
                self.open_frame(ControlFrameKind::If, f);
            }
            MapOperation::Set | MapOperation::GetOrInsert | MapOperation::GetOrInsertComputed => {
                f.instruction(&Instruction::I32Eqz);
                self.emit_weak_assert(
                    RuntimeErrorMessage::WEAKMAP_KEY_MUST_BE_AN_OBJECT_OR_UNREGISTERED_SYMBOL,
                    f,
                )?;
            }
        }
        match op {
            MapOperation::Has | MapOperation::Delete => {
                let import = match op {
                    MapOperation::Has => GcHostImport::WeakMapHas,
                    MapOperation::Delete => GcHostImport::WeakMapDelete,
                    MapOperation::Get
                    | MapOperation::Set
                    | MapOperation::GetOrInsert
                    | MapOperation::GetOrInsertComputed => unreachable!(),
                };
                self.weak_import(import)?
                    .weak_map_membership(&resource, &key, found, s, f);
            }
            MapOperation::Set => {
                self.emit_weak_map_put(&record, &resource, &key, &argument, f)?;
                value.set_reference(&record, s, f);
            }
            MapOperation::Get | MapOperation::GetOrInsert | MapOperation::GetOrInsertComputed => {
                if matches!(op, MapOperation::GetOrInsertComputed) {
                    self.emit_collection_assert_callable(&argument, RuntimeErrorMessage::WEAKMAP_PROTOTYPE_GETORINSERTCOMPUTED_CALLBACK_MUST_BE_CALLABLE, f)?;
                }
                let stored = s.reserve_gc_local(f).initialize(
                    self.weak_import(GcHostImport::WeakMapGet)?
                        .get_weak_map(&resource, &key, s, f),
                    f,
                );
                stored.load(s, f).is_null(f);
                f.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, f);
                let present = s
                    .reserve_gc_local(f)
                    .initialize(stored.load(s, f).require_non_null(f), f);
                s.struct_type::<StoredValue>()
                    .read_into(&present, &value, s, f);
                present.clear(f);
                f.instruction(&Instruction::Else);
                match op {
                    MapOperation::Get => {}
                    MapOperation::GetOrInsert => {
                        value.copy_from(&argument, f);
                        self.emit_weak_map_put(&record, &resource, &key, &value, f)?;
                    }
                    MapOperation::GetOrInsertComputed => {
                        let undefined = s.reserve_value_local(f);
                        undefined.set_undefined(f);
                        let pending = s.reserve_completion(f);
                        let args = self.emit_pre_evaluated_arg_vector(&[&key], f);
                        self.emit_function_or_proxy_call_with_argv(
                            &argument, &undefined, &args, &pending, f,
                        )?;
                        args.clear(f);
                        self.emit_collection_propagate(&pending, f);
                        value.copy_from(pending.value(), f);
                        // The callback can replace this key; the computed value wins.
                        self.emit_weak_map_put(&record, &resource, &key, &value, f)?;
                        pending.clear(f);
                        undefined.clear(f);
                    }
                    MapOperation::Has | MapOperation::Delete | MapOperation::Set => unreachable!(),
                }
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
                stored.clear(f);
            }
        }
        match op {
            MapOperation::Get | MapOperation::Has | MapOperation::Delete => {
                self.pop_control(ControlFrameKind::If);
                f.instruction(&Instruction::End);
            }
            MapOperation::Set | MapOperation::GetOrInsert | MapOperation::GetOrInsertComputed => {}
        }
        match op {
            MapOperation::Has | MapOperation::Delete => self.emit_collection_boolean(found, f),
            MapOperation::Get
            | MapOperation::Set
            | MapOperation::GetOrInsert
            | MapOperation::GetOrInsertComputed => self.completion().set_normal(&value, f),
        }
        s.release_i32_local(found, f);
        value.clear(f);
        argument.clear(f);
        key.clear(f);
        resource.clear(f);
        record.clear(f);
        Ok(())
    }
}
