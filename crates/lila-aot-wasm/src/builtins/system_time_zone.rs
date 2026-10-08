//! One checked projection of the immutable defining-Realm system-zone snapshot.
use crate::builtins::intl_provider_wire::IntlByteArrayReader;
use crate::gc_types::{
    CodeUnitArray, GcHostImport, GcLocal, I32Local, I64Local, StringValue, StringValueSchema,
};
use crate::{emit::FunctionBuilder, *};
use lila_intl::{
    SystemTimeZoneKind, MAX_TIME_ZONE_IDENTIFIER_BYTES, SYSTEM_TIME_ZONE_WIRE_VERSION,
};

pub(in crate::builtins) struct ResolvedSystemTimeZoneLocals {
    identifier: GcLocal<StringValue>,
    kind: I64Local,
    fixed_seconds: I64Local,
}
impl ResolvedSystemTimeZoneLocals {
    pub(in crate::builtins) fn identifier(&self) -> &GcLocal<StringValue> {
        &self.identifier
    }
    pub(in crate::builtins) fn kind_local(&self) -> I64Local {
        self.kind
    }
    pub(in crate::builtins) fn fixed_offset_seconds(&self) -> I64Local {
        self.fixed_seconds
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        self.identifier.clear(function);
        builder
            .runtime_schema()
            .release_i64_local(self.fixed_seconds, function);
        builder
            .runtime_schema()
            .release_i64_local(self.kind, function);
    }
}

fn corrupt_if(function: &mut Function) {
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_system_time_zone(
        &mut self,
        function: &mut Function,
    ) -> Result<ResolvedSystemTimeZoneLocals, EmitError> {
        let schema = self.runtime_schema();
        let import = self
            .functions
            .gc_host_imports()
            .get(GcHostImport::SystemTimeZoneSnapshot)
            .ok_or_else(|| EmitError::unsupported("missing GC system time zone snapshot import"))?;
        let bytes = schema
            .reserve_gc_local(function)
            .initialize(import.call_system_time_zone_snapshot(function)?, function);
        let reader = IntlByteArrayReader::new(&bytes, schema, function);
        let version = schema.reserve_i64_local(function);
        let kind = schema.reserve_i64_local(function);
        let seconds = schema.reserve_i64_local(function);
        reader.read_u64(version, schema, function);
        reader.read_u64(kind, schema, function);
        reader.read_u64(seconds, schema, function);
        // The last header word is the UTF-8 byte extent consumed by this reader.
        let identifier = reader.read_utf8(schema, function);
        reader.finish(schema, function);
        bytes.clear(function);
        version.load(function);
        function.instruction(&Instruction::I64Const(SYSTEM_TIME_ZONE_WIRE_VERSION));
        function.instruction(&Instruction::I64Ne);
        kind.load(function);
        function.instruction(&Instruction::I64Const(SystemTimeZoneKind::Named.code()));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        corrupt_if(function);
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StringValue>()
                .field(StringValueSchema::CODE_UNITS)
                .read(&identifier, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I32Eqz);
        length.load(function);
        function.instruction(&Instruction::I32Const(
            MAX_TIME_ZONE_IDENTIFIER_BYTES as i32,
        ));
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::I32Or);
        corrupt_if(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        self.emit_branch_if_to_target(exit, function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I32Const(0x7f));
        function.instruction(&Instruction::I32GtU);
        corrupt_if(function);
        unit.load(function);
        function.instruction(&Instruction::I32Eqz);
        corrupt_if(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        kind.load(function);
        function.instruction(&Instruction::I64Const(
            SystemTimeZoneKind::FixedOffset.code(),
        ));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_system_fixed_zone_association(&units, length, seconds, function);
        function.instruction(&Instruction::Else);
        seconds.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        corrupt_if(function);
        let fold = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        fold.store(function);
        let equal = schema.reserve_i32_local(function);
        let utc = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("UTC", function)?,
            function,
        );
        self.emit_gc_string_equality(&identifier, &utc, fold, equal, function);
        kind.load(function);
        function.instruction(&Instruction::I64Const(SystemTimeZoneKind::Utc.code()));
        function.instruction(&Instruction::I64Eq);
        equal.load(function);
        function.instruction(&Instruction::I32Ne);
        corrupt_if(function);
        utc.clear(function);
        schema.release_i32_local(equal, function);
        schema.release_i32_local(fold, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        units.clear(function);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(length, function);
        schema.release_i64_local(version, function);
        Ok(ResolvedSystemTimeZoneLocals {
            identifier,
            kind,
            fixed_seconds: seconds,
        })
    }

    fn emit_system_fixed_zone_association(
        &mut self,
        units: &GcLocal<CodeUnitArray>,
        length: I32Local,
        seconds: I64Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let sign = schema.reserve_i32_local(function);
        let hours = schema.reserve_i64_local(function);
        let minutes = schema.reserve_i64_local(function);
        let expected = schema.reserve_i64_local(function);
        length.load(function);
        function.instruction(&Instruction::I32Const(6));
        function.instruction(&Instruction::I32Ne);
        corrupt_if(function);
        for (position, output) in [(0, sign), (3, unit)] {
            function.instruction(&Instruction::I32Const(position));
            index.store(function);
            schema
                .array_type::<CodeUnitArray>()
                .read(units, index, schema, function)
                .store(output, function);
        }
        sign.load(function);
        function.instruction(&Instruction::I32Const(i32::from(b'+')));
        function.instruction(&Instruction::I32Ne);
        sign.load(function);
        function.instruction(&Instruction::I32Const(i32::from(b'-')));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I32And);
        unit.load(function);
        function.instruction(&Instruction::I32Const(i32::from(b':')));
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::I32Or);
        corrupt_if(function);
        for (destination, positions, maximum) in [(hours, [1, 2], 23), (minutes, [4, 5], 59)] {
            function.instruction(&Instruction::I64Const(0));
            destination.store(function);
            for position in positions {
                function.instruction(&Instruction::I32Const(position));
                index.store(function);
                schema
                    .array_type::<CodeUnitArray>()
                    .read(units, index, schema, function)
                    .store(unit, function);
                unit.load(function);
                function.instruction(&Instruction::I32Const(i32::from(b'0')));
                function.instruction(&Instruction::I32Sub);
                unit.store(function);
                unit.load(function);
                function.instruction(&Instruction::I32Const(9));
                function.instruction(&Instruction::I32GtU);
                corrupt_if(function);
                destination.load(function);
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::I64Mul);
                unit.load(function);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::I64Add);
                destination.store(function);
            }
            destination.load(function);
            function.instruction(&Instruction::I64Const(maximum));
            function.instruction(&Instruction::I64GtU);
            corrupt_if(function);
        }
        hours.load(function);
        function.instruction(&Instruction::I64Const(3600));
        function.instruction(&Instruction::I64Mul);
        minutes.load(function);
        function.instruction(&Instruction::I64Const(60));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        expected.store(function);
        sign.load(function);
        function.instruction(&Instruction::I32Const(i32::from(b'-')));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        expected.load(function);
        function.instruction(&Instruction::I64Eqz);
        corrupt_if(function);
        function.instruction(&Instruction::I64Const(0));
        expected.load(function);
        function.instruction(&Instruction::I64Sub);
        expected.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        seconds.load(function);
        expected.load(function);
        function.instruction(&Instruction::I64Ne);
        corrupt_if(function);
        for value in [expected, minutes, hours] {
            schema.release_i64_local(value, function);
        }
        for value in [sign, unit, index] {
            schema.release_i32_local(value, function);
        }
    }
}
