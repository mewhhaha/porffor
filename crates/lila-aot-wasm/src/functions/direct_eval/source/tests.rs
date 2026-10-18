use super::*;
use lila_ir::{encode_js_string_utf16, ValueInfo, ValueKind};

fn literal(source: &str) -> TypedExpr {
    TypedExpr::from_info(
        ValueInfo::new(ValueKind::String),
        ExprIr::String(source.into()),
    )
}

#[test]
fn literal_selection_uses_exact_utf16_including_surrogate_transport_aliases() {
    let units = vec![0x27, 0xd83d, 0xdc38, 0x27, 0x3b];
    let canonical = encode_js_string_utf16(&units);
    let split = format!(
        "'{}{}';",
        encode_js_string_utf16(&[0xd83d]),
        encode_js_string_utf16(&[0xdc38])
    );
    assert_ne!(canonical, split);
    for source in [&canonical, &split] {
        let hint = DirectEvalSource::of(&[literal(source)]);
        assert!(hint.admits(&canonical));
        assert!(hint.admits(&split));
        assert!(!hint.admits("'\\uD83D\\uDC38';"));
        let marker = "'\u{F0000}D83D';".encode_utf16().collect::<Vec<_>>();
        assert!(!hint.admits(&encode_js_string_utf16(&marker)));
    }
    let lone_high = encode_js_string_utf16(&[0xd83d]);
    let lone_low = encode_js_string_utf16(&[0xdc38]);
    let hint = DirectEvalSource::of(&[literal(&lone_high)]);
    assert!(hint.admits(&lone_high));
    assert!(!hint.admits(&lone_low));
}

#[test]
fn missing_and_nonliteral_first_arguments_retain_every_prepared_source() {
    let nonliteral = TypedExpr::from_info(
        ValueInfo::new(ValueKind::String),
        ExprIr::Conditional {
            condition: Box::new(TypedExpr::undefined()),
            then_expr: Box::new(literal("first")),
            else_expr: Box::new(literal("second")),
        },
    );
    for arguments in [
        vec![],
        vec![TypedExpr::undefined()],
        vec![TypedExpr::undefined(), literal("later")],
        vec![nonliteral],
    ] {
        let hint = DirectEvalSource::of(&arguments);
        for candidate in ["first", "second", "later", "throw new Error('unrelated');"] {
            assert!(hint.admits(candidate));
        }
    }
}
