use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn init_builtin_constructor_object(
        &mut self,
        builtin: StandardBuiltinId,
        prototype_global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                builtin.debug_name()
            ))
        })?;
        let constructor_global_index =
            standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin constructor global `{}`",
                    builtin.debug_name()
                ))
            })?;
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let prototype_object_local = self.reserve_temp_local();

        // Bundled once so a family installer in `intrinsics/` receives the same
        // values this function computed, without each extraction growing a
        // nine-parameter signature. See `intrinsics::IntrinsicInstall`.
        let intrinsic_context = IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        };

        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(constructor_global_index));

        if builtin.constructable() && !matches!(builtin, StandardBuiltinId::BigIntConstructor) {
            if matches!(
                builtin,
                StandardBuiltinId::EvalErrorConstructor
                    | StandardBuiltinId::AggregateErrorConstructor
                    | StandardBuiltinId::SuppressedErrorConstructor
                    | StandardBuiltinId::RangeErrorConstructor
                    | StandardBuiltinId::SyntaxErrorConstructor
                    | StandardBuiltinId::TypeErrorConstructor
                    | StandardBuiltinId::URIErrorConstructor
                    | StandardBuiltinId::ReferenceErrorConstructor
            ) {
                function.instruction(&Instruction::GlobalGet(ERROR_CONSTRUCTOR_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_PROTOTYPE_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
            }
            if is_typed_array_constructor(builtin) {
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                    object_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
                    typed_array_bytes_per_element(builtin),
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
                    typed_array_element_kind(builtin),
                    function,
                );
                function.instruction(&Instruction::GlobalGet(
                    TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_PROTOTYPE_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Object.tag() as u64,
                    function,
                );
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                    prototype_object_local,
                    function,
                );
                self.emit_object_define_number_data_from_f64_const_with_flags(
                    object_local,
                    "BYTES_PER_ELEMENT",
                    typed_array_bytes_per_element(builtin) as f64,
                    false,
                    false,
                    false,
                    function,
                )?;
                self.emit_object_define_number_data_from_f64_const_with_flags(
                    prototype_object_local,
                    "BYTES_PER_ELEMENT",
                    typed_array_bytes_per_element(builtin) as f64,
                    false,
                    false,
                    false,
                    function,
                )?;
            } else {
                let prototype_kind = if builtin == StandardBuiltinId::ArrayConstructor {
                    ValueKind::Array
                } else {
                    ValueKind::Object
                };
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                    prototype_kind.tag() as u64,
                    function,
                );
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(prototype_kind.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_define_data_with_configurable(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    false,
                    false,
                    false,
                    function,
                )?;

                if builtin != StandardBuiltinId::IteratorConstructor {
                    function
                        .instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::LocalGet(object_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::GlobalGet(prototype_global_index));
                    function.instruction(&Instruction::LocalSet(prototype_object_local));
                    self.emit_object_append_data_property_with_flags(
                        prototype_object_local,
                        key_local,
                        payload_local,
                        tag_local,
                        true,
                        false,
                        true,
                        function,
                    )?;
                }
            }
        }

        match builtin {
            StandardBuiltinId::FunctionConstructor => {
                self.install_function_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::PromiseConstructor => {
                self.install_promise_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::PromisePrototypeThen
            | StandardBuiltinId::PromisePrototypeCatch
            | StandardBuiltinId::PromisePrototypeFinally
            | StandardBuiltinId::PromiseThenFinally
            | StandardBuiltinId::PromiseCatchFinally
            | StandardBuiltinId::PromiseValueThunk
            | StandardBuiltinId::PromiseThrower
            | StandardBuiltinId::PromiseSpeciesGetter
            | StandardBuiltinId::PromiseResolve
            | StandardBuiltinId::PromiseWithResolvers
            | StandardBuiltinId::PromiseTry
            | StandardBuiltinId::PromiseReject
            | StandardBuiltinId::PromiseAll
            | StandardBuiltinId::PromiseAllSettled
            | StandardBuiltinId::PromiseAllKeyed
            | StandardBuiltinId::PromiseAllSettledKeyed
            | StandardBuiltinId::PromiseAny
            | StandardBuiltinId::PromiseRace
            | StandardBuiltinId::PromiseAllResolveElement
            | StandardBuiltinId::PromiseAllSettledResolveElement
            | StandardBuiltinId::PromiseAllSettledRejectElement
            | StandardBuiltinId::PromiseAnyRejectElement
            | StandardBuiltinId::PromiseAllKeyedResolveElement
            | StandardBuiltinId::PromiseAllSettledKeyedResolveElement
            | StandardBuiltinId::PromiseAllSettledKeyedRejectElement
            | StandardBuiltinId::PromiseCapabilityExecutor
            | StandardBuiltinId::PromiseResolveFunction
            | StandardBuiltinId::PromiseRejectFunction
            | StandardBuiltinId::ArrayFromAsyncFulfilled
            | StandardBuiltinId::ArrayFromAsyncRejected => {}
            StandardBuiltinId::MapConstructor => {
                self.install_map_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::MapPrototypeClear
            | StandardBuiltinId::MapPrototypeDelete
            | StandardBuiltinId::MapPrototypeForEach
            | StandardBuiltinId::MapPrototypeKeys
            | StandardBuiltinId::MapPrototypeValues
            | StandardBuiltinId::MapPrototypeEntries
            | StandardBuiltinId::MapIteratorNext
            | StandardBuiltinId::MapPrototypeGet
            | StandardBuiltinId::MapPrototypeGetOrInsert
            | StandardBuiltinId::MapPrototypeGetOrInsertComputed
            | StandardBuiltinId::MapPrototypeHas
            | StandardBuiltinId::MapPrototypeSet
            | StandardBuiltinId::MapPrototypeSizeGetter
            | StandardBuiltinId::MapSpeciesGetter => {}
            StandardBuiltinId::WeakMapConstructor => {
                self.install_weak_map_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::WeakMapPrototypeDelete
            | StandardBuiltinId::WeakMapPrototypeGet
            | StandardBuiltinId::WeakMapPrototypeGetOrInsert
            | StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed
            | StandardBuiltinId::WeakMapPrototypeHas
            | StandardBuiltinId::WeakMapPrototypeSet => {}
            StandardBuiltinId::WeakSetConstructor => {
                self.install_weak_set_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::WeakSetPrototypeAdd
            | StandardBuiltinId::WeakSetPrototypeDelete
            | StandardBuiltinId::WeakSetPrototypeHas => {}
            StandardBuiltinId::WeakRefConstructor => {
                self.install_weak_ref_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::WeakRefPrototypeDeref => {}
            StandardBuiltinId::FinalizationRegistryConstructor => self
                .install_finalization_registry_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::FinalizationRegistryPrototypeRegister
            | StandardBuiltinId::FinalizationRegistryPrototypeUnregister => {}
            StandardBuiltinId::AsyncDisposableStackConstructor => self
                .install_async_disposable_stack_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::AsyncDisposableStackPrototypeUse
            | StandardBuiltinId::AsyncDisposableStackPrototypeAdopt
            | StandardBuiltinId::AsyncDisposableStackPrototypeDefer
            | StandardBuiltinId::AsyncDisposableStackPrototypeMove
            | StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync
            | StandardBuiltinId::AsyncDisposableStackPrototypeDisposedGetter
            | StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
            | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected => {}
            StandardBuiltinId::SetConstructor => {
                self.install_set_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::SetPrototypeAdd
            | StandardBuiltinId::SetPrototypeClear
            | StandardBuiltinId::SetPrototypeDelete
            | StandardBuiltinId::SetPrototypeDifference
            | StandardBuiltinId::SetPrototypeForEach
            | StandardBuiltinId::SetPrototypeIntersection
            | StandardBuiltinId::SetPrototypeIsDisjointFrom
            | StandardBuiltinId::SetPrototypeIsSubsetOf
            | StandardBuiltinId::SetPrototypeIsSupersetOf
            | StandardBuiltinId::SetPrototypeSymmetricDifference
            | StandardBuiltinId::SetPrototypeUnion
            | StandardBuiltinId::SetPrototypeValues
            | StandardBuiltinId::SetPrototypeEntries
            | StandardBuiltinId::SetIteratorNext
            | StandardBuiltinId::SetPrototypeHas
            | StandardBuiltinId::SetPrototypeSizeGetter
            | StandardBuiltinId::SetSpeciesGetter => {}
            StandardBuiltinId::ObjectConstructor => {
                self.install_object_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::ProxyConstructor => {
                self.install_proxy_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::RegExpConstructor => {
                self.install_regexp_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::IteratorConstructor => {
                self.install_iterator_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::ArrayConstructor => {
                self.install_array_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::StringConstructor => {
                self.install_string_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::ArrayBufferConstructor
            | StandardBuiltinId::SharedArrayBufferConstructor => {
                self.install_array_buffer_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::DataViewConstructor => {
                self.install_data_view_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::TemporalInstantConstructor => {
                self.install_temporal_instant_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::TemporalZonedDateTimeConstructor => self
                .install_temporal_zoned_date_time_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::TemporalPlainDateConstructor => self
                .install_temporal_plain_date_constructor_intrinsics(&intrinsic_context, function)?,
            StandardBuiltinId::TemporalDurationConstructor => {
                self.install_temporal_duration_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::TemporalPlainTimeConstructor => self
                .install_temporal_plain_time_constructor_intrinsics(&intrinsic_context, function)?,
            StandardBuiltinId::TemporalPlainDateTimeConstructor => self
                .install_temporal_plain_date_time_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::TemporalPlainYearMonthConstructor => self
                .install_temporal_plain_year_month_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::TemporalPlainMonthDayConstructor => self
                .install_temporal_plain_month_day_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::IntlLocaleConstructor => {
                self.install_intl_locale_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::IntlDateTimeFormatConstructor => self
                .install_intl_date_time_format_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinId::DateConstructor => {
                self.install_date_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::ErrorConstructor => {
                self.install_error_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::BigIntConstructor => {
                self.install_big_int_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::SymbolConstructor => {
                self.install_symbol_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::NumberConstructor => {
                self.install_number_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::BooleanConstructor => {
                self.install_boolean_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinId::BigIntAsIntN
            | StandardBuiltinId::BigIntAsUintN
            | StandardBuiltinId::BigIntPrototypeToString
            | StandardBuiltinId::BigIntPrototypeToLocaleString
            | StandardBuiltinId::BigIntPrototypeValueOf
            | StandardBuiltinId::Float64ArrayConstructor
            | StandardBuiltinId::Float32ArrayConstructor
            | StandardBuiltinId::Int32ArrayConstructor
            | StandardBuiltinId::Int16ArrayConstructor
            | StandardBuiltinId::Int8ArrayConstructor
            | StandardBuiltinId::Uint32ArrayConstructor
            | StandardBuiltinId::Uint16ArrayConstructor
            | StandardBuiltinId::Uint8ArrayConstructor
            | StandardBuiltinId::Uint8ClampedArrayConstructor
            | StandardBuiltinId::BigInt64ArrayConstructor
            | StandardBuiltinId::BigUint64ArrayConstructor
            | StandardBuiltinId::ArrayBufferSpeciesGetter
            | StandardBuiltinId::RegExpSpeciesGetter
            | StandardBuiltinId::EvalErrorConstructor
            | StandardBuiltinId::AggregateErrorConstructor
            | StandardBuiltinId::SuppressedErrorConstructor
            | StandardBuiltinId::RangeErrorConstructor
            | StandardBuiltinId::SyntaxErrorConstructor
            | StandardBuiltinId::TypeErrorConstructor
            | StandardBuiltinId::URIErrorConstructor
            | StandardBuiltinId::ReferenceErrorConstructor
            | StandardBuiltinId::ErrorIsError
            | StandardBuiltinId::FunctionPrototypeCall
            | StandardBuiltinId::FunctionPrototypeApply
            | StandardBuiltinId::FunctionPrototypeBind
            | StandardBuiltinId::FunctionPrototypeToString
            | StandardBuiltinId::StringPrototypeToString
            | StandardBuiltinId::StringPrototypeValueOf
            | StandardBuiltinId::StringPrototypeCharAt
            | StandardBuiltinId::StringPrototypeConcat
            | StandardBuiltinId::StringPrototypeCharCodeAt
            | StandardBuiltinId::StringPrototypeCodePointAt
            | StandardBuiltinId::StringPrototypeAt
            | StandardBuiltinId::ObjectCreate
            | StandardBuiltinId::ObjectGetPrototypeOf
            | StandardBuiltinId::ObjectSetPrototypeOf
            | StandardBuiltinId::ObjectDefineProperty
            | StandardBuiltinId::ObjectDefineProperties
            | StandardBuiltinId::ObjectGetOwnPropertyDescriptor
            | StandardBuiltinId::ObjectGetOwnPropertyDescriptors
            | StandardBuiltinId::ObjectAssign
            | StandardBuiltinId::ObjectGetOwnPropertyNames
            | StandardBuiltinId::ObjectGetOwnPropertySymbols
            | StandardBuiltinId::ObjectKeys
            | StandardBuiltinId::ObjectValues
            | StandardBuiltinId::ObjectEntries
            | StandardBuiltinId::ObjectHasOwn
            | StandardBuiltinId::ObjectIs
            | StandardBuiltinId::ObjectIsSealed
            | StandardBuiltinId::ObjectIsFrozen
            | StandardBuiltinId::ObjectSeal
            | StandardBuiltinId::ObjectFreeze
            | StandardBuiltinId::ObjectIsExtensible
            | StandardBuiltinId::ObjectPreventExtensions
            | StandardBuiltinId::ObjectPrototypeHasOwnProperty
            | StandardBuiltinId::ObjectPrototypeLookupGetter
            | StandardBuiltinId::ObjectPrototypeLookupSetter
            | StandardBuiltinId::ObjectPrototypePropertyIsEnumerable
            | StandardBuiltinId::ObjectPrototypeIsPrototypeOf
            | StandardBuiltinId::ObjectPrototypeToString
            | StandardBuiltinId::ObjectPrototypeToLocaleString
            | StandardBuiltinId::ObjectPrototypeValueOf
            | StandardBuiltinId::ProxyRevocable
            | StandardBuiltinId::ProxyRevoke
            | StandardBuiltinId::ReflectConstruct
            | StandardBuiltinId::ReflectApply
            | StandardBuiltinId::ReflectGet
            | StandardBuiltinId::ReflectGetPrototypeOf
            | StandardBuiltinId::ReflectGetOwnPropertyDescriptor
            | StandardBuiltinId::ReflectSet
            | StandardBuiltinId::ReflectHas
            | StandardBuiltinId::ReflectDefineProperty
            | StandardBuiltinId::ReflectDeleteProperty
            | StandardBuiltinId::ReflectIsExtensible
            | StandardBuiltinId::ReflectPreventExtensions
            | StandardBuiltinId::ReflectSetPrototypeOf
            | StandardBuiltinId::ReflectOwnKeys
            | StandardBuiltinId::ArrayFrom
            | StandardBuiltinId::ArrayFromAsync
            | StandardBuiltinId::ArrayIsArray
            | StandardBuiltinId::NumberIsInteger
            | StandardBuiltinId::NumberIsSafeInteger
            | StandardBuiltinId::NumberIsFinite
            | StandardBuiltinId::NumberIsNaN
            | StandardBuiltinId::NumberPrototypeToExponential
            | StandardBuiltinId::NumberPrototypeToFixed
            | StandardBuiltinId::NumberPrototypeToPrecision
            | StandardBuiltinId::NumberPrototypeToString
            | StandardBuiltinId::NumberPrototypeToLocaleString
            | StandardBuiltinId::NumberPrototypeValueOf
            | StandardBuiltinId::BooleanPrototypeToString
            | StandardBuiltinId::BooleanPrototypeValueOf
            | StandardBuiltinId::GlobalIsFinite
            | StandardBuiltinId::GlobalIsNaN
            | StandardBuiltinId::MathAbs
            | StandardBuiltinId::MathAcos
            | StandardBuiltinId::MathAcosh
            | StandardBuiltinId::MathAsin
            | StandardBuiltinId::MathAsinh
            | StandardBuiltinId::MathAtan
            | StandardBuiltinId::MathAtan2
            | StandardBuiltinId::MathAtanh
            | StandardBuiltinId::MathCbrt
            | StandardBuiltinId::MathCeil
            | StandardBuiltinId::MathClz32
            | StandardBuiltinId::MathCos
            | StandardBuiltinId::MathCosh
            | StandardBuiltinId::MathExp
            | StandardBuiltinId::MathExpm1
            | StandardBuiltinId::MathF16Round
            | StandardBuiltinId::MathFloor
            | StandardBuiltinId::MathFround
            | StandardBuiltinId::MathHypot
            | StandardBuiltinId::MathImul
            | StandardBuiltinId::MathLog
            | StandardBuiltinId::MathLog10
            | StandardBuiltinId::MathLog1p
            | StandardBuiltinId::MathLog2
            | StandardBuiltinId::MathPow
            | StandardBuiltinId::MathRandom
            | StandardBuiltinId::MathRound
            | StandardBuiltinId::MathSign
            | StandardBuiltinId::MathSin
            | StandardBuiltinId::MathSinh
            | StandardBuiltinId::MathSqrt
            | StandardBuiltinId::MathSumPrecise
            | StandardBuiltinId::MathTan
            | StandardBuiltinId::MathTanh
            | StandardBuiltinId::MathTrunc
            | StandardBuiltinId::MathMin
            | StandardBuiltinId::MathMax
            | StandardBuiltinId::ArrayPrototypeConcat
            | StandardBuiltinId::ArrayPrototypeJoin
            | StandardBuiltinId::ArrayPrototypeSlice
            | StandardBuiltinId::ArrayPrototypeSplice
            | StandardBuiltinId::ArrayPrototypeSort
            | StandardBuiltinId::ArrayPrototypeToLocaleString
            | StandardBuiltinId::ArrayPrototypeFlat
            | StandardBuiltinId::ArrayPrototypeFlatMap
            | StandardBuiltinId::ArrayPrototypeAt
            | StandardBuiltinId::TypedArrayPrototypeAt
            | StandardBuiltinId::TypedArrayPrototypeIncludes
            | StandardBuiltinId::TypedArrayPrototypeIndexOf
            | StandardBuiltinId::TypedArrayPrototypeLastIndexOf
            | StandardBuiltinId::TypedArrayPrototypeFind
            | StandardBuiltinId::TypedArrayPrototypeFindIndex
            | StandardBuiltinId::TypedArrayPrototypeFindLast
            | StandardBuiltinId::TypedArrayPrototypeFindLastIndex
            | StandardBuiltinId::TypedArrayPrototypeEvery
            | StandardBuiltinId::TypedArrayPrototypeSome
            | StandardBuiltinId::TypedArrayPrototypeMap
            | StandardBuiltinId::TypedArrayPrototypeFilter
            | StandardBuiltinId::TypedArrayPrototypeForEach
            | StandardBuiltinId::TypedArrayPrototypeReduce
            | StandardBuiltinId::TypedArrayPrototypeReduceRight
            | StandardBuiltinId::ArrayPrototypeToReversed
            | StandardBuiltinId::ArrayPrototypeToSpliced
            | StandardBuiltinId::ArrayPrototypeToSorted
            | StandardBuiltinId::ArrayPrototypeWith
            | StandardBuiltinId::ArrayPrototypeReverse
            | StandardBuiltinId::ArrayPrototypeCopyWithin
            | StandardBuiltinId::ArrayPrototypeIncludes
            | StandardBuiltinId::ArrayPrototypeIndexOf
            | StandardBuiltinId::ArrayPrototypeLastIndexOf
            | StandardBuiltinId::ArrayPrototypeFind
            | StandardBuiltinId::ArrayPrototypeFindIndex
            | StandardBuiltinId::ArrayPrototypeFindLast
            | StandardBuiltinId::ArrayPrototypeFindLastIndex
            | StandardBuiltinId::ArrayPrototypeEvery
            | StandardBuiltinId::ArrayPrototypeSome
            | StandardBuiltinId::ArrayPrototypeForEach
            | StandardBuiltinId::ArrayPrototypeFilter
            | StandardBuiltinId::ArrayPrototypeMap
            | StandardBuiltinId::ArrayPrototypeReduce
            | StandardBuiltinId::ArrayPrototypeReduceRight
            | StandardBuiltinId::ArrayPrototypePop
            | StandardBuiltinId::ArrayPrototypePush
            | StandardBuiltinId::ArrayPrototypeShift
            | StandardBuiltinId::ArrayPrototypeUnshift
            | StandardBuiltinId::ArrayPrototypeFill
            | StandardBuiltinId::ArrayPrototypeKeys
            | StandardBuiltinId::ArrayPrototypeEntries
            | StandardBuiltinId::ArrayPrototypeValues
            | StandardBuiltinId::ArrayIteratorNext
            | StandardBuiltinId::ArrayIteratorIdentity
            | StandardBuiltinId::StringIteratorNext
            | StandardBuiltinId::IteratorFrom
            | StandardBuiltinId::IteratorHelperNext
            | StandardBuiltinId::IteratorHelperReturn
            | StandardBuiltinId::IteratorPrototypeToArray
            | StandardBuiltinId::IteratorPrototypeForEach
            | StandardBuiltinId::IteratorPrototypeEvery
            | StandardBuiltinId::IteratorPrototypeSome
            | StandardBuiltinId::IteratorPrototypeFind
            | StandardBuiltinId::IteratorPrototypeReduce
            | StandardBuiltinId::IteratorPrototypeMap
            | StandardBuiltinId::IteratorMapNext
            | StandardBuiltinId::IteratorMapReturn
            | StandardBuiltinId::IteratorPrototypeFilter
            | StandardBuiltinId::IteratorFilterNext
            | StandardBuiltinId::IteratorFilterReturn
            | StandardBuiltinId::IteratorPrototypeFlatMap
            | StandardBuiltinId::IteratorFlatMapNext
            | StandardBuiltinId::IteratorFlatMapReturn
            | StandardBuiltinId::IteratorPrototypeTake
            | StandardBuiltinId::IteratorTakeNext
            | StandardBuiltinId::IteratorTakeReturn
            | StandardBuiltinId::IteratorPrototypeDrop
            | StandardBuiltinId::IteratorDropNext
            | StandardBuiltinId::IteratorDropReturn
            | StandardBuiltinId::IteratorPrototypeConstructorGetter
            | StandardBuiltinId::IteratorPrototypeConstructorSetter
            | StandardBuiltinId::IteratorPrototypeSymbolDispose
            | StandardBuiltinId::IteratorPrototypeToStringTagGetter
            | StandardBuiltinId::IteratorPrototypeToStringTagSetter
            | StandardBuiltinId::IteratorFromWrapperNext
            | StandardBuiltinId::IteratorFromWrapperReturn
            | StandardBuiltinId::ArrayBufferIsView
            | StandardBuiltinId::ArrayBufferPrototypeDetachedGetter
            | StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter
            | StandardBuiltinId::ArrayBufferPrototypeResizableGetter
            | StandardBuiltinId::ArrayBufferPrototypeResize
            | StandardBuiltinId::ArrayBufferPrototypeSlice
            | StandardBuiltinId::SharedArrayBufferPrototypeSlice
            | StandardBuiltinId::ArrayBufferPrototypeTransfer
            | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
            | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
            | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable
            | StandardBuiltinId::DataViewPrototypeGetUint8
            | StandardBuiltinId::DataViewPrototypeSetUint8
            | StandardBuiltinId::DataViewPrototypeGetInt8
            | StandardBuiltinId::DataViewPrototypeSetInt8
            | StandardBuiltinId::DataViewPrototypeGetUint16
            | StandardBuiltinId::DataViewPrototypeSetUint16
            | StandardBuiltinId::DataViewPrototypeGetInt16
            | StandardBuiltinId::DataViewPrototypeSetInt16
            | StandardBuiltinId::DataViewPrototypeGetUint32
            | StandardBuiltinId::DataViewPrototypeSetUint32
            | StandardBuiltinId::DataViewPrototypeGetInt32
            | StandardBuiltinId::DataViewPrototypeSetInt32
            | StandardBuiltinId::DataViewPrototypeGetFloat16
            | StandardBuiltinId::DataViewPrototypeSetFloat16
            | StandardBuiltinId::DataViewPrototypeGetFloat32
            | StandardBuiltinId::DataViewPrototypeSetFloat32
            | StandardBuiltinId::DataViewPrototypeGetFloat64
            | StandardBuiltinId::DataViewPrototypeSetFloat64
            | StandardBuiltinId::DataViewPrototypeGetBigInt64
            | StandardBuiltinId::DataViewPrototypeSetBigInt64
            | StandardBuiltinId::DataViewPrototypeGetBigUint64
            | StandardBuiltinId::DataViewPrototypeSetBigUint64
            | StandardBuiltinId::DataViewPrototypeBufferGetter
            | StandardBuiltinId::DataViewPrototypeByteLengthGetter
            | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeBufferGetter
            | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeToStringTagGetter
            | StandardBuiltinId::TypedArrayPrototypeToString
            | StandardBuiltinId::TypedArrayPrototypeValues
            | StandardBuiltinId::TypedArrayPrototypeKeys
            | StandardBuiltinId::TypedArrayPrototypeEntries
            | StandardBuiltinId::TypedArrayPrototypeJoin
            | StandardBuiltinId::TypedArrayPrototypeToLocaleString
            | StandardBuiltinId::TypedArrayPrototypeSubarray
            | StandardBuiltinId::TypedArrayPrototypeSlice
            | StandardBuiltinId::TypedArrayPrototypeSet
            | StandardBuiltinId::TypedArrayPrototypeReverse
            | StandardBuiltinId::TypedArrayPrototypeCopyWithin
            | StandardBuiltinId::TypedArrayPrototypeSort
            | StandardBuiltinId::TypedArrayPrototypeToReversed
            | StandardBuiltinId::TypedArrayPrototypeToSorted
            | StandardBuiltinId::TypedArrayPrototypeWith
            | StandardBuiltinId::TypedArrayFrom
            | StandardBuiltinId::TypedArrayOf
            | StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeGrow
            | StandardBuiltinId::ArraySpeciesGetter
            | StandardBuiltinId::TypedArraySpeciesGetter
            | StandardBuiltinId::StringPrototypeAnchor
            | StandardBuiltinId::StringPrototypeBig
            | StandardBuiltinId::StringPrototypeBlink
            | StandardBuiltinId::StringPrototypeBold
            | StandardBuiltinId::StringPrototypeFixed
            | StandardBuiltinId::StringPrototypeFontcolor
            | StandardBuiltinId::StringPrototypeFontsize
            | StandardBuiltinId::StringPrototypeItalics
            | StandardBuiltinId::StringPrototypeLink
            | StandardBuiltinId::StringPrototypeSmall
            | StandardBuiltinId::StringPrototypeStrike
            | StandardBuiltinId::StringPrototypeSub
            | StandardBuiltinId::StringPrototypeSubstr
            | StandardBuiltinId::StringPrototypeSubstring
            | StandardBuiltinId::StringPrototypeSup
            | StandardBuiltinId::StringPrototypeMatch
            | StandardBuiltinId::StringPrototypeMatchAll
            | StandardBuiltinId::StringPrototypeReplace
            | StandardBuiltinId::StringPrototypeReplaceAll
            | StandardBuiltinId::StringPrototypeSearch
            | StandardBuiltinId::StringPrototypeIndexOf
            | StandardBuiltinId::StringPrototypeLastIndexOf
            | StandardBuiltinId::StringPrototypeSlice
            | StandardBuiltinId::StringPrototypeSplit
            | StandardBuiltinId::StringPrototypePadStart
            | StandardBuiltinId::StringPrototypePadEnd
            | StandardBuiltinId::StringPrototypeRepeat
            | StandardBuiltinId::RegExpPrototypeFlagsGetter
            | StandardBuiltinId::RegExpPrototypeSourceGetter
            | StandardBuiltinId::RegExpPrototypeHasIndicesGetter
            | StandardBuiltinId::RegExpPrototypeGlobalGetter
            | StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter
            | StandardBuiltinId::RegExpPrototypeMultilineGetter
            | StandardBuiltinId::RegExpPrototypeDotAllGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter
            | StandardBuiltinId::RegExpPrototypeStickyGetter
            | StandardBuiltinId::RegExpPrototypeCompile
            | StandardBuiltinId::RegExpPrototypeExec
            | StandardBuiltinId::RegExpPrototypeTest
            | StandardBuiltinId::RegExpPrototypeToString
            | StandardBuiltinId::RegExpPrototypeSymbolMatch
            | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
            | StandardBuiltinId::RegExpPrototypeSymbolReplace
            | StandardBuiltinId::RegExpPrototypeSymbolSearch
            | StandardBuiltinId::RegExpPrototypeSymbolSplit
            | StandardBuiltinId::StringPrototypeEndsWith
            | StandardBuiltinId::StringPrototypeIncludes
            | StandardBuiltinId::StringPrototypeStartsWith
            | StandardBuiltinId::StringPrototypeNormalize
            | StandardBuiltinId::StringPrototypeLocaleCompare
            | StandardBuiltinId::StringPrototypeIterator
            | StandardBuiltinId::StringPrototypeToLocaleLowerCase
            | StandardBuiltinId::StringPrototypeToLocaleUpperCase
            | StandardBuiltinId::StringPrototypeToLowerCase
            | StandardBuiltinId::StringPrototypeToUpperCase
            | StandardBuiltinId::StringPrototypeTrim
            | StandardBuiltinId::StringPrototypeTrimStart
            | StandardBuiltinId::StringPrototypeTrimEnd
            | StandardBuiltinId::StringPrototypeIsWellFormed
            | StandardBuiltinId::StringPrototypeToWellFormed
            | StandardBuiltinId::DateNow
            | StandardBuiltinId::DateParse
            | StandardBuiltinId::DatePrototypeGetTime
            | StandardBuiltinId::DatePrototypeSetTime
            | StandardBuiltinId::DatePrototypeValueOf
            | StandardBuiltinId::DatePrototypeGetFullYear
            | StandardBuiltinId::DatePrototypeGetUtcFullYear
            | StandardBuiltinId::DatePrototypeGetMonth
            | StandardBuiltinId::DatePrototypeGetUtcMonth
            | StandardBuiltinId::DatePrototypeGetDate
            | StandardBuiltinId::DatePrototypeGetUtcDate
            | StandardBuiltinId::DatePrototypeGetDay
            | StandardBuiltinId::DatePrototypeGetUtcDay
            | StandardBuiltinId::DatePrototypeGetHours
            | StandardBuiltinId::DatePrototypeGetUtcHours
            | StandardBuiltinId::DatePrototypeGetMinutes
            | StandardBuiltinId::DatePrototypeGetUtcMinutes
            | StandardBuiltinId::DatePrototypeGetSeconds
            | StandardBuiltinId::DatePrototypeGetUtcSeconds
            | StandardBuiltinId::DatePrototypeGetMilliseconds
            | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeGetTimezoneOffset
            | StandardBuiltinId::DatePrototypeGetYear
            | StandardBuiltinId::DatePrototypeSetYear
            | StandardBuiltinId::DatePrototypeSetFullYear
            | StandardBuiltinId::DatePrototypeSetUtcFullYear
            | StandardBuiltinId::DatePrototypeSetMonth
            | StandardBuiltinId::DatePrototypeSetUtcMonth
            | StandardBuiltinId::DatePrototypeSetDate
            | StandardBuiltinId::DatePrototypeSetUtcDate
            | StandardBuiltinId::DatePrototypeSetHours
            | StandardBuiltinId::DatePrototypeSetUtcHours
            | StandardBuiltinId::DatePrototypeSetMinutes
            | StandardBuiltinId::DatePrototypeSetUtcMinutes
            | StandardBuiltinId::DatePrototypeSetSeconds
            | StandardBuiltinId::DatePrototypeSetUtcSeconds
            | StandardBuiltinId::DatePrototypeSetMilliseconds
            | StandardBuiltinId::DatePrototypeSetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeToIsoString
            | StandardBuiltinId::DatePrototypeToJson
            | StandardBuiltinId::DatePrototypeToDateString
            | StandardBuiltinId::DatePrototypeToLocaleDateString
            | StandardBuiltinId::DatePrototypeToLocaleString
            | StandardBuiltinId::DatePrototypeToLocaleTimeString
            | StandardBuiltinId::DatePrototypeToTemporalInstant
            | StandardBuiltinId::DatePrototypeToTimeString
            | StandardBuiltinId::DatePrototypeToString
            | StandardBuiltinId::DatePrototypeToUtcString
            | StandardBuiltinId::DateUtc
            | StandardBuiltinId::ArrayOf
            | StandardBuiltinId::IteratorConcat
            | StandardBuiltinId::IteratorConcatNext
            | StandardBuiltinId::IteratorConcatReturn
            | StandardBuiltinId::IteratorZip
            | StandardBuiltinId::IteratorZipKeyed
            | StandardBuiltinId::IteratorZipNext
            | StandardBuiltinId::IteratorZipReturn
            | StandardBuiltinId::ErrorPrototypeToString
            | StandardBuiltinId::BoundFunctionInvoker
            | StandardBuiltinId::RegExpLegacyStaticGetter
            | StandardBuiltinId::RegExpLegacyStaticSetter
            | StandardBuiltinId::RegExpEscape
            | StandardBuiltinId::JsonParse
            | StandardBuiltinId::JsonStringify
            | StandardBuiltinId::JsonRawJson
            | StandardBuiltinId::JsonIsRawJson
            | StandardBuiltinId::AtomicsAdd
            | StandardBuiltinId::AtomicsAnd
            | StandardBuiltinId::AtomicsCompareExchange
            | StandardBuiltinId::AtomicsExchange
            | StandardBuiltinId::AtomicsLoad
            | StandardBuiltinId::AtomicsNotify
            | StandardBuiltinId::AtomicsOr
            | StandardBuiltinId::AtomicsPause
            | StandardBuiltinId::AtomicsSub
            | StandardBuiltinId::AtomicsStore
            | StandardBuiltinId::AtomicsWait
            | StandardBuiltinId::AtomicsWaitAsync
            | StandardBuiltinId::AtomicsXor
            | StandardBuiltinId::AtomicsIsLockFree
            | StandardBuiltinId::EvalFunction
            | StandardBuiltinId::StringFromCharCode
            | StandardBuiltinId::StringFromCodePoint
            | StandardBuiltinId::StringRaw
            | StandardBuiltinId::GeneratorPrototypeNext
            | StandardBuiltinId::GeneratorPrototypeReturn
            | StandardBuiltinId::GeneratorPrototypeThrow
            | StandardBuiltinId::AsyncGeneratorPrototypeNext
            | StandardBuiltinId::AsyncGeneratorPrototypeReturn
            | StandardBuiltinId::AsyncGeneratorPrototypeThrow
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected
            | StandardBuiltinId::ThrowTypeError
            | StandardBuiltinId::Escape
            | StandardBuiltinId::Unescape
            | StandardBuiltinId::EncodeUri
            | StandardBuiltinId::EncodeUriComponent
            | StandardBuiltinId::DecodeUri
            | StandardBuiltinId::DecodeUriComponent
            | StandardBuiltinId::SymbolFor
            | StandardBuiltinId::SymbolKeyFor
            | StandardBuiltinId::MapGroupBy
            | StandardBuiltinId::ObjectGroupBy
            | StandardBuiltinId::ObjectFromEntries
            | StandardBuiltinId::ObjectPrototypeProtoGetter
            | StandardBuiltinId::ObjectPrototypeProtoSetter
            | StandardBuiltinId::SymbolPrototypeDescriptionGetter
            | StandardBuiltinId::SymbolPrototypeToString
            | StandardBuiltinId::SymbolPrototypeValueOf
            | StandardBuiltinId::SymbolPrototypeToPrimitive
            | StandardBuiltinId::DatePrototypeToPrimitive
            | StandardBuiltinId::IntlGetCanonicalLocales
            | StandardBuiltinId::IntlLocalePrototypeLanguageGetter
            | StandardBuiltinId::IntlLocalePrototypeScriptGetter
            | StandardBuiltinId::IntlLocalePrototypeRegionGetter
            | StandardBuiltinId::IntlLocalePrototypeBaseNameGetter
            | StandardBuiltinId::IntlLocalePrototypeToString
            | StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf
            | StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts
            | StandardBuiltinId::IntlDateTimeFormatBoundFormat
            | StandardBuiltinId::TemporalPlainDateFrom
            | StandardBuiltinId::TemporalPlainDateCompare
            | StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeWith
            | StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar
            | StandardBuiltinId::TemporalPlainDatePrototypeEquals
            | StandardBuiltinId::TemporalPlainDatePrototypeToString
            | StandardBuiltinId::TemporalPlainDatePrototypeToJson
            | StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainDatePrototypeValueOf
            | StandardBuiltinId::TemporalPlainDatePrototypeAdd
            | StandardBuiltinId::TemporalPlainDatePrototypeSubtract
            | StandardBuiltinId::TemporalPlainDatePrototypeUntil
            | StandardBuiltinId::TemporalPlainDatePrototypeSince
            | StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime
            | StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth
            | StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay
            | StandardBuiltinId::TemporalPlainYearMonthFrom
            | StandardBuiltinId::TemporalPlainYearMonthCompare
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeWith
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeSince
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToString
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate
            | StandardBuiltinId::TemporalPlainMonthDayFrom
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeWith
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate
            | StandardBuiltinId::TemporalDurationFrom
            | StandardBuiltinId::TemporalDurationCompare
            | StandardBuiltinId::TemporalDurationPrototypeYearsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMonthsGetter
            | StandardBuiltinId::TemporalDurationPrototypeWeeksGetter
            | StandardBuiltinId::TemporalDurationPrototypeDaysGetter
            | StandardBuiltinId::TemporalDurationPrototypeHoursGetter
            | StandardBuiltinId::TemporalDurationPrototypeMinutesGetter
            | StandardBuiltinId::TemporalDurationPrototypeSecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeSignGetter
            | StandardBuiltinId::TemporalDurationPrototypeBlankGetter
            | StandardBuiltinId::TemporalDurationPrototypeWith
            | StandardBuiltinId::TemporalDurationPrototypeNegated
            | StandardBuiltinId::TemporalDurationPrototypeAbs
            | StandardBuiltinId::TemporalDurationPrototypeAdd
            | StandardBuiltinId::TemporalDurationPrototypeSubtract
            | StandardBuiltinId::TemporalDurationPrototypeRound
            | StandardBuiltinId::TemporalDurationPrototypeTotal
            | StandardBuiltinId::TemporalDurationPrototypeToString
            | StandardBuiltinId::TemporalDurationPrototypeToJson
            | StandardBuiltinId::TemporalDurationPrototypeToLocaleString
            | StandardBuiltinId::TemporalDurationPrototypeValueOf
            | StandardBuiltinId::TemporalPlainTimeFrom
            | StandardBuiltinId::TemporalPlainTimeCompare
            | StandardBuiltinId::TemporalPlainTimePrototypeHourGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeWith
            | StandardBuiltinId::TemporalPlainTimePrototypeAdd
            | StandardBuiltinId::TemporalPlainTimePrototypeSubtract
            | StandardBuiltinId::TemporalPlainTimePrototypeUntil
            | StandardBuiltinId::TemporalPlainTimePrototypeSince
            | StandardBuiltinId::TemporalPlainTimePrototypeRound
            | StandardBuiltinId::TemporalPlainTimePrototypeEquals
            | StandardBuiltinId::TemporalPlainTimePrototypeToString
            | StandardBuiltinId::TemporalPlainTimePrototypeToJson
            | StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainTimePrototypeValueOf
            | StandardBuiltinId::TemporalPlainDateTimeFrom
            | StandardBuiltinId::TemporalPlainDateTimeCompare
            | StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWith
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar
            | StandardBuiltinId::TemporalPlainDateTimePrototypeAdd
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract
            | StandardBuiltinId::TemporalPlainDateTimePrototypeUntil
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSince
            | StandardBuiltinId::TemporalPlainDateTimePrototypeRound
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEquals
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToString
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToJson
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime
            | StandardBuiltinId::TemporalNowInstant
            | StandardBuiltinId::TemporalNowTimeZoneId
            | StandardBuiltinId::TemporalNowZonedDateTimeIso
            | StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter
            | StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter
            | StandardBuiltinId::TemporalInstantPrototypeEquals
            | StandardBuiltinId::TemporalInstantFrom
            | StandardBuiltinId::TemporalInstantCompare
            | StandardBuiltinId::TemporalInstantFromEpochMilliseconds
            | StandardBuiltinId::TemporalInstantFromEpochNanoseconds
            | StandardBuiltinId::TemporalInstantPrototypeToString
            | StandardBuiltinId::TemporalInstantPrototypeToJson
            | StandardBuiltinId::TemporalInstantPrototypeValueOf
            | StandardBuiltinId::TemporalZonedDateTimeFrom
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEquals
            | StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant
            | StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime
            | StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone
            | StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar
            | StandardBuiltinId::TemporalZonedDateTimePrototypeAdd
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract
            | StandardBuiltinId::TemporalZonedDateTimePrototypeUntil
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSince => {}
        }

        self.release_temp_local(prototype_object_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_throw_type_error_intrinsic(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
                )
            })?;
        let function_prototype_local = self.reserve_temp_local();
        let thrower_payload_local = self.reserve_temp_local();
        let thrower_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.emit_function_value_payload(thrower_meta, function)?;
        function.instruction(&Instruction::LocalSet(thrower_payload_local));
        function.instruction(&Instruction::LocalGet(thrower_payload_local));
        function.instruction(&Instruction::GlobalSet(THROW_TYPE_ERROR_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            THROW_TYPE_ERROR_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_THROW_TYPE_ERROR_OFFSET,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(thrower_tag_local));

        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        for name in ["arguments", "caller"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_append_accessor_property_with_flags(
                function_prototype_local,
                key_local,
                Some((thrower_payload_local, thrower_tag_local)),
                Some((thrower_payload_local, thrower_tag_local)),
                false,
                true,
                function,
            )?;
        }

        self.release_temp_local(key_local);
        self.release_temp_local(thrower_tag_local);
        self.release_temp_local(thrower_payload_local);
        self.release_temp_local(function_prototype_local);
        Ok(())
    }

    pub(crate) fn init_reflect_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let construct_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectConstruct.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.construct`",
                )
            })?;
        let apply_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectApply.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.apply`",
                )
            })?;
        let get_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.get`",
                )
            })?;
        let get_prototype_of_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetPrototypeOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getPrototypeOf`",
                )
            })?;
        let get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                )
            })?;
        let set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;
        let has_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectHas.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.has`",
                )
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                )
            })?;
        let delete_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDeleteProperty.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.deleteProperty`",
                )
            })?;
        let is_extensible_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectIsExtensible.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.isExtensible`",
                )
            })?;
        let prevent_extensions_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectPreventExtensions.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.preventExtensions`",
                )
            })?;
        let set_prototype_of_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSetPrototypeOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.setPrototypeOf`",
                )
            })?;
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.ownKeys`",
                )
            })?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_function_data(object_local, "construct", construct_meta, function)?;
        self.emit_object_define_function_data(object_local, "apply", apply_meta, function)?;
        self.emit_object_define_function_data(object_local, "get", get_meta, function)?;
        self.emit_object_define_function_data(
            object_local,
            "getPrototypeOf",
            get_prototype_of_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "getOwnPropertyDescriptor",
            get_own_property_descriptor_meta,
            function,
        )?;
        self.emit_object_define_function_data(object_local, "set", set_meta, function)?;
        self.emit_object_define_function_data(object_local, "has", has_meta, function)?;
        self.emit_object_define_function_data(
            object_local,
            "defineProperty",
            define_property_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "deleteProperty",
            delete_property_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "isExtensible",
            is_extensible_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "preventExtensions",
            prevent_extensions_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "setPrototypeOf",
            set_prototype_of_meta,
            function,
        )?;
        self.emit_object_define_function_data(object_local, "ownKeys", own_keys_meta, function)?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Reflect")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(REFLECT_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    /// Temporal proposal 2.2: the `Temporal.Now` namespace object. It is an
    /// ordinary object, not a constructor, so it gets no prototype global and
    /// no branded record — only `Object.prototype`, `Symbol.toStringTag` and
    /// the clock functions this backend can actually answer.
    ///
    /// `plainDateISO`, `plainDateTimeISO` and `plainTimeISO` are deliberately
    /// absent: `Temporal.PlainDate`, `Temporal.PlainDateTime` and
    /// `Temporal.PlainTime` do not exist yet, so those functions would have no
    /// honest value to return.
    fn init_temporal_now_object(
        &mut self,
        object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Temporal.Now")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
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

        for (name, builtin) in [
            ("timeZoneId", StandardBuiltinId::TemporalNowTimeZoneId),
            ("instant", StandardBuiltinId::TemporalNowInstant),
            (
                "zonedDateTimeISO",
                StandardBuiltinId::TemporalNowZonedDateTimeIso,
            ),
        ] {
            if !self
                .runtime_bootstrap_plan
                .should_initialize_standard_builtin(builtin)
            {
                continue;
            }
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        Ok(())
    }

    pub(crate) fn init_temporal_object(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        if [
            StandardBuiltinId::TemporalNowTimeZoneId,
            StandardBuiltinId::TemporalNowInstant,
            StandardBuiltinId::TemporalNowZonedDateTimeIso,
        ]
        .into_iter()
        .any(|builtin| {
            self.runtime_bootstrap_plan
                .should_initialize_standard_builtin(builtin)
        }) {
            let now_local = self.reserve_temp_local();
            let now_tag_local = self.reserve_temp_local();
            self.init_temporal_now_object(now_local, function)?;
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(now_tag_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_NOW_NAME,
                now_local,
                now_tag_local,
                true,
                false,
                true,
                function,
            )?;
            self.release_temp_local(now_tag_local);
            self.release_temp_local(now_local);
        }
        function.instruction(&Instruction::GlobalGet(
            TEMPORAL_INSTANT_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_append_local_data_property_with_flags(
            object_local,
            "Instant",
            constructor_local,
            constructor_tag_local,
            true,
            false,
            true,
            function,
        )?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainDateConstructor)
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_PLAIN_DATE_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_PLAIN_DATE_NAME,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor)
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_ZONED_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                "ZonedDateTime",
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainTimeConstructor)
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_PLAIN_TIME_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_PLAIN_TIME_NAME,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainDateTimeConstructor)
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_PLAIN_DATE_TIME_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_PLAIN_DATE_TIME_NAME,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(
                StandardBuiltinId::TemporalPlainYearMonthConstructor,
            )
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_PLAIN_YEAR_MONTH_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_PLAIN_YEAR_MONTH_NAME,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainMonthDayConstructor)
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_PLAIN_MONTH_DAY_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_PLAIN_MONTH_DAY_NAME,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalDurationConstructor)
        {
            function.instruction(&Instruction::GlobalGet(
                TEMPORAL_DURATION_CONSTRUCTOR_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                TEMPORAL_DURATION_NAME,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Temporal")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(TEMPORAL_OBJECT_GLOBAL_INDEX));

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    /// ECMA-402 8: the `Intl` namespace object. Only the properties this
    /// backend actually implements are installed — nothing is stubbed.
    ///
    /// `members` is a proof, obtainable only from
    /// `RuntimeBootstrapPlan::intl_namespace_members`, that every member is
    /// rooted. Holding it is what lets this function install the whole list
    /// unconditionally; it is also the reason the function cannot be called for
    /// a program that does not get an `Intl` object at all.
    pub(crate) fn init_intl_object(
        &mut self,
        members: IntlNamespaceMembers,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        let get_canonical_locales_meta = self
            .functions
            .get(&StandardBuiltinId::IntlGetCanonicalLocales.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Intl.getCanonicalLocales`",
                )
            })?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_function_data(
            object_local,
            "getCanonicalLocales",
            get_canonical_locales_meta,
            function,
        )?;
        // One list, `INTL_NAMESPACE_CONSTRUCTORS`, decides both what the IR
        // shape claims `Intl` has (`ScriptLowerer::intl_object_value_info`) and
        // what actually gets installed here. They used to be two
        // hand-maintained lists and they drifted: `DateTimeFormat` was declared
        // and never installed, so constant-folded member access hid the gap —
        // `new Intl.DateTimeFormat()` worked while
        // `Object.getOwnPropertyDescriptor(Intl, "DateTimeFormat")`,
        // `Object.keys(Intl)`, `Intl["DateTimeFormat"]` and destructuring all
        // saw nothing. That is `intl402/DateTimeFormat/prop-desc.js`'s
        // "Expected descriptor to exist".
        //
        // Unifying the lists closed the drift but left a second divergence
        // point right here: a per-member `should_initialize_standard_builtin`
        // check with a `continue`, which reintroduced exactly the same wrong
        // object whenever the plan under-rooted the namespace. That check is
        // gone. `members` is the proof that it would have been vacuous, and it
        // is the only way to reach the list at all, so a partially installed
        // `Intl` is now unrepresentable rather than untested.
        //
        // Installation order is `Object.getOwnPropertyNames(Intl)` order, so it
        // is the slice's order and must not be sorted here.
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        for (name, builtin) in members.in_installation_order() {
            let global_index =
                standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: \
                         missing Intl constructor global `{}`",
                        builtin.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::GlobalGet(global_index));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                name,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Intl")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(INTL_OBJECT_GLOBAL_INDEX));

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_math_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        for (name, value) in [
            ("E", std::f64::consts::E),
            ("LN10", std::f64::consts::LN_10),
            ("LN2", std::f64::consts::LN_2),
            ("LOG10E", std::f64::consts::LOG10_E),
            ("LOG2E", std::f64::consts::LOG2_E),
            ("PI", std::f64::consts::PI),
            ("SQRT1_2", std::f64::consts::FRAC_1_SQRT_2),
            ("SQRT2", std::f64::consts::SQRT_2),
        ] {
            self.emit_object_define_number_data_from_f64_const_with_flags(
                object_local,
                name,
                value,
                false,
                false,
                false,
                function,
            )?;
        }
        for (name, builtin) in [
            ("abs", StandardBuiltinId::MathAbs),
            ("acos", StandardBuiltinId::MathAcos),
            ("acosh", StandardBuiltinId::MathAcosh),
            ("asin", StandardBuiltinId::MathAsin),
            ("asinh", StandardBuiltinId::MathAsinh),
            ("atan", StandardBuiltinId::MathAtan),
            ("atan2", StandardBuiltinId::MathAtan2),
            ("atanh", StandardBuiltinId::MathAtanh),
            ("cbrt", StandardBuiltinId::MathCbrt),
            ("ceil", StandardBuiltinId::MathCeil),
            ("clz32", StandardBuiltinId::MathClz32),
            ("cos", StandardBuiltinId::MathCos),
            ("cosh", StandardBuiltinId::MathCosh),
            ("exp", StandardBuiltinId::MathExp),
            ("expm1", StandardBuiltinId::MathExpm1),
            ("f16round", StandardBuiltinId::MathF16Round),
            ("floor", StandardBuiltinId::MathFloor),
            ("fround", StandardBuiltinId::MathFround),
            ("hypot", StandardBuiltinId::MathHypot),
            ("imul", StandardBuiltinId::MathImul),
            ("log", StandardBuiltinId::MathLog),
            ("log10", StandardBuiltinId::MathLog10),
            ("log1p", StandardBuiltinId::MathLog1p),
            ("log2", StandardBuiltinId::MathLog2),
            ("pow", StandardBuiltinId::MathPow),
            ("random", StandardBuiltinId::MathRandom),
            ("round", StandardBuiltinId::MathRound),
            ("sign", StandardBuiltinId::MathSign),
            ("sin", StandardBuiltinId::MathSin),
            ("sinh", StandardBuiltinId::MathSinh),
            ("sqrt", StandardBuiltinId::MathSqrt),
            ("sumPrecise", StandardBuiltinId::MathSumPrecise),
            ("tan", StandardBuiltinId::MathTan),
            ("tanh", StandardBuiltinId::MathTanh),
            ("trunc", StandardBuiltinId::MathTrunc),
            ("min", StandardBuiltinId::MathMin),
            ("max", StandardBuiltinId::MathMax),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Math")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
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
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(MATH_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_json_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(JSON_NAME)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
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
        for (name, builtin) in [
            ("parse", StandardBuiltinId::JsonParse),
            ("stringify", StandardBuiltinId::JsonStringify),
            ("rawJSON", StandardBuiltinId::JsonRawJson),
            ("isRawJSON", StandardBuiltinId::JsonIsRawJson),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(JSON_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_atomics_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(ATOMICS_NAME)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
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

        for (name, builtin) in [
            ("add", StandardBuiltinId::AtomicsAdd),
            ("and", StandardBuiltinId::AtomicsAnd),
            ("compareExchange", StandardBuiltinId::AtomicsCompareExchange),
            ("exchange", StandardBuiltinId::AtomicsExchange),
            ("load", StandardBuiltinId::AtomicsLoad),
            ("notify", StandardBuiltinId::AtomicsNotify),
            ("or", StandardBuiltinId::AtomicsOr),
            ("pause", StandardBuiltinId::AtomicsPause),
            ("store", StandardBuiltinId::AtomicsStore),
            ("sub", StandardBuiltinId::AtomicsSub),
            ("wait", StandardBuiltinId::AtomicsWait),
            ("waitAsync", StandardBuiltinId::AtomicsWaitAsync),
            ("xor", StandardBuiltinId::AtomicsXor),
            ("isLockFree", StandardBuiltinId::AtomicsIsLockFree),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(ATOMICS_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_typed_array_intrinsic(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let typed_array_constructor_local = self.reserve_temp_local();
        let typed_array_prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        let function_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionConstructor.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function`",
                )
            })?;
        self.emit_function_value_payload(function_meta, function)?;
        function.instruction(&Instruction::LocalSet(typed_array_constructor_local));
        function.instruction(&Instruction::LocalGet(typed_array_constructor_local));
        function.instruction(&Instruction::GlobalSet(
            TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(typed_array_prototype_local));
        self.store_i64_local_at_offset(
            typed_array_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            typed_array_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            typed_array_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_object_define_data(
            typed_array_constructor_local,
            key_local,
            typed_array_prototype_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(typed_array_constructor_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            typed_array_prototype_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;

        let species_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArraySpeciesGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(species_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            typed_array_constructor_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;

        for (name, builtin) in [
            ("buffer", StandardBuiltinId::TypedArrayPrototypeBufferGetter),
            (
                "byteLength",
                StandardBuiltinId::TypedArrayPrototypeByteLengthGetter,
            ),
            (
                "byteOffset",
                StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter,
            ),
            ("length", StandardBuiltinId::TypedArrayPrototypeLengthGetter),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(&name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(typed_array_prototype_local));
            self.emit_object_append_accessor_property_with_flags(
                typed_array_prototype_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }

        let to_string_tag_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToStringTagGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get TypedArray.prototype[Symbol.toStringTag]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(to_string_tag_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            typed_array_prototype_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;

        let at_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeAt.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.at`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "at",
            at_meta,
            function,
        )?;

        for (name, builtin) in [
            ("includes", StandardBuiltinId::TypedArrayPrototypeIncludes),
            ("indexOf", StandardBuiltinId::TypedArrayPrototypeIndexOf),
            (
                "lastIndexOf",
                StandardBuiltinId::TypedArrayPrototypeLastIndexOf,
            ),
            ("find", StandardBuiltinId::TypedArrayPrototypeFind),
            ("findIndex", StandardBuiltinId::TypedArrayPrototypeFindIndex),
            ("findLast", StandardBuiltinId::TypedArrayPrototypeFindLast),
            (
                "findLastIndex",
                StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
            ),
            ("every", StandardBuiltinId::TypedArrayPrototypeEvery),
            ("some", StandardBuiltinId::TypedArrayPrototypeSome),
            ("map", StandardBuiltinId::TypedArrayPrototypeMap),
            ("filter", StandardBuiltinId::TypedArrayPrototypeFilter),
            ("forEach", StandardBuiltinId::TypedArrayPrototypeForEach),
            ("reduce", StandardBuiltinId::TypedArrayPrototypeReduce),
            (
                "reduceRight",
                StandardBuiltinId::TypedArrayPrototypeReduceRight,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(
                typed_array_prototype_local,
                name,
                meta,
                function,
            )?;
        }

        let values_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeValues.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.values`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            typed_array_prototype_local,
            "values",
            &["Symbol.iterator"],
            values_meta,
            function,
        )?;
        for (name, builtin) in [
            ("keys", StandardBuiltinId::TypedArrayPrototypeKeys),
            ("entries", StandardBuiltinId::TypedArrayPrototypeEntries),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(
                typed_array_prototype_local,
                name,
                meta,
                function,
            )?;
        }

        let fill_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFill.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.fill`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "fill",
            fill_meta,
            function,
        )?;

        let join_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeJoin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.join`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "join",
            join_meta,
            function,
        )?;

        let subarray_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSubarray.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.subarray`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "subarray",
            subarray_meta,
            function,
        )?;

        let slice_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSlice.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.slice`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "slice",
            slice_meta,
            function,
        )?;

        let set_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.set`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "set",
            set_meta,
            function,
        )?;

        let reverse_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeReverse.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.reverse`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "reverse",
            reverse_meta,
            function,
        )?;

        let copy_within_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeCopyWithin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.copyWithin`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "copyWithin",
            copy_within_meta,
            function,
        )?;

        let sort_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSort.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.sort`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "sort",
            sort_meta,
            function,
        )?;

        let to_reversed_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToReversed.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.toReversed`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toReversed",
            to_reversed_meta,
            function,
        )?;

        let to_sorted_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToSorted.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.toSorted`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toSorted",
            to_sorted_meta,
            function,
        )?;

        let with_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeWith.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.with`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "with",
            with_meta,
            function,
        )?;

        self.emit_object_define_function_global_data(
            typed_array_prototype_local,
            "toString",
            ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
            function,
        )?;
        let to_locale_string_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToLocaleString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.toLocaleString`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toLocaleString",
            to_locale_string_meta,
            function,
        )?;
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.from`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_constructor_local,
            "from",
            from_meta,
            function,
        )?;
        let of_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.of`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_constructor_local,
            "of",
            of_meta,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(typed_array_prototype_local);
        self.release_temp_local(typed_array_constructor_local);
        Ok(())
    }

    pub(crate) fn repair_typed_array_constructor_graph(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_global_index =
            standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin constructor global `{}`",
                    builtin.debug_name()
                ))
            })?;
        let constructor_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::GlobalGet(
            TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            constructor_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        self.load_i64_to_local_from_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data_with_configurable(
            constructor_local,
            key_local,
            prototype_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(
            prototype_local,
            key_local,
            constructor_local,
            tag_local,
            function,
        )?;
        let realm_intrinsic_offset = typed_array_realm_intrinsics_prototype_offset(builtin)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing realm intrinsic prototype offset `{}`",
                    builtin.debug_name()
                ))
            })?;
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_intrinsic_prototype(
            self.scratch_local,
            realm_intrinsic_offset,
            prototype_local,
            function,
        );
        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn repair_error_constructor_graph(
        &mut self,
        constructor_global_index: u32,
        prototype_global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::GlobalGet(ERROR_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            prototype_local,
            key_local,
            constructor_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn repair_native_error_constructor_graphs(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (constructor_global_index, prototype_global_index) in [
            (
                EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
        ] {
            self.repair_error_constructor_graph(
                constructor_global_index,
                prototype_global_index,
                function,
            )?;
        }
        Ok(())
    }

    pub(crate) fn init_array_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorNext.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array Iterator.prototype.next`",
                )
            })?;
        let iterator_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array Iterator.prototype[Symbol.iterator]`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", next_meta, function)?;
        self.emit_object_define_function_data(
            prototype_local,
            "Symbol.iterator",
            iterator_meta,
            function,
        )?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Array Iterator"),
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

    pub(crate) fn init_string_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::StringIteratorNext.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `String Iterator.prototype.next`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", next_meta, function)?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("String Iterator"),
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

    pub(crate) fn init_map_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::MapIteratorNext.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Map Iterator.prototype.next`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", &next_meta, function)?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Map Iterator")));
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

    pub(crate) fn init_set_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::SetIteratorNext.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Set Iterator.prototype.next`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", &next_meta, function)?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Set Iterator")));
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

    pub(crate) fn init_generator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(GENERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        let constructor_key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            constructor_key_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(constructor_key_local);
        for (name, builtin) in [
            ("next", StandardBuiltinId::GeneratorPrototypeNext),
            ("return", StandardBuiltinId::GeneratorPrototypeReturn),
            ("throw", StandardBuiltinId::GeneratorPrototypeThrow),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_object_define_function_data(prototype_local, name, &meta, function)?;
        }
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Generator")));
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

    pub(crate) fn init_generator_function_intrinsics(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
                )
            })?;
        let mut constructor_meta = thrower_meta;
        constructor_meta.name = "GeneratorFunction".to_string();
        constructor_meta.to_string_value =
            "function GeneratorFunction() { [native code] }".to_string();
        constructor_meta.length = 1;
        constructor_meta.length_name_configurable = true;
        constructor_meta.constructable = false;

        self.emit_function_value_payload(&constructor_meta, function)?;
        let constructor_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(constructor_local));
        self.store_i64_const_at_offset(
            constructor_local,
            HEAP_FUNCTION_FLAGS_OFFSET,
            FUNCTION_FLAG_CONSTRUCTABLE,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );

        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            constructor_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_local));
        function.instruction(&Instruction::GlobalSet(
            GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET,
            function,
        );

        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        for (name, value_global_index, value_kind) in [
            (
                "prototype",
                GENERATOR_PROTOTYPE_GLOBAL_INDEX,
                ValueKind::Object,
            ),
            (
                "constructor",
                GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                ValueKind::Function,
            ),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(value_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
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
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("GeneratorFunction"),
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
        self.release_temp_local(prototype_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn init_async_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        let mut identity_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing iterator identity builtin meta",
                )
            })?;
        identity_meta.name = "[Symbol.asyncIterator]".to_string();
        identity_meta.to_string_value =
            "function [Symbol.asyncIterator]() { [native code] }".to_string();
        self.emit_object_define_function_data(
            prototype_local,
            "Symbol.asyncIterator",
            &identity_meta,
            function,
        )?;
        let async_dispose_meta = self
            .functions
            .get(
                &StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
                    .function_id(),
            )
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing AsyncIterator asyncDispose builtin meta",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_local,
            "Symbol.asyncDispose",
            &async_dispose_meta,
            function,
        )?;
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_async_generator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));

        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(
            ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
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

        for (name, builtin) in [
            ("next", StandardBuiltinId::AsyncGeneratorPrototypeNext),
            ("return", StandardBuiltinId::AsyncGeneratorPrototypeReturn),
            ("throw", StandardBuiltinId::AsyncGeneratorPrototypeThrow),
        ] {
            let method_meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_object_define_function_data(prototype_local, name, &method_meta, function)?;
        }

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("AsyncGenerator"),
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

    pub(crate) fn init_async_function_intrinsics(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
                )
            })?;

        for (name, source, prototype_global_index, constructor_global_index, intrinsic_offset) in [
            (
                "AsyncFunction",
                "function AsyncFunction() { [native code] }",
                ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_CONSTRUCTOR_OFFSET,
            ),
            (
                "AsyncGeneratorFunction",
                "function AsyncGeneratorFunction() { [native code] }",
                ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_OFFSET,
            ),
        ] {
            let mut constructor_meta = thrower_meta.clone();
            constructor_meta.name = name.to_string();
            constructor_meta.to_string_value = source.to_string();
            constructor_meta.length = 1;
            constructor_meta.length_name_configurable = true;
            constructor_meta.constructable = false;

            self.emit_function_value_payload(&constructor_meta, function)?;
            let constructor_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.store_i64_const_at_offset(
                constructor_local,
                HEAP_FUNCTION_FLAGS_OFFSET,
                FUNCTION_FLAG_CONSTRUCTABLE,
                function,
            );
            function.instruction(&Instruction::GlobalGet(FUNCTION_CONSTRUCTOR_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(
                constructor_local,
                HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                ValueKind::Function.tag() as u64,
                function,
            );

            let key_local = self.reserve_temp_local();
            let payload_local = self.reserve_temp_local();
            let tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                constructor_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                false,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(constructor_local));
            function.instruction(&Instruction::GlobalSet(constructor_global_index));
            self.emit_store_current_realm_global_intrinsic(
                constructor_global_index,
                intrinsic_offset,
                function,
            );
            self.release_temp_local(tag_local);
            self.release_temp_local(payload_local);
            self.release_temp_local(key_local);
            self.release_temp_local(constructor_local);
        }

        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        for (
            prototype_global_index,
            constructor_global_index,
            instance_prototype_global_index,
            to_string_tag,
        ) in [
            (
                ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                None,
                "AsyncFunction",
            ),
            (
                ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                Some(ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX),
                "AsyncGeneratorFunction",
            ),
        ] {
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(prototype_local));
            if let Some(instance_prototype_global_index) = instance_prototype_global_index {
                function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(instance_prototype_global_index));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
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
            }

            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(constructor_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
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

            function.instruction(&Instruction::I64Const(
                self.strings
                    .property_key_symbol_payload("Symbol.toStringTag"),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(to_string_tag)));
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
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_runtime_roots(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::GlobalSet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        let object_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        self.store_i64_const_at_offset(
            object_prototype_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_IMMUTABLE_PROTOTYPE,
            function,
        );
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_object_prototype(
            self.scratch_local,
            object_prototype_local,
            function,
        );
        self.release_temp_local(object_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        let function_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        self.emit_object_define_number_data_from_f64_const_with_flags(
            function_prototype_local,
            "length",
            0.0,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(function_prototype_local);
        // Array.prototype is itself an Array exotic object.  Allocate it with
        // the array layout, then repair the allocator's default prototype
        // (which would otherwise point back at the not-yet-initialized global).
        let array_prototype_length_local = self.reserve_temp_local();
        let array_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_prototype_length_local));
        self.emit_alloc_array_payload_with_length(
            array_prototype_length_local,
            array_prototype_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_prototype_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_prototype_local));
        function.instruction(&Instruction::GlobalSet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        self.release_temp_local(array_prototype_local);
        self.release_temp_local(array_prototype_length_local);
        self.emit_store_current_realm_global_intrinsic(
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ITERATOR_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ITERATOR_HELPER_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ITERATOR_FROM_WRAPPER_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_MAP_ITERATOR_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_SET_ITERATOR_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(GENERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            GENERATOR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_GENERATOR_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(FUNCTION_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ASYNC_ITERATOR_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(FUNCTION_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(STRING_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(PROMISE_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(MAP_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            MAP_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_MAP_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(WEAK_MAP_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(WEAK_REF_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            WEAK_REF_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(WEAK_SET_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET,
            function,
        );
        // `%AsyncDisposableStack.prototype%` deliberately gets no
        // `HEAP_REALM_INTRINSICS_*` slot. The only case that could observe one
        // is `proto-from-ctor-realm.js`, which is a policy case
        // (`Function constructor dynamic code generation`) and cannot pass on
        // this backend; the constructor therefore falls back to the current
        // realm's global (`NewTargetPrototypeFallback::CurrentGlobal`) and the
        // 344-byte realm-intrinsics record does not move.
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SET_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            SET_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_SET_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            SYMBOL_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_SYMBOL_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            TYPE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_type_error_prototype(
            self.scratch_local,
            native_error_prototype_local,
            function,
        );
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            REFERENCE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            EVAL_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            AGGREGATE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            SUPPRESSED_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            RANGE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            SYNTAX_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(URI_ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(URI_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            URI_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            SHARED_ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(DATA_VIEW_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(DATE_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        let regexp_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(regexp_prototype_local));
        self.store_i64_const_at_offset(
            regexp_prototype_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            self.strings.payload("(?:)") as u64,
            function,
        );
        self.store_i64_const_at_offset(
            regexp_prototype_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            self.strings.payload("") as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(regexp_prototype_local));
        function.instruction(&Instruction::GlobalSet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_intrinsic_prototype(
            self.scratch_local,
            HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET,
            regexp_prototype_local,
            function,
        );
        self.release_temp_local(regexp_prototype_local);
        // These per-realm function-value globals cache the `@@match`/`@@matchAll`/
        // `@@search` methods and the shared Array/TypedArray `toString`. Their only
        // readers are inside constructor-init and builtin bodies that are
        // themselves gated on (or force-compiled from) the same planned kind: the
        // RegExp `@@` slots are read by `init_builtin_constructor_object(RegExp)`
        // and by the String regexp-protocol method bodies (which force RegExp), and
        // the shared `toString` slot is read by the Array / TypedArray prototype
        // setup. When the guarding constructor cannot exist in this module, the
        // slot is never read, so materializing it here would only force a
        // dead builtin body through the emission fixpoint. Skip it (shape-guarded
        // recording — see `FunctionMetaRegistry`).
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::RegExpConstructor)
        {
            let regexp_match_meta = self
                .functions
                .get(&StandardBuiltinId::RegExpPrototypeSymbolMatch.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.match]`",
                    )
                })?;
            self.emit_function_value_payload(&regexp_match_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
            ));
            let regexp_match_all_meta = self
                .functions
                .get(&StandardBuiltinId::RegExpPrototypeSymbolMatchAll.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.matchAll]`",
                    )
                })?;
            self.emit_function_value_payload(&regexp_match_all_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_GLOBAL_INDEX,
            ));
            let regexp_search_meta = self
                .functions
                .get(&StandardBuiltinId::RegExpPrototypeSymbolSearch.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.search]`",
                    )
                })?;
            self.emit_function_value_payload(&regexp_search_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX,
            ));
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayConstructor)
            || self.runtime_bootstrap_plan.needs_typed_array_intrinsic()
        {
            let array_typed_array_to_string_meta = self
                .functions
                .get(&StandardBuiltinId::TypedArrayPrototypeToString.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.toString`",
                    )
                })?;
            self.emit_function_value_payload(&array_typed_array_to_string_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
            ));
        }
        self.init_builtin_constructor_object(
            StandardBuiltinId::FunctionConstructor,
            FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_throw_type_error_intrinsic(function)?;
        self.init_generator_function_intrinsics(function)?;
        self.init_async_function_intrinsics(function)?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ObjectConstructor,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::ProxyConstructor,
                OBJECT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::IteratorConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::IteratorConstructor,
                ITERATOR_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::ArrayConstructor,
                ARRAY_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        self.init_array_iterator_prototype(function)?;
        self.init_string_iterator_prototype(function)?;
        self.init_map_iterator_prototype(function)?;
        self.init_set_iterator_prototype(function)?;
        self.init_generator_prototype(function)?;
        self.init_async_iterator_prototype(function)?;
        self.init_async_generator_prototype(function)?;
        let array_iterator_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(array_iterator_prototype_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_array_iterator_prototype(
            self.scratch_local,
            array_iterator_prototype_local,
            function,
        );
        self.release_temp_local(array_iterator_prototype_local);
        let string_iterator_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(string_iterator_prototype_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_string_iterator_prototype(
            self.scratch_local,
            string_iterator_prototype_local,
            function,
        );
        self.release_temp_local(string_iterator_prototype_local);
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayBufferConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::ArrayBufferConstructor,
                ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::SharedArrayBufferConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::SharedArrayBufferConstructor,
                SHARED_ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::DataViewConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::DataViewConstructor,
                DATA_VIEW_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::DateConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::DateConstructor,
                DATE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalInstantConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalInstantConstructor,
                TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainDateConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainDateConstructor,
                TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalDurationConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalDurationConstructor,
                TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainTimeConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainTimeConstructor,
                TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainDateTimeConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainDateTimeConstructor,
                TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(
                StandardBuiltinId::TemporalPlainYearMonthConstructor,
            )
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainYearMonthConstructor,
                TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainMonthDayConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainMonthDayConstructor,
                TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalZonedDateTimeConstructor,
                TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::IntlLocaleConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::IntlLocaleConstructor,
                INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::IntlDateTimeFormatConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::IntlDateTimeFormatConstructor,
                INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::RegExpConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::RegExpConstructor,
                REGEXP_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self.runtime_bootstrap_plan.needs_typed_array_intrinsic() {
            self.init_typed_array_intrinsic(function)?;
        }
        for builtin in [
            StandardBuiltinId::Float64ArrayConstructor,
            StandardBuiltinId::Float32ArrayConstructor,
            StandardBuiltinId::Int32ArrayConstructor,
            StandardBuiltinId::Int16ArrayConstructor,
            StandardBuiltinId::Int8ArrayConstructor,
            StandardBuiltinId::Uint32ArrayConstructor,
            StandardBuiltinId::Uint16ArrayConstructor,
            StandardBuiltinId::Uint8ArrayConstructor,
            StandardBuiltinId::Uint8ClampedArrayConstructor,
            StandardBuiltinId::BigInt64ArrayConstructor,
            StandardBuiltinId::BigUint64ArrayConstructor,
        ] {
            if self
                .runtime_bootstrap_plan
                .should_initialize_standard_builtin(builtin)
            {
                self.init_builtin_constructor_object(
                    builtin,
                    TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX,
                    function,
                )?;
            }
        }
        for builtin in [
            StandardBuiltinId::Float64ArrayConstructor,
            StandardBuiltinId::Float32ArrayConstructor,
            StandardBuiltinId::Int32ArrayConstructor,
            StandardBuiltinId::Int16ArrayConstructor,
            StandardBuiltinId::Int8ArrayConstructor,
            StandardBuiltinId::Uint32ArrayConstructor,
            StandardBuiltinId::Uint16ArrayConstructor,
            StandardBuiltinId::Uint8ArrayConstructor,
            StandardBuiltinId::Uint8ClampedArrayConstructor,
            StandardBuiltinId::BigInt64ArrayConstructor,
            StandardBuiltinId::BigUint64ArrayConstructor,
        ] {
            if self
                .runtime_bootstrap_plan
                .should_initialize_standard_builtin(builtin)
            {
                self.repair_typed_array_constructor_graph(builtin, function)?;
            }
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::NumberConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::NumberConstructor,
                NUMBER_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::StringConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::StringConstructor,
                STRING_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
            let string_constructor_local = self.reserve_temp_local();
            let from_code_point_meta = self
                .functions
                .get(&StandardBuiltinId::StringFromCodePoint.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `String.fromCodePoint`",
                    )
                })?;
            function.instruction(&Instruction::GlobalGet(STRING_CONSTRUCTOR_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(string_constructor_local));
            let from_char_code_meta = self
                .functions
                .get(&StandardBuiltinId::StringFromCharCode.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `String.fromCharCode`",
                    )
                })?;
            self.emit_object_define_function_data(
                string_constructor_local,
                "fromCharCode",
                from_char_code_meta,
                function,
            )?;
            self.emit_object_define_function_data(
                string_constructor_local,
                "fromCodePoint",
                from_code_point_meta,
                function,
            )?;
            let raw_meta = self
                .functions
                .get(&StandardBuiltinId::StringRaw.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `String.raw`",
                    )
                })?;
            self.emit_object_define_function_data(
                string_constructor_local,
                "raw",
                raw_meta,
                function,
            )?;
            self.release_temp_local(string_constructor_local);
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::BooleanConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::BooleanConstructor,
                BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::PromiseConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::PromiseConstructor,
                PROMISE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::MapConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::MapConstructor,
                MAP_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::WeakMapConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::WeakMapConstructor,
                WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::WeakRefConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::WeakRefConstructor,
                WEAK_REF_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::FinalizationRegistryConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::FinalizationRegistryConstructor,
                FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::WeakSetConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::WeakSetConstructor,
                WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::AsyncDisposableStackConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::AsyncDisposableStackConstructor,
                ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::SetConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::SetConstructor,
                SET_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::SymbolConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::SymbolConstructor,
                SYMBOL_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::BigIntConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::BigIntConstructor,
                OBJECT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        self.init_builtin_constructor_object(
            StandardBuiltinId::ErrorConstructor,
            ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::EvalErrorConstructor,
            EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::AggregateErrorConstructor,
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::SuppressedErrorConstructor,
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::RangeErrorConstructor,
            RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::SyntaxErrorConstructor,
            SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::TypeErrorConstructor,
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::URIErrorConstructor,
            URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ReferenceErrorConstructor,
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.repair_native_error_constructor_graphs(function)?;
        let object_prototype_local = self.reserve_temp_local();
        let object_constructor_local = self.reserve_temp_local();
        let object_constructor_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_constructor_tag_local));
        self.emit_object_append_local_data_property_with_flags(
            object_prototype_local,
            "constructor",
            object_constructor_local,
            object_constructor_tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.release_temp_local(object_constructor_tag_local);
        self.release_temp_local(object_constructor_local);
        self.release_temp_local(object_prototype_local);
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.reflect_object
        {
            self.init_reflect_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.math_object
        {
            self.init_math_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.json_object
        {
            self.init_json_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.atomics_object
        {
            self.init_atomics_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.temporal_object
        {
            self.init_temporal_object(function)?;
        }
        // Unlike its five siblings above, the `Intl` gate hands back the member
        // list rather than a bool: "install `Intl`" and "every member the IR
        // shape declares is rooted" are one decision, made once in `planning`.
        if let Some(intl_namespace_members) = self.runtime_bootstrap_plan.intl_namespace_members() {
            self.init_intl_object(intl_namespace_members, function)?;
        }
        Ok(())
    }

    pub(crate) fn init_script_global_object(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }

        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let script_global_bindings = self
            .script_global_bindings
            .clone()
            .into_iter()
            .map(|(name, kind)| ScriptGlobalBindingIr { name, kind })
            .filter(|binding| {
                self.runtime_bootstrap_plan
                    .should_install_script_global_binding(binding.kind)
            })
            .collect::<Vec<_>>();
        let capacity = (script_global_bindings.len() as u64).max(MIN_HEAP_CAPACITY);

        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(capacity * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, capacity, function);
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            self.scratch_local,
            HEAP_REALM_GLOBAL_OBJECT_OFFSET,
            object_local,
            function,
        );
        self.store_i64_local_at_offset(
            self.scratch_local,
            HEAP_REALM_GLOBAL_THIS_OFFSET,
            object_local,
            function,
        );
        self.store_i64_local_at_offset(
            self.scratch_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            self.current_env_local,
            function,
        );

        for binding in script_global_bindings {
            function.instruction(&Instruction::I64Const(self.strings.payload(&binding.name)));
            function.instruction(&Instruction::LocalSet(key_local));
            match binding.kind {
                ScriptGlobalBindingKind::Intrinsic => {
                    function.instruction(&Instruction::LocalGet(object_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Infinity => {
                    function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::NaN => {
                    function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Undefined => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Var => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Function => {
                    let meta = self.functions.values().find(|meta| meta.name == binding.name).ok_or_else(
                        || {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: unknown script-global function `{}`",
                                binding.name
                            ))
                        },
                    )?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::ReflectObject => {
                    function.instruction(&Instruction::GlobalGet(REFLECT_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::MathObject => {
                    function.instruction(&Instruction::GlobalGet(MATH_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::JsonObject => {
                    function.instruction(&Instruction::GlobalGet(JSON_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::AtomicsObject => {
                    function.instruction(&Instruction::GlobalGet(ATOMICS_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::TemporalObject => {
                    function.instruction(&Instruction::GlobalGet(TEMPORAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::IntlObject => {
                    function.instruction(&Instruction::GlobalGet(INTL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::BuiltinFunction(builtin) => {
                    if let Some(global_index) = standard_builtin_constructor_global_index(builtin) {
                        function.instruction(&Instruction::GlobalGet(global_index));
                        function.instruction(&Instruction::LocalSet(payload_local));
                    } else {
                        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                        self.emit_function_value_payload(meta, function)?;
                        function.instruction(&Instruction::LocalSet(payload_local));
                    }
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::HostFunction(builtin) => {
                    let meta = self
                        .functions
                        .get(&builtin.function_id())
                        .cloned()
                        .ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: unknown script-global host function `{}`",
                                builtin.as_str()
                            ))
                        })?;
                    // `parseInt`/`parseFloat` must be the same object as
                    // `Number.parseInt`/`Number.parseFloat`; source them from the
                    // canonical per-realm global rather than a fresh allocation.
                    if let Some(global_index) =
                        canonical_host_function_global_index_by_name(binding.name.as_str())
                    {
                        self.emit_ensure_canonical_host_function(&meta, global_index, function)?;
                        function.instruction(&Instruction::GlobalGet(global_index));
                    } else {
                        self.emit_function_value_payload(&meta, function)?;
                    }
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            }
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                !matches!(
                    binding.kind,
                    ScriptGlobalBindingKind::Infinity
                        | ScriptGlobalBindingKind::NaN
                        | ScriptGlobalBindingKind::Undefined
                ),
                matches!(
                    binding.kind,
                    ScriptGlobalBindingKind::Var | ScriptGlobalBindingKind::Function
                ),
                !matches!(
                    binding.kind,
                    ScriptGlobalBindingKind::Intrinsic
                        | ScriptGlobalBindingKind::Infinity
                        | ScriptGlobalBindingKind::NaN
                        | ScriptGlobalBindingKind::Undefined
                        | ScriptGlobalBindingKind::Var
                        | ScriptGlobalBindingKind::Function
                ),
                function,
            )?;
        }

        if let Some(slot) = self.owned_env_slot(LEXICAL_THIS_NAME) {
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.write_env_slot_from_locals(slot, 0, payload_local, tag_local, function);
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }
}
