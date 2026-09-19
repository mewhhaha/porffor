use super::*;
use crate::functions::{NonArrayRealmIntrinsicSlot, RealmFunctionMaterializationContext};

/// The two Temporal families whose intrinsic allocation is needed by Instant
/// arithmetic. Other created-Realm Temporal families remain separate work.
#[derive(Clone, Copy)]
pub(crate) enum TemporalIntrinsicFamily {
    Instant,
    Duration,
}

pub(crate) enum TemporalIntrinsicRealm<'a> {
    Entry,
    Created(&'a RealmFunctionMaterializationContext),
}

impl TemporalIntrinsicFamily {
    pub(crate) const ALL: [Self; 2] = [Self::Instant, Self::Duration];
    pub(crate) const fn constructor(self) -> StandardBuiltinId {
        match self {
            Self::Instant => StandardBuiltinId::TemporalInstantConstructor,
            Self::Duration => StandardBuiltinId::TemporalDurationConstructor,
        }
    }
    pub(crate) const fn prototype_slot(self) -> NonArrayRealmIntrinsicSlot {
        match self {
            Self::Instant => NonArrayRealmIntrinsicSlot::TemporalInstantPrototype,
            Self::Duration => NonArrayRealmIntrinsicSlot::TemporalDurationPrototype,
        }
    }
    pub(crate) const fn prototype_global(self) -> u32 {
        match self {
            Self::Instant => TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
            Self::Duration => TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
        }
    }
    fn constructor_methods(self) -> &'static [StandardBuiltinId] {
        match self {
            Self::Instant => &[
                StandardBuiltinId::TemporalInstantFrom,
                StandardBuiltinId::TemporalInstantFromEpochMilliseconds,
                StandardBuiltinId::TemporalInstantFromEpochNanoseconds,
                StandardBuiltinId::TemporalInstantCompare,
            ],
            Self::Duration => &[
                StandardBuiltinId::TemporalDurationFrom,
                StandardBuiltinId::TemporalDurationCompare,
            ],
        }
    }
    fn getters(self) -> &'static [StandardBuiltinId] {
        match self {
            Self::Instant => &[
                StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter,
                StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter,
            ],
            Self::Duration => &[
                StandardBuiltinId::TemporalDurationPrototypeYearsGetter,
                StandardBuiltinId::TemporalDurationPrototypeMonthsGetter,
                StandardBuiltinId::TemporalDurationPrototypeWeeksGetter,
                StandardBuiltinId::TemporalDurationPrototypeDaysGetter,
                StandardBuiltinId::TemporalDurationPrototypeHoursGetter,
                StandardBuiltinId::TemporalDurationPrototypeMinutesGetter,
                StandardBuiltinId::TemporalDurationPrototypeSecondsGetter,
                StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter,
                StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter,
                StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter,
                StandardBuiltinId::TemporalDurationPrototypeSignGetter,
                StandardBuiltinId::TemporalDurationPrototypeBlankGetter,
            ],
        }
    }
    fn methods(self) -> &'static [StandardBuiltinId] {
        match self {
            Self::Instant => &[
                StandardBuiltinId::TemporalInstantPrototypeAdd,
                StandardBuiltinId::TemporalInstantPrototypeSubtract,
                StandardBuiltinId::TemporalInstantPrototypeRound,
                StandardBuiltinId::TemporalInstantPrototypeUntil,
                StandardBuiltinId::TemporalInstantPrototypeSince,
                StandardBuiltinId::TemporalInstantPrototypeToString,
                StandardBuiltinId::TemporalInstantPrototypeEquals,
                StandardBuiltinId::TemporalInstantPrototypeToJson,
                StandardBuiltinId::TemporalInstantPrototypeValueOf,
            ],
            Self::Duration => &[
                StandardBuiltinId::TemporalDurationPrototypeWith,
                StandardBuiltinId::TemporalDurationPrototypeNegated,
                StandardBuiltinId::TemporalDurationPrototypeAbs,
                StandardBuiltinId::TemporalDurationPrototypeAdd,
                StandardBuiltinId::TemporalDurationPrototypeSubtract,
                StandardBuiltinId::TemporalDurationPrototypeRound,
                StandardBuiltinId::TemporalDurationPrototypeTotal,
                StandardBuiltinId::TemporalDurationPrototypeToString,
                StandardBuiltinId::TemporalDurationPrototypeToJson,
                StandardBuiltinId::TemporalDurationPrototypeToLocaleString,
                StandardBuiltinId::TemporalDurationPrototypeValueOf,
            ],
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_install_temporal_intrinsic_members(
        &mut self,
        family: TemporalIntrinsicFamily,
        constructor: u32,
        prototype: u32,
        realm: TemporalIntrinsicRealm<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let callable = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let key = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        for builtin in family.constructor_methods() {
            self.emit_temporal_intrinsic_callable(*builtin, &realm, callable, function)?;
            self.emit_object_define_local_data(
                constructor,
                temporal_intrinsic_property_key(*builtin)?,
                callable,
                tag,
                function,
            )?;
        }
        for builtin in family.getters() {
            self.emit_temporal_intrinsic_callable(*builtin, &realm, callable, function)?;
            function.instruction(&Instruction::I64Const(
                self.strings
                    .payload(temporal_intrinsic_property_key(*builtin)?),
            ));
            function.instruction(&Instruction::LocalSet(key));
            self.emit_object_append_accessor_property_with_flags(
                prototype,
                key,
                Some((callable, tag)),
                None,
                false,
                true,
                function,
            )?;
        }
        for builtin in family.methods() {
            self.emit_temporal_intrinsic_callable(*builtin, &realm, callable, function)?;
            self.emit_object_define_local_data(
                prototype,
                temporal_intrinsic_property_key(*builtin)?,
                callable,
                tag,
                function,
            )?;
        }
        self.emit_define_temporal_intrinsic_to_string_tag(
            prototype,
            family.constructor().debug_name(),
            function,
        )?;
        self.release_temp_local(key);
        self.release_temp_local(tag);
        self.release_temp_local(callable);
        Ok(())
    }

    pub(crate) fn emit_temporal_intrinsic_callable(
        &mut self,
        builtin: StandardBuiltinId,
        realm: &TemporalIntrinsicRealm<'_>,
        callable: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self
            .functions
            .get(&builtin.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "missing Temporal intrinsic metadata: {}",
                    builtin.debug_name()
                ))
            })?;
        match realm {
            TemporalIntrinsicRealm::Entry => {
                self.emit_function_value_payload(&meta, function)?;
                function.instruction(&Instruction::LocalSet(callable));
            }
            TemporalIntrinsicRealm::Created(context) => {
                self.emit_function_value_payload_in_realm(&meta, context, callable, function)?;
                self.store_i64_local_at_offset(
                    callable,
                    HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                    callable,
                    function,
                );
            }
        }
        Ok(())
    }

    pub(crate) fn emit_define_temporal_intrinsic_to_string_tag(
        &mut self,
        object: u32,
        name: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(payload));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        let result = self.emit_object_append_data_property_with_flags(
            object, key, payload, tag, false, false, true, function,
        );
        self.release_temp_local(tag);
        self.release_temp_local(payload);
        self.release_temp_local(key);
        result
    }
}
