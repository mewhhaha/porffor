//! The Intl object's unexposed, per-Realm fallback-symbol identity.

use super::*;
use crate::gc_types::{
    GcLocal, GcOperand, IntrinsicTable, RealmRecord, RealmRecordSchema, StoredValue, SymbolValue,
};

// These slots are outside the closed object-only intrinsic domain. Its generic
// loader rejects Symbols, and no public property exposes this table entry.
pub(super) const PRIVATE_REALM_INTRINSIC_COUNT: u32 = 1;
struct IntlFallbackSymbolSlot;
impl IntlFallbackSymbolSlot {
    const fn index() -> u32 {
        NonArrayRealmIntrinsicSlot::TABLE_LENGTH
    }
}

impl FunctionBuilder<'_> {
    /// Laziness performs no guest operation: the description is an R constant
    /// and Symbol creation has no user hooks. Identity is retained by this
    /// actual Realm across both formatter services, programs and GC.
    pub(crate) fn emit_realm_intl_fallback_symbol(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        function: &mut Function,
    ) -> Result<GcLocal<SymbolValue>, EmitError> {
        let schema = self.runtime_schema();
        let intrinsics = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<RealmRecord>()
                .field(RealmRecordSchema::INTRINSICS)
                .read(realm, schema, function)
                .reference(),
            function,
        );
        let slot = schema.reserve_i32_local(function);
        slot.set_constant(IntlFallbackSymbolSlot::index() as i32, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<IntrinsicTable>()
                .read(&intrinsics, slot, schema, function)
                .reference(),
            function,
        );
        let value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &value, schema, function);
        stored.clear(function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let description = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("IntlLegacyConstructedSymbol", function)?,
            function,
        );
        let symbol = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<SymbolValue>().construct(
                (
                    GcOperand::nullable_reference(&description, schema),
                    GcOperand::null(schema),
                    GcOperand::i64(0),
                ),
                function,
            ),
            function,
        );
        value.set_reference(&symbol, schema, function);
        let published = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&value, function),
            function,
        );
        schema.array_type::<IntrinsicTable>().write(
            &intrinsics,
            slot,
            GcOperand::reference(&published, schema),
            schema,
            function,
        );
        published.clear(function);
        symbol.clear(function);
        description.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // Undefined is the only uninitialized encoding. A different brand is
        // an internal slot-layout error, never a guest-selected replacement.
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Symbol as i32));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let result = schema.reserve_gc_local(function).initialize(
            value.cast_reference::<SymbolValue>(schema, function),
            function,
        );
        value.clear(function);
        schema.release_i32_local(slot, function);
        intrinsics.clear(function);
        Ok(result)
    }
}
