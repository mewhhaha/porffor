//! Complete function-body admission with an explicitly bounded worker count.
use std::num::NonZeroUsize;
use std::sync::Mutex;
use wasmparser::{FuncToValidate, FuncValidatorAllocations, FunctionBody, ValidatorResources};

type Function<'a> = (FuncToValidate<ValidatorResources>, FunctionBody<'a>);

/// External bytes always require body admission. Only the unsafe immutable
/// build-package boundary can reuse full validation by the Cargo producer.
#[derive(Clone, Copy)]
pub(super) enum FunctionBodyValidation {
    AtUse,
    ValidatedAtBuild,
}

fn bounded_jobs(requested: NonZeroUsize, functions: usize) -> usize {
    requested
        .get()
        .min(std::thread::available_parallelism().map_or(1, NonZeroUsize::get))
        .min(functions)
}

pub(super) fn validate_all(functions: Vec<Function<'_>>, requested: NonZeroUsize) -> Option<()> {
    let jobs = bounded_jobs(requested, functions.len());
    let trace = std::env::var_os("LILA_WASM_TRACE").is_some();
    let count = functions.len();
    let started = std::time::Instant::now();
    let result = validate_batch(functions, jobs);
    if trace {
        eprintln!(
            "lila wasm trace: runtime-function validation: {count} bodies / {jobs} jobs in {:?}",
            started.elapsed()
        );
    }
    result
}

fn validate_pending(work: &Mutex<Vec<Function<'_>>>) -> Option<()> {
    let mut allocations = FuncValidatorAllocations::default();
    loop {
        let next = work.lock().ok()?.pop();
        let Some((function, body)) = next else {
            return Some(());
        };
        let mut validator = function.into_validator(allocations);
        validator.validate(&body).ok()?;
        allocations = validator.into_allocations();
    }
}

fn validate_batch(functions: Vec<Function<'_>>, jobs: usize) -> Option<()> {
    if jobs <= 1 {
        let mut allocations = FuncValidatorAllocations::default();
        for (function, body) in functions {
            let mut validator = function.into_validator(allocations);
            validator.validate(&body).ok()?;
            allocations = validator.into_allocations();
        }
        return Some(());
    }
    let work = Mutex::new(functions);
    std::thread::scope(|scope| {
        let mut threads = Vec::with_capacity(jobs);
        let mut accepted = true;
        for index in 0..jobs {
            let work = &work;
            match std::thread::Builder::new()
                .name(format!("lila-runtime-validation-{index}"))
                .stack_size(64 * 1024 * 1024)
                .spawn_scoped(scope, move || validate_pending(work))
            {
                Ok(thread) => threads.push(thread),
                Err(_) => {
                    // A transient thread limit reduces concurrency, not checks.
                    // Calling thread plus spawned workers still fit `jobs`.
                    accepted &= validate_pending(work).is_some();
                    break;
                }
            }
        }
        // Join every spawned worker, including after another worker rejects.
        for thread in threads {
            accepted &= matches!(thread.join(), Ok(Some(())));
        }
        accepted.then_some(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{
        CodeSection, Function as EncodedFunction, FunctionSection, Instruction, Module,
        TypeSection, ValType,
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
        assert!(
            bounded_jobs(request, 8) <= std::thread::available_parallelism().unwrap().get().min(8)
        );
        assert_eq!(bounded_jobs(NonZeroUsize::MIN, 8), 1);
    }
}
