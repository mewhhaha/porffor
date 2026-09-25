use super::*;

/* origin: FreeBSD /usr/src/lib/msun/src/e_log10.c */
/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */
// The reduction and polynomial below are adapted from libm 0.2.16's fdlibm
// log10 implementation. Every operation is emitted into the user's Wasm module.

const IVLN10HI: f64 = 4.34294481878168880939e-01;
const IVLN10LO: f64 = 2.50829467116452752298e-11;
const LOG10_2HI: f64 = 3.01029995663611771306e-01;
const LOG10_2LO: f64 = 3.69423907715893078616e-13;
const LG1: f64 = 6.666666666666735130e-01;
const LG2: f64 = 3.999999999940941908e-01;
const LG3: f64 = 2.857142874366239149e-01;
const LG4: f64 = 2.222219843214978396e-01;
const LG5: f64 = 1.818357216161805012e-01;
const LG6: f64 = 1.531383769920937332e-01;
const LG7: f64 = 1.479819860511658591e-01;

// All temporary locals contain the i64 bit pattern of an f64. This private
// compiler expression tree emits exactly one f64 for every node, so an
// arithmetic formula cannot underflow or leave operands on the Wasm stack.
enum FloatExpr {
    Local(u32),
    Constant(f64),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
}

impl FloatExpr {
    fn emit(&self, function: &mut Function) {
        match self {
            Self::Local(local) => {
                function.instruction(&Instruction::LocalGet(*local));
                function.instruction(&Instruction::F64ReinterpretI64);
            }
            Self::Constant(value) => {
                function.instruction(&Instruction::F64Const(Ieee64::from(*value)));
            }
            Self::Add(left, right) => {
                left.emit(function);
                right.emit(function);
                function.instruction(&Instruction::F64Add);
            }
            Self::Sub(left, right) => {
                left.emit(function);
                right.emit(function);
                function.instruction(&Instruction::F64Sub);
            }
            Self::Mul(left, right) => {
                left.emit(function);
                right.emit(function);
                function.instruction(&Instruction::F64Mul);
            }
            Self::Div(left, right) => {
                left.emit(function);
                right.emit(function);
                function.instruction(&Instruction::F64Div);
            }
        }
    }
}

macro_rules! impl_float_binary_op {
    ($trait:ident, $method:ident, $variant:ident) => {
        impl std::ops::$trait for FloatExpr {
            type Output = Self;

            fn $method(self, right: Self) -> Self {
                Self::$variant(Box::new(self), Box::new(right))
            }
        }
    };
}

impl_float_binary_op!(Add, add, Add);
impl_float_binary_op!(Sub, sub, Sub);
impl_float_binary_op!(Mul, mul, Mul);
impl_float_binary_op!(Div, div, Div);

fn emit_formula(target: u32, expr: FloatExpr, function: &mut Function) {
    expr.emit(function);
    function.instruction(&Instruction::I64ReinterpretF64);
    function.instruction(&Instruction::LocalSet(target));
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_math_log10_number_payload(
        &mut self,
        input_local: u32,
        result_local: u32,
        function: &mut Function,
    ) {
        use FloatExpr::{Constant as C, Local as L};

        let x = self.reserve_temp_local();
        let hx = self.reserve_temp_local();
        let k = self.reserve_temp_local();
        let f = self.reserve_temp_local();
        let hfsq = self.reserve_temp_local();
        let s = self.reserve_temp_local();
        let z = self.reserve_temp_local();
        let w = self.reserve_temp_local();
        let t1 = self.reserve_temp_local();
        let t2 = self.reserve_temp_local();
        let r = self.reserve_temp_local();
        let hi = self.reserve_temp_local();
        let lo = self.reserve_temp_local();
        let val_hi = self.reserve_temp_local();
        let val_lo = self.reserve_temp_local();
        let dk = self.reserve_temp_local();
        let y = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(input_local));
        function.instruction(&Instruction::LocalSet(x));
        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(hx));

        // Both signed zeros have an all-zero magnitude.
        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::I64Const(0x7fff_ffff_ffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::NEG_INFINITY.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(f64::NAN.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(hx));
        function.instruction(&Instruction::I64Const(0x7ff0_0000));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Positive infinity and positive NaN retain their input value.
        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::I64Const(1.0f64.to_bits() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0.0f64.to_bits() as i64));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(k));
        function.instruction(&Instruction::LocalGet(hx));
        function.instruction(&Instruction::I64Const(0x0010_0000));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Scale positive subnormals by 2^54 before extracting the exponent.
        function.instruction(&Instruction::I64Const(-54));
        function.instruction(&Instruction::LocalSet(k));
        emit_formula(x, L(x) * C(f64::from_bits(0x4350_0000_0000_0000)), function);
        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(hx));
        function.instruction(&Instruction::End);

        // Reduce x into [sqrt(2)/2, sqrt(2)] and keep the exponent in k.
        function.instruction(&Instruction::LocalGet(hx));
        function.instruction(&Instruction::I64Const(0x3ff0_0000 - 0x3fe6_a09e));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(hx));
        function.instruction(&Instruction::LocalGet(k));
        function.instruction(&Instruction::LocalGet(hx));
        function.instruction(&Instruction::I64Const(20));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(0x3ff));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(k));
        function.instruction(&Instruction::LocalGet(hx));
        function.instruction(&Instruction::I64Const(0x000f_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x3fe6_a09e));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(x));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(x));

        emit_formula(f, L(x) - C(1.0), function);
        emit_formula(hfsq, (C(0.5) * L(f)) * L(f), function);
        emit_formula(s, L(f) / (C(2.0) + L(f)), function);
        emit_formula(z, L(s) * L(s), function);
        emit_formula(w, L(z) * L(z), function);
        emit_formula(
            t1,
            L(w) * (C(LG2) + L(w) * (C(LG4) + L(w) * C(LG6))),
            function,
        );
        emit_formula(
            t2,
            L(z) * (C(LG1) + L(w) * (C(LG3) + L(w) * (C(LG5) + L(w) * C(LG7)))),
            function,
        );
        emit_formula(r, L(t2) + L(t1), function);

        // hi + lo ~= log(1+f). Clear the low 32 bits of hi as in fdlibm.
        emit_formula(hi, L(f) - L(hfsq), function);
        function.instruction(&Instruction::LocalGet(hi));
        function.instruction(&Instruction::I64Const(0xffff_ffff_0000_0000u64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(hi));
        emit_formula(
            lo,
            ((L(f) - L(hi)) - L(hfsq)) + L(s) * (L(hfsq) + L(r)),
            function,
        );

        emit_formula(val_hi, L(hi) * C(IVLN10HI), function);
        function.instruction(&Instruction::LocalGet(k));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::F64ConvertI32S);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(dk));
        emit_formula(y, L(dk) * C(LOG10_2HI), function);
        emit_formula(
            val_lo,
            ((L(dk) * C(LOG10_2LO)) + ((L(lo) + L(hi)) * C(IVLN10LO))) + (L(lo) * C(IVLN10HI)),
            function,
        );

        // Compensated addition of k*log10(2) to log10(1+f).
        emit_formula(w, L(y) + L(val_hi), function);
        emit_formula(val_lo, L(val_lo) + ((L(y) - L(w)) + L(val_hi)), function);
        emit_formula(result_local, L(val_lo) + L(w), function);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            y, dk, val_lo, val_hi, lo, hi, r, t2, t1, w, z, s, hfsq, f, k, hx, x,
        ] {
            self.release_temp_local(local);
        }
    }
}
