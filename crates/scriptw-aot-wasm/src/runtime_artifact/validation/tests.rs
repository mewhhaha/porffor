#![cfg(test)]
use super::*;
use wasm_encoder::{
    CodeSection, Function as EncodedFunction, FunctionSection, Instruction, Module, TypeSection,
    ValType,
};
use wasmparser::{Parser, ValidPayload, Validator};

fn module(invalid: bool) -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);
    let mut functions = FunctionSection::new();
    let mut code = CodeSection::new();
    for index in 0..8 {
        functions.function(0);
        let mut function = EncodedFunction::new([]);
        function.instruction(if invalid && index == 7 {
            &Instruction::I64Const(1)
        } else {
            &Instruction::I32Const(1)
        });
        function.instruction(&Instruction::End);
        code.function(&function);
    }
    module.section(&functions);
    module.section(&code);
    module.finish()
}

fn bodies(bytes: &[u8]) -> Vec<Function<'_>> {
    let mut validator = Validator::new();
    Parser::new(0)
        .parse_all(bytes)
        .filter_map(
            |payload| match validator.payload(&payload.unwrap()).unwrap() {
                ValidPayload::Func(function, body) => Some((function, body)),
                _ => None,
            },
        )
        .collect()
}

#[test]
fn every_body_is_checked_in_serial_and_parallel_batches() {
    for jobs in [1, 2, 4] {
        assert!(validate_batch(bodies(&module(false)), jobs).is_some());
        assert!(validate_batch(bodies(&module(true)), jobs).is_none());
    }
}

#[test]
fn requested_threads_are_bounded_by_capacity_and_work() {
    let request = NonZeroUsize::new(usize::MAX).unwrap();
    assert_eq!(bounded_jobs(request, 0), 0);
    assert_eq!(bounded_jobs(request, 1), 1);
    assert!(bounded_jobs(request, 8) <= std::thread::available_parallelism().unwrap().get().min(8));
    assert_eq!(bounded_jobs(NonZeroUsize::MIN, 8), 1);
}
