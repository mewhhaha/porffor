use lila_aot_wasm::emit;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;

fn bitwise_body_bytes(expression: String, repetitions: usize) -> u32 {
    std::thread::Builder::new()
        .name(format!("numeric-bitwise-{repetitions}"))
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let statements = format!("{expression};\n").repeat(repetitions);
            let source = format!("function shifts() {{ {statements} return 1; }} shifts();");
            let parsed = parse(&source, ParseOptions::script()).expect("bitwise fixture parses");
            let artifact = emit(&lower(&parsed)).expect("bitwise fixture emits real Wasm");
            artifact
                .function_sizes
                .iter()
                .filter(|body| body.name.starts_with("js::shifts#"))
                .map(|body| body.body_bytes.bytes())
                .max()
                .expect("the shifts function must be emitted")
        })
        .expect("compiler worker starts")
        .join()
        .expect("bitwise emission does not panic")
}

#[test]
fn static_number_bitwise_sites_have_bounded_incremental_emission() {
    for operator in ["<<", ">>", ">>>", "&", "|", "^"] {
        let single = bitwise_body_bytes(format!("2147483649 {operator} 33"), 1);
        let repeated = bitwise_body_bytes(format!("2147483649 {operator} 33"), 65);
        let growth = repeated
            .checked_sub(single)
            .expect("additional Number operations must remain in the emitted body");
        assert!(
            growth > 0,
            "{operator}: the repeated operations disappeared"
        );
        assert!(
            growth < 64 * 512,
            "{operator}: 64 Number operations added {growth} bytes ({single} -> {repeated}); \
             their site cost must not include generic conversion and BigInt dispatch"
        );
    }
}

#[test]
fn unary_and_nested_number_sites_do_not_restore_generic_numeric_dispatch() {
    for expression in [
        "-2147483649",
        "~2147483649",
        "-2147483649 << -1",
        "~2147483649 >> 33",
        "(2147483649 << 1) | ~7",
        "(-1 >>> 1) ^ 3",
    ] {
        let single = bitwise_body_bytes(expression.to_string(), 1);
        let repeated = bitwise_body_bytes(expression.to_string(), 65);
        let growth = repeated
            .checked_sub(single)
            .expect("additional operations remain emitted");
        assert!(growth > 0 && growth < 64 * 768,
            "{expression}: 64 static Number expressions added {growth} bytes ({single} -> {repeated})");
    }
}
