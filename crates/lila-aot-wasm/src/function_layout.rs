//! The one Wasm function index space.
//!
//! ```text
//! [imports][runtime callables][runtime helpers][main][program callables][program helpers]
//!           ^-- runtime half: identical for every program --^
//!                                                 ^-- program half starts at `main`
//! ```
//!
//! Runtime callables are the compiler-owned source bodies, every standard
//! builtin and every host builtin. Program callables are the script's own
//! functions followed by its prepared script units. Program helpers are the
//! helper bodies generated from the script (module operations and the runtime
//! hooks, see `HelperOwner`). Every index is derived
//! here, so the runtime/program boundary is [`Self::first_program_index`]
//! and no other module does position arithmetic.

use lila_ir::{FunctionIr, ScriptIr};

use crate::runtime_helpers::RuntimeHelperFunctionBase;

/// The script's source functions split at the runtime/program boundary. Each
/// list keeps the script's own order.
pub(crate) struct SourceFunctions<'a> {
    pub(crate) runtime_owned: Vec<&'a FunctionIr>,
    pub(crate) program: Vec<&'a FunctionIr>,
}

impl<'a> SourceFunctions<'a> {
    pub(crate) fn partition(script: &'a ScriptIr) -> Self {
        let (runtime_owned, program) = script
            .functions
            .iter()
            .partition(|function| crate::builtins::is_empty_dynamic_function_body(function));
        Self {
            runtime_owned,
            program,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FunctionIndexLayout {
    imported: u32,
    runtime_owned: u32,
    standard: u32,
    host: u32,
    runtime_helpers: u32,
    program_callables: u32,
    program_helpers: u32,
}

fn count(value: usize) -> u32 {
    u32::try_from(value).expect("function count fits Wasm index domain")
}

impl FunctionIndexLayout {
    pub(crate) fn new(
        imported: u32,
        runtime_owned: usize,
        standard: usize,
        host: usize,
        runtime_helpers: usize,
        program_callables: usize,
        program_helpers: usize,
    ) -> Self {
        let layout = Self {
            imported,
            runtime_owned: count(runtime_owned),
            standard: count(standard),
            host: count(host),
            runtime_helpers: count(runtime_helpers),
            program_callables: count(program_callables),
            program_helpers: count(program_helpers),
        };
        // Fails here, once, if the whole space does not fit.
        layout.program_helper_end();
        layout
    }

    fn add(base: u32, offset: u32) -> u32 {
        base.checked_add(offset)
            .expect("function index fits Wasm index domain")
    }

    pub(crate) fn runtime_owned_index(&self, position: usize) -> u32 {
        debug_assert!(position < self.runtime_owned as usize);
        Self::add(self.imported, count(position))
    }

    pub(crate) fn standard_index(&self, position: usize) -> u32 {
        debug_assert!(position < self.standard as usize);
        Self::add(
            Self::add(self.imported, self.runtime_owned),
            count(position),
        )
    }

    pub(crate) fn host_index(&self, position: usize) -> u32 {
        debug_assert!(position < self.host as usize);
        Self::add(
            Self::add(Self::add(self.imported, self.runtime_owned), self.standard),
            count(position),
        )
    }

    fn runtime_helper_first(&self) -> u32 {
        Self::add(
            Self::add(Self::add(self.imported, self.runtime_owned), self.standard),
            self.host,
        )
    }

    pub(crate) fn helper_base(&self) -> RuntimeHelperFunctionBase {
        RuntimeHelperFunctionBase::at(self.runtime_helper_first(), self.program_callable_end())
    }

    /// First index of the program half; `main` itself.
    pub(crate) fn first_program_index(&self) -> u32 {
        Self::add(self.runtime_helper_first(), self.runtime_helpers)
    }

    pub(crate) fn main_index(&self) -> u32 {
        self.first_program_index()
    }

    pub(crate) fn program_callable_index(&self, position: usize) -> u32 {
        debug_assert!(position < self.program_callables as usize);
        Self::add(self.main_index() + 1, count(position))
    }

    fn program_callable_end(&self) -> u32 {
        Self::add(self.main_index() + 1, self.program_callables)
    }

    fn program_helper_end(&self) -> u32 {
        Self::add(self.program_callable_end(), self.program_helpers)
    }

    /// Every defined function: what `ref.func` may name, for the declarative
    /// element segment.
    pub(crate) fn defined_functions(&self) -> std::ops::Range<u32> {
        self.imported..self.program_helper_end()
    }
}
