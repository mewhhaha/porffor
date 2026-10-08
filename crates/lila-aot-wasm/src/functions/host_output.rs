//! GC strings cross the print boundary only as transient UTF-8 bytes.

use super::*;
use crate::gc_types::{CodeUnitArray, GcLocal, I32Local, I64Local, StringValue, StringValueSchema};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_host_print_string(
        &mut self,
        string: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(string, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        let capacity = schema.reserve_i64_local(function);
        length.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Mul);
        capacity.store(function);
        // The native byte-span boundary accepts nonnegative signed i32s.
        capacity.load(function);
        function.instruction(&Instruction::I64Const(i32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let pointer = schema.reserve_i64_local(function);
        schema
            .call_helper(
                crate::runtime_helpers::TransientByteAllocArguments::new(capacity),
                self.runtime_helper_base()?,
                function,
            )
            .store(pointer, function);
        pointer.load(function);
        capacity.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(i32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let index = schema.reserve_i32_local(function);
        let offset = schema.reserve_i32_local(function);
        let code_point = schema.reserve_i32_local(function);
        let low = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        function.instruction(&Instruction::I32Const(0));
        offset.store(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(code_point, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0xd800));
        function.instruction(&Instruction::I32GeU);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0xdbff));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0));
        low.store(function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32LtU);
        self.open_frame(ControlFrameKind::If, function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(low, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        low.load(function);
        function.instruction(&Instruction::I32Const(0xdc00));
        function.instruction(&Instruction::I32GeU);
        low.load(function);
        function.instruction(&Instruction::I32Const(0xdfff));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0xd800));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Const(10));
        function.instruction(&Instruction::I32Shl);
        low.load(function);
        function.instruction(&Instruction::I32Const(0xdc00));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I32Const(0x10000));
        function.instruction(&Instruction::I32Add);
        code_point.store(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0xfffd));
        code_point.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0xdc00));
        function.instruction(&Instruction::I32GeU);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0xdfff));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(0xfffd));
        code_point.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0x7f));
        function.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 0, 0x7f, 0, function);
        function.instruction(&Instruction::Else);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0x7ff));
        function.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 6, 0x1f, 0xc0, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 0, 0x3f, 0x80, function);
        function.instruction(&Instruction::Else);
        code_point.load(function);
        function.instruction(&Instruction::I32Const(0xffff));
        function.instruction(&Instruction::I32LeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 12, 0x0f, 0xe0, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 6, 0x3f, 0x80, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 0, 0x3f, 0x80, function);
        function.instruction(&Instruction::Else);
        self.emit_print_utf8_byte(pointer, offset, code_point, 18, 0x07, 0xf0, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 12, 0x3f, 0x80, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 6, 0x3f, 0x80, function);
        self.emit_print_utf8_byte(pointer, offset, code_point, 0, 0x3f, 0x80, function);
        for _ in 0..3 {
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        pointer.load(function);
        function.instruction(&Instruction::I32WrapI64);
        offset.load(function);
        function.instruction(&Instruction::Call(HOST_PRINT_IMPORT_FUNCTION_INDEX));
        schema.release_i32_local(low, function);
        schema.release_i32_local(code_point, function);
        schema.release_i32_local(offset, function);
        schema.release_i32_local(index, function);
        schema.release_i64_local(pointer, function);
        schema.release_i64_local(capacity, function);
        schema.release_i32_local(length, function);
        units.clear(function);
        Ok(())
    }

    fn emit_print_utf8_byte(
        &self,
        pointer: I64Local,
        offset: I32Local,
        code_point: I32Local,
        shift: i32,
        mask: i32,
        prefix: i32,
        function: &mut Function,
    ) {
        pointer.load(function);
        function.instruction(&Instruction::I32WrapI64);
        offset.load(function);
        function.instruction(&Instruction::I32Add);
        code_point.load(function);
        if shift != 0 {
            function.instruction(&Instruction::I32Const(shift));
            function.instruction(&Instruction::I32ShrU);
        }
        function.instruction(&Instruction::I32Const(mask));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Const(prefix));
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Store8(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        offset.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        offset.store(function);
    }
}
