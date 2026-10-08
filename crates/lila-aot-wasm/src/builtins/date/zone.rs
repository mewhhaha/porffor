//! Date-owned LocalTime and compatible UTC selection followed by TimeClip.

use super::{
    CompletedDateClipLocals, CompletedDateMakeDateLocals, FiniteDateValueLocals,
    PreparedDateClipLocals,
};
use crate::builtins::system_time_zone::ResolvedSystemTimeZoneLocals;
use crate::gc_types::I64Local;
use crate::{emit::FunctionBuilder, *};
use lila_intl::SystemTimeZoneKind;

const CLIP_LIMIT_MILLISECONDS: i64 = 8_640_000_000_000_000;
const ONE_DAY_MILLISECONDS: i64 = 86_400_000;

pub(in crate::builtins) struct DateUtcCoordinateLocals {
    seconds: I64Local,
    nano: I64Local,
}
pub(in crate::builtins) struct DateLocalCoordinateLocals {
    seconds: I64Local,
    nano: I64Local,
}
impl DateUtcCoordinateLocals {
    pub(in crate::builtins) fn floor_seconds(&self) -> I64Local {
        self.seconds
    }
    pub(in crate::builtins) fn nanosecond(&self) -> I64Local {
        self.nano
    }
    fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        builder
            .runtime_schema()
            .release_i64_local(self.nano, function);
        builder
            .runtime_schema()
            .release_i64_local(self.seconds, function);
    }
}
impl DateLocalCoordinateLocals {
    pub(in crate::builtins) fn floor_seconds(&self) -> I64Local {
        self.seconds
    }
    pub(in crate::builtins) fn nanosecond(&self) -> I64Local {
        self.nano
    }
    fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        builder
            .runtime_schema()
            .release_i64_local(self.nano, function);
        builder
            .runtime_schema()
            .release_i64_local(self.seconds, function);
    }
}

pub(in crate::builtins) struct DateLocalProjectionLocals {
    local_payload: I64Local,
    system_zone: ResolvedSystemTimeZoneLocals,
    offset_seconds: I64Local,
}
impl DateLocalProjectionLocals {
    pub(in crate::builtins) fn local_payload(&self) -> I64Local {
        self.local_payload
    }
    pub(in crate::builtins) fn offset_seconds(&self) -> I64Local {
        self.offset_seconds
    }
    pub(in crate::builtins) fn system_zone(&self) -> &ResolvedSystemTimeZoneLocals {
        &self.system_zone
    }
    pub(in crate::builtins) fn release(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        builder
            .runtime_schema()
            .release_i64_local(self.offset_seconds, function);
        self.system_zone.release(builder, function);
        builder
            .runtime_schema()
            .release_i64_local(self.local_payload, function);
    }
}

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_date_local_projection(
        &mut self,
        value: &FiniteDateValueLocals,
        function: &mut Function,
    ) -> Result<DateLocalProjectionLocals, EmitError> {
        let local_payload = self.runtime_schema().reserve_i64_local(function);
        let system_zone = self.emit_system_time_zone(function)?;
        let offset_seconds = self.runtime_schema().reserve_i64_local(function);
        let instant = DateUtcCoordinateLocals {
            seconds: self.runtime_schema().reserve_i64_local(function),
            nano: self.runtime_schema().reserve_i64_local(function),
        };
        // A private captured Date slot has already completed TimeClip. This
        // callback excludes NaN; no new observable range validation occurs.
        self.emit_date_milliseconds_pair(value.payload(), instant.seconds, instant.nano, function);
        system_zone.kind_local().load(function);
        function.instruction(&Instruction::I64Const(SystemTimeZoneKind::Named.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_named_offset_into(&system_zone, &instant, offset_seconds, function)?;
        function.instruction(&Instruction::Else);
        system_zone.fixed_offset_seconds().load(function);
        offset_seconds.store(function);
        function.instruction(&Instruction::End);
        value.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        offset_seconds.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Const(1000.0.into()));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::I64ReinterpretF64);
        local_payload.store(function);
        instant.release(self, function);
        Ok(DateLocalProjectionLocals {
            local_payload,
            system_zone,
            offset_seconds,
        })
    }

    pub(in crate::builtins) fn emit_date_utc_time_clip_from_make_date_into(
        &mut self,
        prepared: PreparedDateClipLocals,
        local: &CompletedDateMakeDateLocals,
        function: &mut Function,
    ) -> Result<CompletedDateClipLocals, EmitError> {
        let selected = self.runtime_schema().reserve_i64_local(function);
        // All MakeDate coercions and math have completed. Every admitted
        // offset is strictly below one day, proving the far-out clip shortcut.
        local.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        local.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        local.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(
            ((CLIP_LIMIT_MILLISECONDS + ONE_DAY_MILLISECONDS) as f64).into(),
        ));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::F64Const(f64::NAN.into()));
        function.instruction(&Instruction::I64ReinterpretF64);
        selected.store(function);
        function.instruction(&Instruction::Else);
        let system = self.emit_system_time_zone(function)?;
        let coordinate = DateLocalCoordinateLocals {
            seconds: self.runtime_schema().reserve_i64_local(function),
            nano: self.runtime_schema().reserve_i64_local(function),
        };
        self.emit_date_milliseconds_pair(
            local.payload(),
            coordinate.seconds,
            coordinate.nano,
            function,
        );
        let offset = self.runtime_schema().reserve_i64_local(function);
        system.kind_local().load(function);
        function.instruction(&Instruction::I64Const(SystemTimeZoneKind::Named.code()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_date_named_compatible_offset_into(&system, &coordinate, offset, function)?;
        function.instruction(&Instruction::Else);
        system.fixed_offset_seconds().load(function);
        offset.store(function);
        function.instruction(&Instruction::End);
        local.payload().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        offset.load(function);
        function.instruction(&Instruction::F64ConvertI64S);
        function.instruction(&Instruction::F64Const(1000.0.into()));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        selected.store(function);
        self.runtime_schema().release_i64_local(offset, function);
        coordinate.release(self, function);
        system.release(self, function);
        function.instruction(&Instruction::End);
        let output = self.emit_date_time_clip_into(prepared, selected, function);
        self.runtime_schema().release_i64_local(selected, function);
        Ok(output)
    }

    // Private callers prove integral finite milliseconds on the exact f64
    // integer grid before conversion; this is normalization, not a raw mint.
    fn emit_date_milliseconds_pair(
        &mut self,
        payload: I64Local,
        seconds: I64Local,
        nano: I64Local,
        function: &mut Function,
    ) {
        let milliseconds = self.runtime_schema().reserve_i64_local(function);
        payload.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64S);
        milliseconds.store(function);
        milliseconds.load(function);
        function.instruction(&Instruction::I64Const(1000));
        function.instruction(&Instruction::I64DivS);
        seconds.store(function);
        milliseconds.load(function);
        function.instruction(&Instruction::I64Const(1000));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(1_000_000));
        function.instruction(&Instruction::I64Mul);
        nano.store(function);
        nano.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        seconds.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        seconds.store(function);
        nano.load(function);
        function.instruction(&Instruction::I64Const(1_000_000_000));
        function.instruction(&Instruction::I64Add);
        nano.store(function);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(milliseconds, function);
    }
}
