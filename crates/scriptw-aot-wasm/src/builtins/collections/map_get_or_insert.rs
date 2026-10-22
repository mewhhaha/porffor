use super::*;
#[derive(Clone, Copy)]
enum MapGetOrInsertValueSource {
    ValueArgument,
    ComputedCallback,
}
impl FunctionBuilder<'_> {
    pub(crate) fn emit_map_prototype_get_or_insert(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_collection_get_or_insert(MapGetOrInsertValueSource::ValueArgument, f)
    }
    pub(crate) fn emit_map_prototype_get_or_insert_computed(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_collection_get_or_insert(MapGetOrInsertValueSource::ComputedCallback, f)
    }
    fn emit_collection_get_or_insert(
        &mut self,
        source: MapGetOrInsertValueSource,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let map = self.emit_collection_map_receiver(f)?;
        let key = s.reserve_value_local(f);
        let argument = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let undefined = s.reserve_value_local(f);
        let pending = s.reserve_completion(f);
        let index = s.reserve_i64_local(f);
        let found = s.reserve_i32_local(f);
        self.emit_builtin_arg_to_value(0, &key, f);
        self.emit_builtin_arg_to_value(1, &argument, f);
        undefined.set_undefined(f);
        match source {
            MapGetOrInsertValueSource::ValueArgument => {}
            MapGetOrInsertValueSource::ComputedCallback => self.emit_collection_assert_callable(
                &argument,
                RuntimeErrorMessage::MAP_PROTOTYPE_GETORINSERTCOMPUTED_CALLBACK_MUST_BE_CALLABLE,
                f,
            )?,
        }
        self.emit_collection_normalize_zero(&key, f);
        // Key identity/content and its raw hash survive user callbacks. Keep
        // only that rooted key/hash pair, never a bucket, entry or table view.
        let prepared = self.emit_collection_prepare_key(&key, f);
        self.emit_collection_find_prepared_map(&map, &prepared, index, found, f)?;
        found.load(f);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_collection_read_map(&map, index, &key, &value, f);
        f.instruction(&Instruction::Else);
        match source {
            MapGetOrInsertValueSource::ValueArgument => value.copy_from(&argument, f),
            MapGetOrInsertValueSource::ComputedCallback => {
                let args = self.emit_pre_evaluated_arg_vector(&[&key], f);
                self.emit_function_or_proxy_call_with_argv(
                    &argument, &undefined, &args, &pending, f,
                )?;
                args.clear(f);
                self.emit_collection_propagate(&pending, f);
                value.copy_from(pending.value(), f);
            }
        }
        // put probes the current owner again: a callback may clear, grow,
        // delete/reinsert, or insert this key. Its computed value wins.
        self.emit_collection_put_prepared_map(&map, &prepared, &value, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        prepared.release(self, f);
        self.completion().set_normal(&value, f);
        s.release_i32_local(found, f);
        s.release_i64_local(index, f);
        pending.clear(f);
        undefined.clear(f);
        value.clear(f);
        argument.clear(f);
        key.clear(f);
        map.clear(f);
        Ok(())
    }
}
