//! Fresh JSON frames publish unique Array data properties without key conversion.
use super::*;
use crate::heap::StoredPropertyAttributes;

/// One immutable undefined cell supplies both absent accessor fields. Each data
/// descriptor remains distinct, and changing a descriptor replaces its fields.
pub(super) struct JsonArrayDataDescriptors {
    absent: GcLocal<StoredValue>,
}
impl JsonArrayDataDescriptors {
    pub(super) fn new(builder: &FunctionBuilder<'_>, f: &mut Function) -> Self {
        let s = builder.runtime_schema();
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let absent = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&undefined, f), f);
        undefined.clear(f);
        Self { absent }
    }
    fn descriptor(
        &self,
        value: &GcLocal<StoredValue>,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) -> GcLocal<PropertyDescriptor> {
        let s = builder.runtime_schema();
        s.reserve_gc_local(f).initialize(
            s.struct_type::<PropertyDescriptor>().construct(
                (
                    GcOperand::descriptor_word(
                        StoredPropertyAttributes::Data {
                            writable: true,
                            enumerable: true,
                            configurable: true,
                        }
                        .descriptor_word(),
                    ),
                    GcOperand::reference(value, s),
                    GcOperand::reference(&self.absent, s),
                    GcOperand::reference(&self.absent, s),
                ),
                f,
            ),
            f,
        )
    }
    pub(super) fn clear(self, f: &mut Function) {
        self.absent.clear(f);
    }
}

/// No constructor accepts a JavaScript Array. The parser's private frame owns
/// its unpublished container and next unique ordinal; consuming append advances
/// that ordinal after successful publication, so this insertion cannot repeat.
struct FreshJsonArrayElement<'a> {
    frame: &'a GcLocal<JsonParseFrame>,
    array: GcLocal<ArrayObject>,
    ordinal: I64Local,
}
impl<'a> FreshJsonArrayElement<'a> {
    fn from_frame(
        frame: &'a GcLocal<JsonParseFrame>,
        ordinal: I64Local,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Self {
        let s = builder.runtime_schema();
        let state = s.reserve_i32_local(f);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::STATE)
            .read(frame, s, f)
            .store(state, f);
        state.load(f);
        f.instruction(&Instruction::I32Const(
            JsonParseFrameState::ArrayValue.wire_code(),
        ));
        f.instruction(&Instruction::I32Ne);
        builder.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        builder.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i32_local(state, f);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::INDEX)
            .read(frame, s, f)
            .store_i64(ordinal, f);
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<JsonParseFrame>()
                .field(JsonParseFrameSchema::CONTAINER)
                .read(frame, s, f)
                .reference(),
            f,
        );
        let container = s.reserve_value_local(f);
        s.struct_type::<StoredValue>()
            .read_into(&stored, &container, s, f);
        let array = s
            .reserve_gc_local(f)
            .initialize(container.cast_reference::<ArrayObject>(s, f), f);
        container.clear(f);
        stored.clear(f);
        Self {
            frame,
            array,
            ordinal,
        }
    }
    fn append(
        self,
        value: &GcLocal<StoredValue>,
        descriptors: &JsonArrayDataDescriptors,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = builder.runtime_schema();
        let next = s.reserve_i64_local(f);
        self.ordinal.load(f);
        f.instruction(&Instruction::I64Const(1));
        f.instruction(&Instruction::I64Add);
        next.store(f);
        self.ordinal.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64LtU);
        builder.open_frame(ControlFrameKind::If, f);
        // StoredValue is immutable. The parse record and property can share it;
        // future writes replace a descriptor field rather than modifying it.
        let descriptor = descriptors.descriptor(value, builder, f);
        builder.emit_array_indexed_publish_descriptor(&self.array, self.ordinal, &descriptor, f)?;
        s.struct_type::<ArrayObject>()
            .field(ArrayObjectSchema::LENGTH)
            .write(&self.array, GcOperand::i64_local(next), s, f);
        descriptor.clear(f);
        f.instruction(&Instruction::Else);
        // 2^32-1 and higher are ordinary String properties, not Array indices.
        // Retain the original generic definition path without narrowing them.
        let container = s.reserve_value_local(f);
        container.set_reference(&self.array, s, f);
        let element = s.reserve_value_local(f);
        s.struct_type::<StoredValue>()
            .read_into(value, &element, s, f);
        let key = builder.emit_json_index_key(self.ordinal, f)?;
        builder.emit_json_define(&container, &key, &element, false, f)?;
        key.clear(f);
        element.clear(f);
        container.clear(f);
        builder.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.struct_type::<JsonParseFrame>()
            .field(JsonParseFrameSchema::INDEX)
            .write(self.frame, GcOperand::i64_local(next), s, f);
        s.release_i64_local(next, f);
        self.array.clear(f);
        Ok(())
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_json_parse_array_append(
        &mut self,
        frame: &GcLocal<JsonParseFrame>,
        value: &GcLocal<StoredValue>,
        descriptors: &JsonArrayDataDescriptors,
        ordinal: I64Local,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        FreshJsonArrayElement::from_frame(frame, ordinal, self, f).append(
            value,
            descriptors,
            self,
            f,
        )
    }
}
