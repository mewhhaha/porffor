//! The pinned ICU4X fast Persian rule, emitted as integer arithmetic.
//! The correction list is shared by forward dates, inverse dates and leap years.
//! Parent factories regulate complete calendar fields before reaching this leaf.

use super::*;
use crate::gc_types::I64Local;

// calendrical_calculations 0.2.4, ICU4X c9fac4e625ccb2c6a7aa35079fff9709db4385ac,
// utils/calendrical_calculations/src/persian.rs. Its fast 33-year rule uses this
// Julian 622-03-19 epoch (RD 226896), relative to ISO 1970-01-01 (RD 719163).
const PERSIAN_EPOCH_DAY: i64 = -492_267;
const NON_LEAP_CORRECTION: [i64; 78] = [
    1502, 1601, 1634, 1667, 1700, 1733, 1766, 1799, 1832, 1865, 1898, 1931, 1964, 1997, 2030, 2059,
    2063, 2096, 2129, 2158, 2162, 2191, 2195, 2224, 2228, 2257, 2261, 2290, 2294, 2323, 2327, 2356,
    2360, 2389, 2393, 2422, 2426, 2455, 2459, 2488, 2492, 2521, 2525, 2554, 2558, 2587, 2591, 2620,
    2624, 2653, 2657, 2686, 2690, 2719, 2723, 2748, 2752, 2756, 2781, 2785, 2789, 2818, 2822, 2847,
    2851, 2855, 2880, 2884, 2888, 2913, 2917, 2921, 2946, 2950, 2954, 2979, 2983, 2987,
];

impl FunctionBuilder<'_> {
    /// The denominator is a positive literal. The stack retains the quotient
    /// while reading the original remainder, so numerator and output may alias.
    fn emit_temporal_persian_div_euclid(
        &self,
        numerator: I64Local,
        denominator: i64,
        out: I64Local,
        function: &mut Function,
    ) {
        (numerator).load(function);
        function.instruction(&Instruction::I64Const(denominator));
        function.instruction(&Instruction::I64DivS);
        (numerator).load(function);
        function.instruction(&Instruction::I64Const(denominator));
        function.instruction(&Instruction::I64RemS);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        (out).store(function);
    }

    fn emit_temporal_persian_correction_i32(
        &self,
        year: I64Local,
        following_year: bool,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I32Const(0));
        for correction in NON_LEAP_CORRECTION {
            (year).load(function);
            function.instruction(&Instruction::I64Const(
                correction + i64::from(following_year),
            ));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
    }

    fn emit_temporal_persian_leap_i32(&mut self, year: I64Local, function: &mut Function) {
        self.emit_temporal_persian_correction_i32(year, false, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::Else);
        self.emit_temporal_persian_correction_i32(year, true, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        let remainder = self.runtime_schema().reserve_i64_local(function);
        (year).load(function);
        function.instruction(&Instruction::I64Const(25));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(33));
        function.instruction(&Instruction::I64RemS);
        (remainder).store(function);
        (remainder).load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        self.open_frame(ControlFrameKind::If, function);
        (remainder).load(function);
        function.instruction(&Instruction::I64Const(33));
        function.instruction(&Instruction::I64Add);
        (remainder).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (remainder).load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64LtS);
        self.runtime_schema().release_i64_local(remainder, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_temporal_persian_month_length(
        &self,
        month: I64Local,
        leap: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        (month).load(function);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::Else);
        (month).load(function);
        function.instruction(&Instruction::I64Const(11));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::Else);
        (leap).load(function);
        function.instruction(&Instruction::I64Const(29));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        (out).store(function);
    }

    pub(super) fn emit_temporal_persian_days_in_month(
        &mut self,
        year: I64Local,
        month: I64Local,
        out: I64Local,
        function: &mut Function,
    ) {
        let leap = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_persian_leap_i32(year, function);
        function.instruction(&Instruction::I64ExtendI32U);
        (leap).store(function);
        self.emit_temporal_persian_month_length(month, leap, out, function);
        self.runtime_schema().release_i64_local(leap, function);
    }

    /// Number of whole days before the given month, on the Wasm stack.
    fn emit_temporal_persian_month_offset(&self, month: I64Local, function: &mut Function) {
        (month).load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::Else);
        (month).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::End);
    }

    /// Exact fixed_from_fast_persian translated from RD to Unix epoch days.
    /// The triple is not mutated; parent regulation has bounded its year.
    fn emit_temporal_persian_epoch_days(
        &mut self,
        fields: [I64Local; 3],
        out: I64Local,
        function: &mut Function,
    ) {
        let numerator = self.runtime_schema().reserve_i64_local(function);
        let leap_years = self.runtime_schema().reserve_i64_local(function);
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(21));
        function.instruction(&Instruction::I64Add);
        (numerator).store(function);
        self.emit_temporal_persian_div_euclid(numerator, 33, leap_years, function);
        function.instruction(&Instruction::I64Const(PERSIAN_EPOCH_DAY - 2));
        (fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(365));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        (leap_years).load(function);
        function.instruction(&Instruction::I64Add);
        self.emit_temporal_persian_correction_i32(fields[0], true, function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Sub);
        self.emit_temporal_persian_month_offset(fields[1], function);
        function.instruction(&Instruction::I64Add);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Add);
        (out).store(function);
        self.runtime_schema()
            .release_i64_local(leap_years, function);
        self.runtime_schema().release_i64_local(numerator, function);
    }

    pub(super) fn emit_temporal_persian_to_iso(
        &mut self,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_persian_epoch_days(fields, epoch, function);
        self.emit_temporal_civil_from_days(epoch, fields[0], fields[1], fields[2], function);
        self.runtime_schema().release_i64_local(epoch, function);
    }

    /// Populate the parent's existing projection owner. Its common footer
    /// derives days_in_year from this same leap flag after the branch joins.
    pub(super) fn emit_temporal_persian_project_date(
        &mut self,
        iso: [I64Local; 3],
        result: &TemporalCalendarDateLocals,
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        let numerator = self.runtime_schema().reserve_i64_local(function);
        let one = self.runtime_schema().reserve_i64_local(function);
        let year_start = self.runtime_schema().reserve_i64_local(function);
        self.emit_temporal_plain_date_epoch_days(iso[0], iso[1], iso[2], epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(PERSIAN_EPOCH_DAY));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(33));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        (numerator).store(function);
        self.emit_temporal_persian_div_euclid(numerator, 12_053, result.fields[0], function);
        (result.fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.fields[0]).store(function);
        function.instruction(&Instruction::I64Const(1));
        (one).store(function);
        self.emit_temporal_persian_epoch_days([result.fields[0], one, one], year_start, function);
        (epoch).load(function);
        (year_start).load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.day_of_year).store(function);
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(366));
        function.instruction(&Instruction::I64Eq);
        self.emit_temporal_persian_correction_i32(result.fields[0], false, function);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        (result.fields[0]).load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        (result.fields[0]).store(function);
        function.instruction(&Instruction::I64Const(1));
        (result.day_of_year).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(186));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(31));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::Else);
        (result.day_of_year).load(function);
        function.instruction(&Instruction::I64Const(23));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64DivS);
        function.instruction(&Instruction::End);
        (result.fields[1]).store(function);
        (result.day_of_year).load(function);
        self.emit_temporal_persian_month_offset(result.fields[1], function);
        function.instruction(&Instruction::I64Sub);
        (result.fields[2]).store(function);
        self.emit_temporal_persian_leap_i32(result.fields[0], function);
        function.instruction(&Instruction::I64ExtendI32U);
        (result.leap).store(function);
        self.emit_temporal_persian_month_length(
            result.fields[1],
            result.leap,
            result.days_in_month,
            function,
        );
        for local in [year_start, one, numerator, epoch] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// All possible month/day pairs occur in leap 1350 or common 1351 before
    /// ISO 1972-12-31. Invalid 1351 M12D30 must select 1350 before conversion.
    pub(super) fn emit_temporal_persian_month_day_reference(
        &mut self,
        fields: [I64Local; 3],
        function: &mut Function,
    ) {
        let epoch = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::I64Const(1351));
        (fields[0]).store(function);
        (fields[1]).load(function);
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Eq);
        (fields[2]).load(function);
        function.instruction(&Instruction::I64Const(30));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1350));
        (fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_persian_epoch_days(fields, epoch, function);
        (epoch).load(function);
        function.instruction(&Instruction::I64Const(1095));
        function.instruction(&Instruction::I64GtS);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I64Const(1350));
        (fields[0]).store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_temporal_persian_to_iso(fields, function);
        self.runtime_schema().release_i64_local(epoch, function);
    }
}
