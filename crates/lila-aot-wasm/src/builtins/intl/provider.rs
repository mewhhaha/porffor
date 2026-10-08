//! Pure native transformations mint canonical tags before their component view.
use super::*;
use crate::builtins::intl_provider_wire::{IntlByteArrayBuilder, IntlByteArrayReader};
use lila_intl::{
    IntlOperation, LocaleTransformError, LocaleTransformRequest, LocaleTransformResult,
};

pub(super) struct CanonicalLocaleComponents {
    pub(super) tag: GcLocal<StringValue>,
    pub(super) language: GcLocal<StringValue>,
    pub(super) script: GcLocal<StringValue, Nullable>,
    pub(super) region: GcLocal<StringValue, Nullable>,
    pub(super) variants: GcLocal<StringValue, Nullable>,
    pub(super) base_name: GcLocal<StringValue>,
    pub(super) suffix: GcLocal<StringValue>,
}
impl CanonicalLocaleComponents {
    pub(super) fn clear(self, function: &mut Function) {
        self.suffix.clear(function);
        self.base_name.clear(function);
        self.variants.clear(function);
        self.region.clear(function);
        self.script.clear(function);
        self.language.clear(function);
        self.tag.clear(function);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_intl_provider_locale_transform<O>(
        &mut self,
        input: &GcLocal<StringValue>,
        error: RuntimeErrorMessage,
        function: &mut Function,
    ) -> Result<GcLocal<StringValue>, EmitError>
    where
        O: IntlOperation<
            Request = LocaleTransformRequest,
            Response = LocaleTransformResult,
            Error = LocaleTransformError,
        >,
    {
        let schema = self.runtime_schema();
        let request = IntlByteArrayBuilder::with_operation(O::HOST_OP, schema, function);
        request.append_remaining_utf8(input, schema, function);
        let request = request.finish(schema, function);
        let reply = self.emit_intl_provider_byte_call(&request, function)?;
        reply.load(schema, function).is_null(function);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_intl_number_range_error(error, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let response = schema.reserve_gc_local(function).initialize(
            reply.load(schema, function).require_non_null(function),
            function,
        );
        let reader = IntlByteArrayReader::new(&response, schema, function);
        let tag = reader.consume_remaining_utf8(schema, function);
        reader.finish(schema, function);
        response.clear(function);
        reply.clear(function);
        request.clear(function);
        Ok(tag)
    }

    pub(super) fn emit_intl_locale_token_end(
        &self,
        units: &GcLocal<CodeUnitArray>,
        length: I32Local,
        start: I32Local,
        end: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let unit = schema.reserve_i32_local(function);
        start.load(function);
        end.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        end.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<CodeUnitArray>()
            .read(units, end, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I32Const(b'-' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::BrIf(1));
        end.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        end.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(unit, function);
    }
    pub(super) fn emit_intl_locale_slice(
        &self,
        tag: &GcLocal<StringValue>,
        start: I32Local,
        end: I32Local,
        function: &mut Function,
    ) -> GcLocal<StringValue> {
        let schema = self.runtime_schema();
        let begin = schema.reserve_i64_local(function);
        let finish = schema.reserve_i64_local(function);
        start.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        begin.store(function);
        end.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        finish.store(function);
        let text = schema.reserve_gc_local(function).initialize(
            self.emit_gc_string_slice(tag, begin, finish, function),
            function,
        );
        schema.release_i64_local(finish, function);
        schema.release_i64_local(begin, function);
        text
    }

    // The native transformation is the syntax/canonicalization authority. This
    // projection walks only its canonical ASCII result and invokes no JS code.
    pub(super) fn emit_intl_locale_components(
        &mut self,
        tag: GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<CanonicalLocaleComponents, EmitError> {
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&tag, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let position = schema.reserve_i32_local(function);
        let end = schema.reserve_i32_local(function);
        let base_end = schema.reserve_i32_local(function);
        let variants_start = schema.reserve_i32_local(function);
        let zero = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        set_i32(zero, 0, function);
        self.emit_intl_locale_token_end(&units, length, zero, end, function);
        let language = self.emit_intl_locale_slice(&tag, zero, end, function);
        end.load(function);
        base_end.store(function);
        end.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        position.store(function);
        let script = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        let region = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        let variants = schema
            .reserve_gc_local::<StringValue, Nullable>(function)
            .initialize_null(schema, function);
        for (is_script, output) in [(true, &script), (false, &region)] {
            position.load(function);
            length.load(function);
            function.instruction(&Instruction::I32LtU);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_intl_locale_token_end(&units, length, position, end, function);
            end.load(function);
            position.load(function);
            function.instruction(&Instruction::I32Sub);
            if is_script {
                function.instruction(&Instruction::I32Const(4));
                function.instruction(&Instruction::I32Eq);
                schema
                    .array_type::<CodeUnitArray>()
                    .read(&units, position, schema, function)
                    .store(unit, function);
                unit.load(function);
                function.instruction(&Instruction::I32Const(0x20));
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Const(b'a' as i32));
                function.instruction(&Instruction::I32Sub);
                function.instruction(&Instruction::I32Const(25));
                function.instruction(&Instruction::I32LeU);
                function.instruction(&Instruction::I32And);
            } else {
                function.instruction(&Instruction::I32Const(2));
                function.instruction(&Instruction::I32Eq);
                end.load(function);
                position.load(function);
                function.instruction(&Instruction::I32Sub);
                function.instruction(&Instruction::I32Const(3));
                function.instruction(&Instruction::I32Eq);
                function.instruction(&Instruction::I32Or);
            }
            self.open_frame(ControlFrameKind::If, function);
            let component = self.emit_intl_locale_slice(&tag, position, end, function);
            output.replace(component.load(schema, function).nullable(), function);
            component.clear(function);
            end.load(function);
            base_end.store(function);
            end.load(function);
            function.instruction(&Instruction::I32Const(1));
            function.instruction(&Instruction::I32Add);
            position.store(function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        position.load(function);
        variants_start.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        position.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_intl_locale_token_end(&units, length, position, end, function);
        end.load(function);
        position.load(function);
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::BrIf(1));
        end.load(function);
        base_end.store(function);
        end.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        position.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        base_end.load(function);
        variants_start.load(function);
        function.instruction(&Instruction::I32GtU);
        self.open_frame(ControlFrameKind::If, function);
        let text = self.emit_intl_locale_slice(&tag, variants_start, base_end, function);
        variants.replace(text.load(schema, function).nullable(), function);
        text.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let base_name = self.emit_intl_locale_slice(&tag, zero, base_end, function);
        let suffix = self.emit_intl_locale_slice(&tag, base_end, length, function);
        for local in [unit, zero, variants_start, base_end, end, position, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
        Ok(CanonicalLocaleComponents {
            tag,
            language,
            script,
            region,
            variants,
            base_name,
            suffix,
        })
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_intl_locale_provider_response(
        &mut self,
        operation: lila_intl::IntlHostOp,
        tag: &GcLocal<StringValue>,
        framed: bool,
        function: &mut Function,
    ) -> Result<GcLocal<ByteArray>, EmitError> {
        let schema = self.runtime_schema();
        let request = IntlByteArrayBuilder::with_operation(operation, schema, function);
        if framed {
            request.append_u64_constant(
                lila_intl::LOCALE_INFORMATION_WIRE_VERSION,
                schema,
                function,
            );
            request.append_u64_constant(u64::from(operation.code()) * 2, schema, function);
            request.append_utf8(tag, schema, function);
        } else {
            request.append_remaining_utf8(tag, schema, function);
        }
        let request = request.finish(schema, function);
        let reply = self.emit_intl_provider_byte_call(&request, function)?;
        reply.load(schema, function).is_null(function);
        self.emit_intl_locale_provider_fault_if(function);
        let response = schema.reserve_gc_local(function).initialize(
            reply.load(schema, function).require_non_null(function),
            function,
        );
        reply.clear(function);
        request.clear(function);
        Ok(response)
    }
    pub(super) fn emit_intl_locale_provider_fault_if(&self, function: &mut Function) {
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }
}
