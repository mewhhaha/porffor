use super::*;
use crate::functions::ArgumentListConstruction;
use crate::gc_types::{CodeUnitArray, StringConstruction, StringValueSchema};

/// Only completed strings enter this list. User capture Gets and conversions
/// happen once before append; final publication only copies their UTF-16 units.
pub(super) struct SubstitutionPieces(ArgumentListConstruction);

impl SubstitutionPieces {
    pub(super) fn new(s: &RuntimeSchema, f: &mut Function) -> Self {
        Self(ArgumentListConstruction::new(s, f))
    }

    pub(super) fn append(
        &self,
        string: &GcLocal<StringValue>,
        s: &RuntimeSchema,
        f: &mut Function,
    ) {
        let value = s.reserve_value_local(f);
        value.set_reference(string, s, f);
        self.0.append(&value, s, f);
        value.clear(f);
    }

    pub(super) fn publish(
        self,
        b: &mut FunctionBuilder<'_>,
        total: I64Local,
        f: &mut Function,
    ) -> GcLocal<StringValue> {
        let s = b.runtime_schema();
        let pieces = self.0.finish(b, f);
        let length = s.reserve_i32_local(f);
        total.load(f);
        f.instruction(&Instruction::I32WrapI64);
        length.store(f);
        let construction = StringConstruction::allocate(s, s.reserve_gc_local(f), length, f);
        let index = s.reserve_i32_local(f);
        let count = s.reserve_i32_local(f);
        let destination = s.reserve_i32_local(f);
        let piece_length = s.reserve_i32_local(f);
        let value = s.reserve_value_local(f);
        index.set_constant(0, f);
        destination.set_constant(0, f);
        s.array_type::<ValueArray>().length(&pieces, s, f);
        count.store(f);
        let done = b.open_frame(ControlFrameKind::Block, f);
        let next = b.open_frame(ControlFrameKind::Loop, f);
        index.load(f);
        count.load(f);
        f.instruction(&Instruction::I32GeU);
        b.emit_branch_if_to_target(done, f);
        b.emit_argument_vector_entry_to_value(&pieces, index, &value, f);
        let string = s
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<StringValue>(s, f), f);
        let units = s.reserve_gc_local(f).initialize(
            s.struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&string, s, f)
                .reference(),
            f,
        );
        s.array_type::<CodeUnitArray>().length(&units, s, f);
        piece_length.store(f);
        construction.copy_units_from(destination, &units, piece_length, s, f);
        destination.load(f);
        piece_length.load(f);
        f.instruction(&Instruction::I32Add);
        destination.store(f);
        units.clear(f);
        string.clear(f);
        index.load(f);
        f.instruction(&Instruction::I32Const(1));
        f.instruction(&Instruction::I32Add);
        index.store(f);
        b.emit_branch_to_target(next, f);
        b.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        b.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        let result = s
            .reserve_gc_local(f)
            .initialize(construction.publish(s, f), f);
        pieces.clear(f);
        value.clear(f);
        for local in [length, index, count, destination, piece_length] {
            s.release_i32_local(local, f);
        }
        result
    }
}

/// Whole replacement output retains only completed Strings. Each append
/// validates the bounded UTF-16 length before the final exact-size copy.
#[must_use = "replacement pieces must be consumed by final publication"]
pub(super) struct ReplacementPieces {
    pieces: SubstitutionPieces,
    total: I64Local,
}

impl ReplacementPieces {
    pub(super) fn new(s: &RuntimeSchema, f: &mut Function) -> Self {
        let total = s.reserve_i64_local(f);
        total.set_constant(0, f);
        Self {
            pieces: SubstitutionPieces::new(s, f),
            total,
        }
    }

    pub(super) fn append(
        &self,
        b: &mut FunctionBuilder<'_>,
        text: &GcLocal<StringValue>,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = b.runtime_schema();
        let length = s.reserve_i64_local(f);
        b.emit_native_gc_string_length(text, length, f);
        self.total.load(f);
        length.load(f);
        f.instruction(&Instruction::I64Add);
        self.total.store(f);
        self.total.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        b.emit_native_string_error_if(
            RuntimeErrorMessage::REPLACEMENT_RESULT_WOULD_EXCEED_MAXIMUM_STRING_LENGTH,
            NativeErrorKind::RangeError,
            output,
            exit,
            f,
        )?;
        // Empty pieces have no observable identity and contribute no units.
        // Keep their completed conversions, but allocate no list node for them.
        length.load(f);
        f.instruction(&Instruction::I64Eqz);
        b.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Else);
        self.pieces.append(text, s, f);
        b.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i64_local(length, f);
        Ok(())
    }

    pub(super) fn publish(
        self,
        b: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> GcLocal<StringValue> {
        let s = b.runtime_schema();
        let result = self.pieces.publish(b, self.total, f);
        s.release_i64_local(self.total, f);
        result
    }
}
