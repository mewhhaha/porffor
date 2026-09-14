use lila_ir::{
    ArithmeticBinaryOp, BitwiseBinaryOp, EqualityBinaryOp, ExprIr, RelationalBinaryOp,
    SpecOperationIr, TypedExpr, UnaryBitwiseOp,
};

// Exhausting this proof budget retains the ordinary runtime branch. It does
// not reject the program or change the compiler's expression-size limits.
const PROOF_DEPTH: usize = 128;

/// Only immutable, effect-free IR operands can select a branch here. Static
/// value-kind hints cannot establish that an identifier or property is stable.
pub(super) fn truthiness(condition: &TypedExpr) -> Option<bool> {
    condition_truthiness(condition, PROOF_DEPTH)
}

fn condition_truthiness(condition: &TypedExpr, depth: usize) -> Option<bool> {
    let depth = depth.checked_sub(1)?;
    match &condition.expr {
        ExprIr::Boolean(value) => Some(*value),
        ExprIr::LogicalNot { expr } => Some(!condition_truthiness(expr, depth)?),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::StrictEqualityComparison | SpecOperationIr::IsLooselyEqual,
            operands,
        } => {
            let [lhs, rhs] = operands.as_slice() else {
                return None;
            };
            Some(number(lhs, depth)? == number(rhs, depth)?)
        }
        ExprIr::StrictEquality { op, lhs, rhs } | ExprIr::LooseEquality { op, lhs, rhs } => {
            let lhs = number(lhs, depth)?;
            let rhs = number(rhs, depth)?;
            Some(match op {
                EqualityBinaryOp::StrictEqual | EqualityBinaryOp::LooseEqual => lhs == rhs,
                EqualityBinaryOp::StrictNotEqual | EqualityBinaryOp::LooseNotEqual => lhs != rhs,
            })
        }
        ExprIr::CompareNumber { op, lhs, rhs } | ExprIr::CompareValue { op, lhs, rhs } => {
            let lhs = number(lhs, depth)?;
            let rhs = number(rhs, depth)?;
            Some(match op {
                RelationalBinaryOp::LessThan => lhs < rhs,
                RelationalBinaryOp::LessThanOrEqual => lhs <= rhs,
                RelationalBinaryOp::GreaterThan => lhs > rhs,
                RelationalBinaryOp::GreaterThanOrEqual => lhs >= rhs,
            })
        }
        _ => {
            let value = number(condition, depth)?;
            Some(value != 0.0 && !value.is_nan())
        }
    }
}

fn number(expression: &TypedExpr, depth: usize) -> Option<f64> {
    let depth = depth.checked_sub(1)?;
    match &expression.expr {
        ExprIr::Number(bits) => Some(f64::from_bits(*bits)),
        ExprIr::UnaryPlus { expr } => number(expr, depth),
        ExprIr::UnaryMinusNumeric { expr } => Some(-number(expr, depth)?),
        ExprIr::UnaryBitwiseNumeric { op, expr } => {
            let value = to_uint32(number(expr, depth)?);
            Some(match op {
                UnaryBitwiseOp::Complement => (!value as i32) as f64,
            })
        }
        ExprIr::BinaryNumber { op, lhs, rhs } | ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {
            let lhs = number(lhs, depth)?;
            let rhs = number(rhs, depth)?;
            Some(match op {
                ArithmeticBinaryOp::Add => lhs + rhs,
                ArithmeticBinaryOp::Sub => lhs - rhs,
                ArithmeticBinaryOp::Mul => lhs * rhs,
                ArithmeticBinaryOp::Div => lhs / rhs,
                ArithmeticBinaryOp::Mod => lhs % rhs,
                // Number exponentiation has additional ECMAScript cases and
                // implementation-approximated results; it is outside this proof.
                ArithmeticBinaryOp::Exp => return None,
            })
        }
        ExprIr::CoerciveAdd { lhs, rhs } => Some(number(lhs, depth)? + number(rhs, depth)?),
        ExprIr::BitwiseNumeric { op, lhs, rhs } => {
            let lhs = to_uint32(number(lhs, depth)?);
            let rhs = to_uint32(number(rhs, depth)?);
            Some(match op {
                BitwiseBinaryOp::And => (lhs & rhs) as i32 as f64,
                BitwiseBinaryOp::Or => (lhs | rhs) as i32 as f64,
                BitwiseBinaryOp::Xor => (lhs ^ rhs) as i32 as f64,
                BitwiseBinaryOp::Shl => lhs.wrapping_shl(rhs & 31) as i32 as f64,
                BitwiseBinaryOp::Shr => ((lhs as i32) >> (rhs & 31)) as f64,
                BitwiseBinaryOp::UShr => (lhs >> (rhs & 31)) as f64,
            })
        }
        // In particular, never strip comma effects or follow mutable bindings,
        // property reads, calls, conversion operations, or BigInt operands.
        _ => None,
    }
}

fn to_uint32(number: f64) -> u32 {
    let bits = number.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    // Magnitudes below one truncate to zero. At exponent >= 84, every
    // finite binary64 integer is a multiple of 2^32; infinities and NaNs
    // also map to zero. The remaining shifts are in the range 0..=52.
    if !(0..84).contains(&exponent) {
        return 0;
    }
    let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    let residue = if exponent >= 52 {
        (significand as u32).wrapping_shl((exponent - 52) as u32)
    } else {
        (significand >> (52 - exponent)) as u32
    };
    if bits >> 63 != 0 {
        residue.wrapping_neg()
    } else {
        residue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};
    use lila_ir::{lower, StatementIr, ValueInfo, ValueKind};

    fn lowered_condition(source: &str) -> TypedExpr {
        let source = format!("function sample(value, Number, Math, NaN, Infinity) {{ if ({source}) return 1; else return 2; }}");
        let parsed = parse(&source, ParseOptions::script()).expect("numeric guard parses");
        let lowered = lower(&parsed);
        assert!(lowered.is_wasm_supported(), "{:?}", lowered.diagnostics);
        let function = lowered
            .script
            .expect("script IR")
            .functions
            .into_iter()
            .find(|function| function.name == "sample")
            .expect("sample function");
        let StatementIr::If { condition, .. } =
            function.body.statements.into_iter().next().expect("guard")
        else {
            panic!("branch selection belongs to emission, after lowering");
        };
        condition
    }

    #[test]
    fn immutable_number_guards_use_ecmascript_numeric_semantics() {
        for (source, expected) in [
            ("true", true),
            ("false", false),
            ("1 << 0 !== 1", false),
            ("+(1 << 1) === 2", true),
            ("-2147483649 >> 33 === 1073741823", true),
            ("-1 >>> 0 === 4294967295", true),
            ("4294967297 << 33 === 2", true),
            ("7.9 << 1.9 === 14", true),
            ("1 << -1 === -2147483648", true),
            ("9007199254740991 | 0", true),
            ("18446744073709551616 | 0", false),
            ("~4294967295 === 0", true),
            ("(7 & 3) ^ 1", true),
            ("(0 / 0) !== (0 / 0)", true),
            ("0 / 0 >= 1", false),
            ("0 / 0 < 1", false),
            ("-0 === 0", true),
            ("1 / -0 < 0", true),
            ("-0", false),
            ("0 / 0", false),
            ("(1 / 0) >>> 0", false),
            ("(-5 % 2) === -1", true),
            ("1 / (-4 % 2) < 0", true),
            ("(2 * 3 - 1) == 5", true),
            ("(2 + 3) != 5", false),
            ("!(1 << 0 !== 1)", true),
        ] {
            assert_eq!(
                truthiness(&lowered_condition(source)),
                Some(expected),
                "{source}"
            );
        }
    }

    #[test]
    fn mutable_effectful_and_bigint_operands_remain_runtime_conditions() {
        for source in [
            "value << 0 !== 1",
            "Number.EPSILON === 1",
            "Math.PI === 1",
            "NaN !== NaN",
            "Infinity > 1",
            "value() === 1",
            "(value = 1, 1) === 1",
            "(value(), 1) === 1",
            "value.x === 1",
            "(1n << 1n) === 2n",
            "(1n >>> 0n) === 0n",
            "+1n === 1",
            "(1n << 1) === 2n",
        ] {
            assert_eq!(truthiness(&lowered_condition(source)), None, "{source}");
        }
        let mutable_number = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Identifier("mutable".into()),
        );
        assert_eq!(truthiness(&mutable_number), None);
        let exponentiation = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::BinaryNumber {
                op: ArithmeticBinaryOp::Exp,
                lhs: Box::new(TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Number),
                    ExprIr::Number(2.0_f64.to_bits()),
                )),
                rhs: Box::new(TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Number),
                    ExprIr::Number(3.0_f64.to_bits()),
                )),
            },
        );
        assert_eq!(truthiness(&exponentiation), None);
    }

    #[test]
    fn number_to_uint32_preserves_truncation_and_all_binary64_exponents() {
        for (number, expected) in [
            (0.0, 0),
            (-0.0, 0),
            (f64::NAN, 0),
            (f64::INFINITY, 0),
            (f64::NEG_INFINITY, 0),
            (f64::MIN_POSITIVE, 0),
            (f64::MAX, 0),
            (0.999, 0),
            (-0.999, 0),
            (1.9, 1),
            (-1.9, u32::MAX),
            (4_294_967_297.0, 1),
            (-4_294_967_297.0, u32::MAX),
            (9_007_199_254_740_991.0, u32::MAX),
            (9_223_372_036_854_777_856.0, 2048),
            (-9_223_372_036_854_777_856.0, 4_294_965_248),
        ] {
            assert_eq!(to_uint32(number), expected, "{number:?}");
        }
        for exponent in 0..=1023 {
            let power = f64::from_bits(((exponent + 1023) as u64) << 52);
            let expected = if exponent < 32 { 1_u32 << exponent } else { 0 };
            assert_eq!(to_uint32(power), expected, "2^{exponent}");
            assert_eq!(to_uint32(-power), expected.wrapping_neg(), "-2^{exponent}");
        }
    }

    #[test]
    fn exhausting_the_proof_budget_keeps_the_runtime_expression() {
        let mut expression = TypedExpr::from_info(
            ValueInfo::new(ValueKind::Number),
            ExprIr::Number(1.0_f64.to_bits()),
        );
        for _ in 0..PROOF_DEPTH {
            expression = TypedExpr::from_info(
                ValueInfo::new(ValueKind::Number),
                ExprIr::UnaryPlus {
                    expr: Box::new(expression),
                },
            );
        }
        assert_eq!(truthiness(&expression), None);
    }
}
