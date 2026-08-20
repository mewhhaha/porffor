//! `collections` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

#[derive(Clone, Copy)]
enum CollectionPrototypeIntrinsic {
    Map,
    Set,
    WeakMap,
    WeakSet,
}

impl CollectionPrototypeIntrinsic {
    const fn prototype_global_index(self) -> u32 {
        match self {
            Self::Map => MAP_PROTOTYPE_GLOBAL_INDEX,
            Self::Set => SET_PROTOTYPE_GLOBAL_INDEX,
            Self::WeakMap => WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
            Self::WeakSet => WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
        }
    }

    const fn to_string_tag(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Set => "Set",
            Self::WeakMap => "WeakMap",
            Self::WeakSet => "WeakSet",
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_collection_prototype_to_string_tag(
        &mut self,
        intrinsic: CollectionPrototypeIntrinsic,
        prototype_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload(intrinsic.to_string_tag()),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);

        Ok(())
    }

    pub(crate) fn install_map_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let intrinsic = CollectionPrototypeIntrinsic::Map;
        let map_prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let group_by_meta = self
            .functions
            .get(&StandardBuiltinId::MapGroupBy.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Map.groupBy`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "groupBy", &group_by_meta, function)?;
        let species_getter = self
            .functions
            .get(&StandardBuiltinId::MapSpeciesGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Map[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&species_getter, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            object_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(intrinsic.prototype_global_index()));
        function.instruction(&Instruction::LocalSet(map_prototype_local));
        for (name, builtin) in [
            ("clear", StandardBuiltinId::MapPrototypeClear),
            ("delete", StandardBuiltinId::MapPrototypeDelete),
            ("forEach", StandardBuiltinId::MapPrototypeForEach),
            ("get", StandardBuiltinId::MapPrototypeGet),
            ("getOrInsert", StandardBuiltinId::MapPrototypeGetOrInsert),
            (
                "getOrInsertComputed",
                StandardBuiltinId::MapPrototypeGetOrInsertComputed,
            ),
            ("has", StandardBuiltinId::MapPrototypeHas),
            ("keys", StandardBuiltinId::MapPrototypeKeys),
            ("set", StandardBuiltinId::MapPrototypeSet),
            ("values", StandardBuiltinId::MapPrototypeValues),
        ] {
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
            self.emit_object_define_function_data(map_prototype_local, name, &meta, function)?;
        }
        let entries_meta = self
            .functions
            .get(&StandardBuiltinId::MapPrototypeEntries.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Map.prototype.entries`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            map_prototype_local,
            "entries",
            &["Symbol.iterator"],
            &entries_meta,
            function,
        )?;
        let size_getter = self
            .functions
            .get(&StandardBuiltinId::MapPrototypeSizeGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `get Map.prototype.size`",
                )
            })?;
        function.instruction(&Instruction::I64Const(self.strings.payload("size")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&size_getter, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            map_prototype_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;
        self.emit_collection_prototype_to_string_tag(intrinsic, map_prototype_local, function)?;
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(map_prototype_local);

        Ok(())
    }

    pub(crate) fn install_weak_map_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let intrinsic = CollectionPrototypeIntrinsic::WeakMap;
        let weak_map_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(intrinsic.prototype_global_index()));
        function.instruction(&Instruction::LocalSet(weak_map_prototype_local));
        for (name, builtin) in [
            ("delete", StandardBuiltinId::WeakMapPrototypeDelete),
            ("get", StandardBuiltinId::WeakMapPrototypeGet),
            (
                "getOrInsert",
                StandardBuiltinId::WeakMapPrototypeGetOrInsert,
            ),
            (
                "getOrInsertComputed",
                StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed,
            ),
            ("has", StandardBuiltinId::WeakMapPrototypeHas),
            ("set", StandardBuiltinId::WeakMapPrototypeSet),
        ] {
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
            self.emit_object_define_function_data(weak_map_prototype_local, name, &meta, function)?;
        }
        self.emit_collection_prototype_to_string_tag(
            intrinsic,
            weak_map_prototype_local,
            function,
        )?;
        self.release_temp_local(weak_map_prototype_local);

        Ok(())
    }

    pub(crate) fn install_weak_set_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let intrinsic = CollectionPrototypeIntrinsic::WeakSet;
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(intrinsic.prototype_global_index()));
        function.instruction(&Instruction::LocalSet(prototype_local));
        for (name, builtin) in [
            ("add", StandardBuiltinId::WeakSetPrototypeAdd),
            ("delete", StandardBuiltinId::WeakSetPrototypeDelete),
            ("has", StandardBuiltinId::WeakSetPrototypeHas),
        ] {
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
            self.emit_object_define_function_data(prototype_local, name, &meta, function)?;
        }
        self.emit_collection_prototype_to_string_tag(intrinsic, prototype_local, function)?;
        self.release_temp_local(prototype_local);

        Ok(())
    }

    pub(crate) fn install_weak_ref_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let weak_ref_prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let deref_meta = self
            .functions
            .get(&StandardBuiltinId::WeakRefPrototypeDeref.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `WeakRef.prototype.deref`",
                )
            })?;

        function.instruction(&Instruction::GlobalGet(WEAK_REF_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(weak_ref_prototype_local));
        self.emit_object_define_function_data(
            weak_ref_prototype_local,
            "deref",
            &deref_meta,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("WeakRef")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            weak_ref_prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(weak_ref_prototype_local);

        Ok(())
    }

    pub(crate) fn install_finalization_registry_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let register_meta = self
            .functions
            .get(
                &StandardBuiltinId::FinalizationRegistryPrototypeRegister.function_id(),
            )
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `FinalizationRegistry.prototype.register`",
                )
            })?;
        let unregister_meta = self
            .functions
            .get(
                &StandardBuiltinId::FinalizationRegistryPrototypeUnregister.function_id(),
            )
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `FinalizationRegistry.prototype.unregister`",
                )
            })?;

        function.instruction(&Instruction::GlobalGet(
            FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(
            prototype_local,
            "register",
            &register_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_local,
            "unregister",
            &unregister_meta,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("FinalizationRegistry"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);

        Ok(())
    }

    /// `%AsyncDisposableStack.prototype%` (ECMA-262 explicit-resource-management
    /// 27.4.3).
    ///
    /// Two shapes here are load-bearing rather than stylistic:
    ///
    /// * `disposeAsync` is defined *with* `Symbol.asyncDispose` as an alias, not
    ///   as two independent installs, because
    ///   `prototype/Symbol.asyncDispose.js` asserts the two properties hold the
    ///   same function *object*.
    /// * `disposed` is an accessor with no setter, so
    ///   `prototype/disposed/getter.js` sees `typeof descriptor.set ===
    ///   'undefined'`.
    pub(crate) fn install_async_disposable_stack_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(
            ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        for (name, builtin) in [
            (
                "adopt",
                StandardBuiltinId::AsyncDisposableStackPrototypeAdopt,
            ),
            (
                "defer",
                StandardBuiltinId::AsyncDisposableStackPrototypeDefer,
            ),
            ("move", StandardBuiltinId::AsyncDisposableStackPrototypeMove),
            ("use", StandardBuiltinId::AsyncDisposableStackPrototypeUse),
        ] {
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
            self.emit_object_define_function_data(prototype_local, name, &meta, function)?;
        }
        let dispose_async_meta = self
            .functions
            .get(&StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `AsyncDisposableStack.prototype.disposeAsync`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            prototype_local,
            "disposeAsync",
            &["Symbol.asyncDispose"],
            &dispose_async_meta,
            function,
        )?;
        let disposed_getter = self
            .functions
            .get(&StandardBuiltinId::AsyncDisposableStackPrototypeDisposedGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `get AsyncDisposableStack.prototype.disposed`",
                )
            })?;
        function.instruction(&Instruction::I64Const(self.strings.payload("disposed")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&disposed_getter, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            prototype_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("AsyncDisposableStack"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);

        Ok(())
    }

    pub(crate) fn install_set_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let intrinsic = CollectionPrototypeIntrinsic::Set;
        let set_prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let species_getter = self
            .functions
            .get(&StandardBuiltinId::SetSpeciesGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Set[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&species_getter, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            object_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(intrinsic.prototype_global_index()));
        function.instruction(&Instruction::LocalSet(set_prototype_local));
        for (name, builtin) in [
            ("add", StandardBuiltinId::SetPrototypeAdd),
            ("clear", StandardBuiltinId::SetPrototypeClear),
            ("delete", StandardBuiltinId::SetPrototypeDelete),
            ("difference", StandardBuiltinId::SetPrototypeDifference),
            ("forEach", StandardBuiltinId::SetPrototypeForEach),
            ("has", StandardBuiltinId::SetPrototypeHas),
            ("intersection", StandardBuiltinId::SetPrototypeIntersection),
            (
                "isDisjointFrom",
                StandardBuiltinId::SetPrototypeIsDisjointFrom,
            ),
            ("isSubsetOf", StandardBuiltinId::SetPrototypeIsSubsetOf),
            ("isSupersetOf", StandardBuiltinId::SetPrototypeIsSupersetOf),
            (
                "symmetricDifference",
                StandardBuiltinId::SetPrototypeSymmetricDifference,
            ),
            ("union", StandardBuiltinId::SetPrototypeUnion),
        ] {
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
            self.emit_object_define_function_data(set_prototype_local, name, &meta, function)?;
        }
        let values_meta = self
            .functions
            .get(&StandardBuiltinId::SetPrototypeValues.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Set.prototype.values`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            set_prototype_local,
            "values",
            &["keys", "Symbol.iterator"],
            &values_meta,
            function,
        )?;
        let entries_meta = self
            .functions
            .get(&StandardBuiltinId::SetPrototypeEntries.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Set.prototype.entries`",
                )
            })?;
        self.emit_object_define_function_data(
            set_prototype_local,
            "entries",
            &entries_meta,
            function,
        )?;
        let size_getter = self
            .functions
            .get(&StandardBuiltinId::SetPrototypeSizeGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `get Set.prototype.size`",
                )
            })?;
        function.instruction(&Instruction::I64Const(self.strings.payload("size")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(&size_getter, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            set_prototype_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;
        self.emit_collection_prototype_to_string_tag(intrinsic, set_prototype_local, function)?;
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(set_prototype_local);

        Ok(())
    }
}
