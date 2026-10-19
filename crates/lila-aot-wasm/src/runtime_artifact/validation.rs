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
mod tests;
