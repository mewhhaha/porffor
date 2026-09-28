//! Per-function attribution for the emitted code section.
//!
//! A Test262 failure that reads
//!
//! ```text
//! [origin:unknown] wasmtime module validation failed: Compilation error: Code for function is too large
//! ```
//!
//! names neither the function nor its size, and the compiler could not answer
//! either question: bodies were pushed straight into a bare
//! [`CodeSection`](wasm_encoder::CodeSection) as anonymous `Function` values.
//!
//! (The `[origin:unknown]` prefix is a separate thing — a `lila-test262`
//! `FailureOrigin`, classified from the detail text — and nothing here changes
//! it. Naming the function is the whole claim.)
//!
//! This module closes that gap by construction rather than by convention:
//!
//! * [`EmittedFunction`] is the only thing [`ModuleCode::push`] accepts, and it
//!   cannot be built without a [`FunctionIdentity`]. Its byte length is
//!   measured inside [`EmittedFunction::new`], the one place that owns the
//!   finished body, so no caller can compute or forget it.
//! * [`ModuleCode`] is the only path to a `CodeSection` in this crate.
//!   `emit.rs` no longer imports `CodeSection` at all, so emitting a body
//!   without recording who it belongs to is not expressible.
//! * [`FunctionIdentity::wasm_name`] is an exhaustive match with no `_` arm, so
//!   a new class of emitted function fails `cargo check` until it is named.
//!
//! The identity table is then spent twice. It becomes a Wasm custom `name`
//! section — wasmtime builds its per-function symbol as
//! `wasm[0]::function[N]::<name>` from exactly that section, so the same table
//! that measures a body also names it in wasmtime's own diagnostics — and it
//! feeds the `debug_dump` size report that `tests/emit_golden.rs` records for
//! all 527 CLI fixtures.

use std::num::NonZeroU32;

use lila_ir::{FunctionId, HostBuiltinId, StandardBuiltinId};
use wasm_encoder::{CodeSection, Encode, Instruction, NameMap, NameSection};

// The body handed to `EmittedFunction::new` is the label-counting sink, not the
// raw encoder type: `Function::into_body` is what asserts the body closed every
// frame it opened. See `code_sink.rs`.
use crate::code_sink::Function;
use crate::module::EmitError;
use crate::runtime_helpers::RuntimeHelperId;

/// Environment variable that turns on the full per-function size report in
/// `WasmArtifact::debug_dump`. Off by default: the report is one line per
/// emitted function and there are thousands of them in a bootstrap-heavy
/// module.
pub(crate) const EMIT_SIZE_REPORT_ENV: &str = "LILA_EMIT_SIZE_REPORT";

/// Environment variable naming a file the *full* per-function size report is
/// written to, one `emitted function: key=value ...` line per emitted body.
///
/// [`EMIT_SIZE_REPORT_ENV`] only appends the report to
/// `WasmArtifact::debug_dump`, and that string has exactly one printer in the
/// workspace (`lila-engine`, gated on `LILA_WASM_TRACE_DUMP`) which for
/// most of this compiler's life was unreachable from `lila build wasm` and from
/// the Test262 wasm-aot backend. A caller could therefore set the report
/// variable, see nothing, and read that as "no large functions" rather than as
/// "the dump was dropped". This variable is honoured **inside `emit()`**, so no
/// engine or CLI caller can drop it.
///
/// One thing it cannot see: the engine's program-Wasm cache. On a cache hit
/// `emit()` is never called, so the file still holds the report of the last
/// *emission*, not of the last run. Delete the file first if that distinction
/// matters.
pub(crate) const EMIT_SIZE_REPORT_PATH_ENV: &str = "LILA_EMIT_SIZE_REPORT_PATH";

/// Environment variable holding a per-function body-size budget in bytes.
///
/// Deliberately opt-in. A budget set below the largest body Cranelift accepts
/// today turns green tests red, and the maximum across the 527 CLI fixtures
/// plus a real-suite shard has not been measured yet. Once it has, this becomes
/// a calibrated constant and the check becomes unconditional.
pub(crate) const FUNCTION_BODY_BUDGET_ENV: &str = "LILA_EMIT_FUNCTION_BODY_BUDGET_BYTES";

/// Which source-level thing a code-section entry is the body of.
///
/// Closed set on purpose: every arm of [`Self::wasm_name`] and
/// [`Self::category`] is spelled out, so a new kind of emitted function is a
/// compile error rather than an unnamed function in the `name` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FunctionIdentity {
    /// The exported `main` body: script top level plus the runtime bootstrap.
    Main,
    /// A function declared by the user program.
    Script { id: FunctionId, name: String },
    /// A standard builtin compiled with its real body.
    StandardBuiltin(StandardBuiltinId),
    /// The single shared "this builtin was not emitted" stub body. The builtin
    /// it carries is only the representative the stub was built from.
    StandardBuiltinStub(StandardBuiltinId),
    /// A host builtin (`print`, the `$262.agent` surface, ...).
    HostBuiltin(HostBuiltinId),
    /// One of the shared runtime helpers.
    RuntimeHelper(RuntimeHelperId),
    /// A thin guard kept at the original function index.
    StackGuardWrapper(Box<FunctionIdentity>),
    /// The original body moved after the old defined-function space.
    GuardedBody(Box<FunctionIdentity>),
}

impl FunctionIdentity {
    /// The symbol written into the Wasm `name` section.
    ///
    /// Prefixed by category so a failure reading
    /// `wasm[0]::function[812]::builtin::Intl.DateTimeFormat.prototype.format`
    /// says both what kind of thing blew up and which one.
    pub(crate) fn wasm_name(&self) -> String {
        match self {
            Self::Main => "lila::main".to_string(),
            Self::Script { id, name } if name.is_empty() => format!("js::{id}"),
            Self::Script { id, name } => format!("js::{name}#{id}"),
            Self::StandardBuiltin(builtin) => format!("builtin::{}", builtin.debug_name()),
            // Deliberately not named after the representative builtin. One
            // body is shared by *every* stubbed builtin in the module, so
            // `builtin_stub::Math.max` would send a reader who saw it in
            // wasmtime's `wasm[0]::function[N]::…` symbol to look at `Math.max`
            // for a body `Math.max` has nothing to do with. The representative
            // stays in `report_lines`, where there is room to say so.
            Self::StandardBuiltinStub(_) => "builtin_stub::shared".to_string(),
            Self::HostBuiltin(builtin) => format!("host::{}", builtin.as_str()),
            Self::RuntimeHelper(helper) => format!("helper::{}", helper.debug_name()),
            Self::StackGuardWrapper(target) => {
                format!("stack_guard::wrapper::{}", target.wasm_name())
            }
            // The relocated body remains the source-level implementation for
            // size reports and native failure diagnostics. The thin wrapper
            // has its own explicit `stack_guard::wrapper::` symbol.
            Self::GuardedBody(target) => target.wasm_name(),
        }
    }

    /// Coarse bucket used by the size report so a reader can tell at a glance
    /// whether the biggest body is user code, a builtin, or a helper.
    pub(crate) fn category(&self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Script { .. } => "script",
            Self::StandardBuiltin(_) => "builtin",
            Self::StandardBuiltinStub(_) => "builtin-stub",
            Self::HostBuiltin(_) => "host-builtin",
            Self::RuntimeHelper(_) => "runtime-helper",
            Self::StackGuardWrapper(_) => "stack-guard-wrapper",
            Self::GuardedBody(target) => target.category(),
        }
    }

    pub(crate) fn is_runtime_helper(&self) -> bool {
        match self {
            Self::RuntimeHelper(_) => true,
            Self::GuardedBody(target) => target.is_runtime_helper(),
            Self::Main
            | Self::Script { .. }
            | Self::StandardBuiltin(_)
            | Self::StandardBuiltinStub(_)
            | Self::HostBuiltin(_)
            | Self::StackGuardWrapper(_) => false,
        }
    }
}

/// The encoded length of one function body, in bytes, excluding the
/// variable-width size prefix the code section writes in front of it.
///
/// A newtype rather than a bare `u32` so a size cannot be swapped with a
/// function index, a budget, or a local count at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionBodySize(u32);

impl FunctionBodySize {
    /// Measures a finished body. This is the only constructor, and it is called
    /// from [`EmittedFunction::new`], so a recorded size is always the size of
    /// the body recorded beside it.
    fn of(raw_body: &[u8]) -> Self {
        Self(u32::try_from(raw_body.len()).unwrap_or(u32::MAX))
    }

    pub const fn bytes(self) -> u32 {
        self.0
    }
}

/// The number of locals a body declares.
///
/// Recorded next to the byte size because it, not the byte size, is what the
/// failure this module exists for is actually about. Cranelift raises
/// `CodeTooLarge` when `VRegAllocator::alloc` exhausts the 2,097,151 virtual
/// registers `VReg::MAX_BITS = 21` permits, and the Wasm frontend materialises
/// a value per live local at every control-flow join — so virtual-register
/// pressure grows with `locals x blocks`, which a byte count alone can hide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionLocalCount(u32);

impl FunctionLocalCount {
    /// Every value type this decoder will step over, as its single-byte
    /// encoding.
    ///
    /// The list is a *validation* set, not documentation. Wasm's GC and typed
    /// reference types (`(ref null $t)` = `0x63`, `(ref $t)` = `0x64`) encode as
    /// more than one byte, so stepping over exactly one byte for such a type
    /// would leave the cursor mid-heap-type and the loop would then read
    /// instruction bytes as group counts — producing a wrong but entirely
    /// plausible local count. Rejecting anything not listed here is what makes
    /// [`Self::decode`]'s "cannot decode" answer honest.
    const SINGLE_BYTE_VALUE_TYPES: [u8; 7] = [
        0x7F, // i32
        0x7E, // i64
        0x7D, // f32
        0x7C, // f64
        0x7B, // v128
        0x70, // funcref
        0x6F, // externref
    ];

    /// Decodes the local declaration that prefixes an encoded function body:
    /// a LEB128 group count, then that many `(LEB128 count, value type)` pairs.
    ///
    /// `None` means "this body's local declaration did not decode", which the
    /// report prints as `unknown` rather than as a number. Every body here is
    /// produced by `wasm_encoder`, so `None` means the encoder started emitting
    /// a value type shape this does not know — and the declared-local count is
    /// the figure that predicts `CodeTooLarge`, so a wrong-but-plausible value
    /// is worse than an absent one.
    fn decode(raw_body: &[u8]) -> Option<Self> {
        fn read_u32(bytes: &[u8], cursor: &mut usize) -> Option<u32> {
            let mut value: u32 = 0;
            let mut shift = 0;
            loop {
                let byte = *bytes.get(*cursor)?;
                *cursor += 1;
                value |= u32::from(byte & 0x7f).checked_shl(shift)?;
                if byte & 0x80 == 0 {
                    return Some(value);
                }
                shift += 7;
                if shift >= 32 {
                    return None;
                }
            }
        }

        let mut cursor = 0usize;
        let groups = read_u32(raw_body, &mut cursor)?;
        let mut total: u32 = 0;
        for _ in 0..groups {
            let count = read_u32(raw_body, &mut cursor)?;
            let value_type = *raw_body.get(cursor)?;
            if !Self::SINGLE_BYTE_VALUE_TYPES.contains(&value_type) {
                return None;
            }
            cursor += 1;
            total = total.checked_add(count)?;
        }
        Some(Self(total))
    }

    pub const fn count(self) -> u32 {
        self.0
    }
}

/// Renders a possibly-undecodable local count for the size report.
///
/// `unknown` rather than `0`, so "the decoder does not understand this body"
/// cannot be mistaken for "this body declares no locals".
fn format_declared_locals(locals: Option<FunctionLocalCount>) -> String {
    match locals {
        Some(locals) => locals.count().to_string(),
        None => "unknown".to_string(),
    }
}

/// One emitted function, as the size report sees it.
///
/// This is the typed form of a `report_lines` row, and it is the *only* form:
/// `WasmArtifact::function_sizes`, the `largest emitted function:` line, the
/// `most locals in an emitted function:` line and the full opt-in report are
/// all rendered from one [`ModuleFunctionTable::summaries`] call, so they
/// cannot disagree with each other. The precedent for insisting on that is the
/// `runtime helper functions: 27` literal in `debug_dump`, which had drifted to
/// a counted truth of 32 + 1 precisely because it was a second copy.
///
/// [`Self::category`] is `&'static str` sourced from
/// [`FunctionIdentity::category`], an exhaustive match with no `_` arm, so a new
/// class of emitted function fails to build until it is named — the report
/// cannot silently acquire an `other` bucket.
///
/// The two size figures keep the newtypes [`EmittedFunctionRecord`] stores them
/// in rather than being flattened to `u32` on the way out. This row exists to
/// carry *two* numbers that are both "a count about one body" and that must
/// never be confused — the whole point of reporting `most_locals` separately
/// from `largest` is that they answer different questions — so
/// `EmittedFunctionSummary { body_bytes: locals, declared_locals: bytes }` has
/// to be a type error rather than a plausible transcription slip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedFunctionSummary {
    pub wasm_index: u32,
    pub name: String,
    pub category: &'static str,
    pub body_bytes: FunctionBodySize,
    /// `None` when the body's local declaration did not decode; see
    /// [`FunctionLocalCount::decode`]. Rendered as `unknown`, never as `0`.
    pub declared_locals: Option<FunctionLocalCount>,
}

impl EmittedFunctionSummary {
    /// The `key=value` field list shared by every line of the size report.
    ///
    /// `name=` is **last** on purpose: emitted names contain spaces
    /// (`get Object.prototype.__proto__`, `Array Iterator.prototype.next`), so
    /// a positional layout could not be parsed back. `tests/emit_golden.rs`
    /// parses exactly these keys out of `debug_dump`.
    fn fields(&self) -> String {
        format!(
            "index={} bytes={} locals={} kind={} name={}",
            self.wasm_index,
            self.body_bytes.bytes(),
            format_declared_locals(self.declared_locals),
            self.category,
            self.name
        )
    }

    /// The largest body among `summaries`.
    ///
    /// Iterates in code-section order and keeps the *last* maximum, which is
    /// what `Iterator::max_by_key` does; the ordering is load-bearing only in
    /// that it must not change, because `debug_dump` is a golden artifact
    /// (rung G) and a re-ordered tie would show up as a spurious diff.
    pub(crate) fn largest(summaries: &[Self]) -> Option<&Self> {
        summaries.iter().max_by_key(|summary| summary.body_bytes)
    }

    /// The body declaring the most locals.
    ///
    /// Reported separately from [`Self::largest`] because they are often
    /// different functions and it is this one that predicts Cranelift's
    /// `CodeTooLarge`: the Wasm frontend materialises a value per live local at
    /// every control-flow join, so virtual-register pressure tracks
    /// `locals x blocks` rather than encoded size.
    ///
    /// `None` sorts below every `Some`, so a body whose declaration did not
    /// decode can never be reported as the worst offender on the strength of a
    /// number nobody knows.
    pub(crate) fn most_locals(summaries: &[Self]) -> Option<&Self> {
        summaries
            .iter()
            .max_by_key(|summary| summary.declared_locals)
    }

    /// One line per function, largest first, for the opt-in size report.
    pub(crate) fn report_lines(summaries: &[Self], limit: usize) -> Vec<String> {
        let mut ranked = summaries.iter().collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .body_bytes
                .cmp(&left.body_bytes)
                .then(left.wasm_index.cmp(&right.wasm_index))
        });
        ranked
            .into_iter()
            .take(limit)
            .map(|summary| format!("emitted function: {}", summary.fields()))
            .collect()
    }

    /// The `largest emitted function:` / `most locals in an emitted function:`
    /// line body, or the `none` fallback when nothing was emitted.
    pub(crate) fn attribution_line(prefix: &str, summary: Option<&Self>) -> String {
        match summary {
            Some(summary) => format!("{prefix}: {}", summary.fields()),
            None => format!("{prefix}: none"),
        }
    }
}

impl core::fmt::Display for FunctionBodySize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}

/// A finished function body together with the identity it was compiled for and
/// its measured size.
///
/// There is no way to build one without an identity, and no way to get a body
/// into the code section without building one.
#[derive(Debug, Clone)]
pub(crate) struct EmittedFunction {
    identity: FunctionIdentity,
    /// The body in the form `code_sink::Function::into_body().into_raw_body()`
    /// produces: the local
    /// declaration followed by the instruction bytes, with no length prefix.
    /// `CodeSection::raw` prepends the same length prefix `CodeSection::function`
    /// would, so this is byte-identical to pushing the `Function` itself, and it
    /// lets the local declaration be read back without cloning megabytes.
    raw_body: Vec<u8>,
    body_bytes: FunctionBodySize,
    declared_locals: Option<FunctionLocalCount>,
}

impl EmittedFunction {
    pub(crate) fn new(identity: FunctionIdentity, body: Function) -> Self {
        // `into_body_named` asserts the label stack is empty, so an emitter
        // that opened a frame and never closed it fails here — with the
        // identity in the panic message, not merely in scope — instead of as
        // an anonymous wasmtime validation error inside whichever Test262 case
        // happened to compile it.
        let raw_body = body.into_body_named(&identity.wasm_name()).into_raw_body();
        let body_bytes = FunctionBodySize::of(&raw_body);
        let declared_locals = FunctionLocalCount::decode(&raw_body);
        Self {
            identity,
            raw_body,
            body_bytes,
            declared_locals,
        }
    }

    pub(crate) fn identity(&self) -> &FunctionIdentity {
        &self.identity
    }

    /// Re-labels an already-finished body without decoding or re-encoding it.
    /// The raw bytes and their measurements stay paired while relocation gives
    /// the body a distinct entry in the Wasm `name` section.
    pub(crate) fn with_identity(mut self, identity: FunctionIdentity) -> Self {
        self.identity = identity;
        self
    }

    /// Preserve the caller's active Realm around ordinary Wasm calls while
    /// leaving it untouched across tail calls. Guard wrappers install the
    /// callee Realm and tail-call this body. Capture the active Realm at each
    /// call site: a body may temporarily enter another Realm (for example,
    /// while draining Promise jobs), so its entry Realm is not always the
    /// active Realm to restore when a nested call returns.
    pub(crate) fn with_active_realm_restoration(
        mut self,
        parameter_count: u32,
        active_realm_global_index: u32,
    ) -> Result<Self, String> {
        let body = wasmparser::FunctionBody::new(wasmparser::BinaryReader::new(&self.raw_body, 0));
        let mut locals = body
            .get_locals_reader()
            .map_err(|error| format!("cannot read relocated function locals: {error}"))?;
        let local_group_count = locals.get_count();
        let mut declared_local_count = 0u32;
        for _ in 0..local_group_count {
            let (count, _) = locals
                .read()
                .map_err(|error| format!("cannot read relocated function locals: {error}"))?;
            declared_local_count = declared_local_count
                .checked_add(count)
                .ok_or_else(|| "relocated function local count overflows u32".to_string())?;
        }
        let operators_start = locals.original_position();
        let realm_local_index = parameter_count
            .checked_add(declared_local_count)
            .ok_or_else(|| "active Realm local index overflows u32".to_string())?;
        let transformed_local_count = declared_local_count
            .checked_add(1)
            .ok_or_else(|| "relocated function local count overflows u32".to_string())?;
        let expanded_group_count = local_group_count
            .checked_add(1)
            .ok_or_else(|| "relocated function local group count overflows u32".to_string())?;

        let mut operators = body
            .get_operators_reader()
            .map_err(|error| format!("cannot read relocated function operators: {error}"))?;
        let mut call_ranges = Vec::new();
        let mut saw_final_end = false;
        while !operators.eof() {
            let (operator, start) = operators
                .read_with_offset()
                .map_err(|error| format!("cannot read relocated function operators: {error}"))?;
            let end = operators.original_position();
            match operator {
                wasmparser::Operator::Call { .. }
                | wasmparser::Operator::CallIndirect { .. }
                | wasmparser::Operator::CallRef { .. } => call_ranges.push((start, end)),
                wasmparser::Operator::End if operators.eof() => saw_final_end = true,
                _ => {}
            }
        }
        if !saw_final_end || operators_start > self.raw_body.len() {
            return Err("relocated function body has no final end operator".to_string());
        }
        if call_ranges.is_empty() {
            return Ok(self);
        }

        let (old_group_count, group_count_prefix_len) = read_var_u32(&self.raw_body)?;
        if old_group_count != local_group_count {
            return Err("relocated function local count changed while decoding".to_string());
        }

        let instrumentation_bytes = call_ranges
            .len()
            .checked_mul(24)
            .and_then(|bytes| bytes.checked_add(16))
            .ok_or_else(|| "relocated function instrumentation size overflows usize".to_string())?;
        let mut transformed = Vec::with_capacity(
            self.raw_body
                .len()
                .checked_add(instrumentation_bytes)
                .ok_or_else(|| "relocated function body size overflows usize".to_string())?,
        );
        write_var_u32(expanded_group_count, &mut transformed);
        transformed.extend_from_slice(&self.raw_body[group_count_prefix_len..operators_start]);
        transformed.push(1); // one new local declaration group
        transformed.push(0x7e); // i64

        let mut copied_through = operators_start;
        for (call_start, call_end) in call_ranges {
            if call_start < copied_through
                || call_end < call_start
                || call_end > self.raw_body.len()
            {
                return Err("relocated function operator offsets are not ordered".to_string());
            }
            transformed.extend_from_slice(&self.raw_body[copied_through..call_start]);
            Instruction::GlobalGet(active_realm_global_index).encode(&mut transformed);
            Instruction::LocalSet(realm_local_index).encode(&mut transformed);
            transformed.extend_from_slice(&self.raw_body[call_start..call_end]);
            Instruction::LocalGet(realm_local_index).encode(&mut transformed);
            Instruction::GlobalSet(active_realm_global_index).encode(&mut transformed);
            copied_through = call_end;
        }
        transformed.extend_from_slice(&self.raw_body[copied_through..]);
        self.raw_body = transformed;
        self.body_bytes = FunctionBodySize::of(&self.raw_body);
        self.declared_locals = Some(FunctionLocalCount(transformed_local_count));
        Ok(self)
    }
}

fn read_var_u32(bytes: &[u8]) -> Result<(u32, usize), String> {
    let mut reader = wasmparser::BinaryReader::new(bytes, 0);
    let value = reader
        .read_var_u32()
        .map_err(|error| format!("invalid local declaration count: {error}"))?;
    Ok((value, reader.original_position()))
}

fn write_var_u32(mut value: u32, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// One row of the emitted-function table.
#[derive(Debug, Clone)]
pub(crate) struct EmittedFunctionRecord {
    pub(crate) wasm_index: u32,
    pub(crate) identity: FunctionIdentity,
    pub(crate) body_bytes: FunctionBodySize,
    /// `None` when the body's local declaration did not decode; see
    /// [`FunctionLocalCount::decode`].
    pub(crate) declared_locals: Option<FunctionLocalCount>,
}

/// The code section under construction, plus the attribution table.
///
/// `CodeSection` is private to this type. `emit.rs` cannot name it, so the
/// "push a body and forget to record it" state is unrepresentable rather than
/// something review has to notice.
pub(crate) struct ModuleCode {
    functions: Vec<EmittedFunction>,
    first_wasm_index: u32,
    next_wasm_index: u32,
}

impl ModuleCode {
    /// `first_wasm_index` is the Wasm function index the first pushed body will
    /// occupy, i.e. the imported function count (imports come first in the
    /// index space).
    pub(crate) fn new(first_wasm_index: u32) -> Self {
        Self {
            functions: Vec::new(),
            first_wasm_index,
            next_wasm_index: first_wasm_index,
        }
    }

    pub(crate) fn push(&mut self, function: EmittedFunction) {
        self.next_wasm_index = self
            .next_wasm_index
            .checked_add(1)
            .expect("emitted Wasm function index space overflow");
        self.functions.push(function);
    }

    /// Preserve main's original index for its entry wrapper, then move the
    /// compiled body to the end of code-section order. The caller has already
    /// appended all other original functions and guarded JS bodies, so the
    /// resulting index is the metadata's startup-body index.
    pub(crate) fn wrap_main_and_relocate(
        &mut self,
        wrapper: EmittedFunction,
        active_realm_global_index: u32,
    ) -> Result<u32, String> {
        let main = std::mem::replace(
            self.functions
                .first_mut()
                .expect("compiled module code starts with main"),
            wrapper,
        );
        assert!(matches!(main.identity(), FunctionIdentity::Main));
        let main = main
            .with_identity(FunctionIdentity::GuardedBody(Box::new(
                FunctionIdentity::Main,
            )))
            .with_active_realm_restoration(0, active_realm_global_index)?;
        let local_count = main
            .declared_locals
            .ok_or_else(|| "could not count relocated main-body locals".to_string())?
            .count();
        self.push(main);
        Ok(local_count)
    }

    pub(crate) fn finish(self) -> (CodeSection, ModuleFunctionTable) {
        let mut section = CodeSection::new();
        let mut records = Vec::with_capacity(self.functions.len());
        let first_wasm_index = self.first_wasm_index;
        for (offset, function) in self.functions.into_iter().enumerate() {
            let wasm_index = first_wasm_index
                .checked_add(u32::try_from(offset).expect("function count fits u32"))
                .expect("emitted Wasm function index space overflow");
            section.raw(&function.raw_body);
            records.push(EmittedFunctionRecord {
                wasm_index,
                identity: function.identity,
                body_bytes: function.body_bytes,
                declared_locals: function.declared_locals,
            });
        }
        (section, ModuleFunctionTable { records })
    }
}

/// Every emitted function, in code-section order, with its identity and size.
#[derive(Debug, Clone)]
pub(crate) struct ModuleFunctionTable {
    records: Vec<EmittedFunctionRecord>,
}

impl ModuleFunctionTable {
    pub(crate) fn records(&self) -> &[EmittedFunctionRecord] {
        &self.records
    }

    /// The Wasm custom `name` section for this module.
    ///
    /// Indices are appended in ascending order, which the encoding requires;
    /// `ModuleCode::push` hands out indices monotonically, so iteration order
    /// is already ascending.
    pub(crate) fn name_section(&self) -> NameSection {
        let mut names = NameSection::new();
        names.module("lila");
        let mut functions = NameMap::new();
        for record in &self.records {
            functions.append(record.wasm_index, &record.identity.wasm_name());
        }
        names.functions(&functions);
        names
    }

    /// The one traversal.
    ///
    /// Every size figure the compiler reports about itself — the typed
    /// `WasmArtifact::function_sizes`, the two `debug_dump` attribution lines
    /// and the opt-in full report — is derived from the slice this returns.
    /// There is deliberately no second accessor that walks `records` for the
    /// same question: two traversals is how the `runtime helper functions: 27`
    /// literal came to disagree with the code section by five.
    ///
    /// Order is code-section order, not size order. `report_lines` sorts a
    /// borrowed view when it needs a ranking, so the stored order stays the one
    /// that makes `wasm_index` monotonic.
    pub(crate) fn summaries(&self) -> Vec<EmittedFunctionSummary> {
        self.records
            .iter()
            .map(|record| EmittedFunctionSummary {
                wasm_index: record.wasm_index,
                name: record.identity.wasm_name(),
                category: record.identity.category(),
                body_bytes: record.body_bytes,
                declared_locals: record.declared_locals,
            })
            .collect()
    }

    pub(crate) fn total_body_bytes(&self) -> u64 {
        self.records
            .iter()
            .map(|record| u64::from(record.body_bytes.bytes()))
            .sum()
    }

    /// Number of runtime-helper bodies actually written, derived from the same
    /// table the bodies were pushed into. The `debug_dump` line this replaces
    /// was a hand-maintained literal `27` against a counted truth of 32 + 1.
    pub(crate) fn runtime_helper_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.identity.is_runtime_helper())
            .count()
    }

    /// Fails the emission when any body exceeds `budget`.
    ///
    /// Reports the *largest* offender rather than the first, so a single run
    /// names the function worth attacking.
    pub(crate) fn check_budget(&self, budget: FunctionBodyBudget) -> Result<(), EmitError> {
        let Some(worst) = self
            .records
            .iter()
            .filter(|record| budget.is_exceeded_by(record.body_bytes))
            .max_by_key(|record| record.body_bytes)
        else {
            return Ok(());
        };
        Err(EmitError::function_too_large(
            &worst.identity,
            worst.body_bytes,
            budget,
        ))
    }
}

/// A per-function body-size ceiling, in bytes.
///
/// Built once from a validated value and passed as this type thereafter, so
/// there is no bare `u32` threshold to thread through call sites and no way to
/// express a zero budget that would fail every module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FunctionBodyBudget(NonZeroU32);

impl FunctionBodyBudget {
    pub(crate) const fn from_bytes(bytes: NonZeroU32) -> Self {
        Self(bytes)
    }

    /// Reads the budget from [`FUNCTION_BODY_BUDGET_ENV`]. A missing, empty,
    /// unparseable or zero value means "no budget", which is the default: this
    /// lane lands the check in report-only form until the maximum body size
    /// across the fixture corpus has been measured.
    pub(crate) fn from_env() -> Option<Self> {
        let raw = std::env::var(FUNCTION_BODY_BUDGET_ENV).ok()?;
        let bytes = raw.trim().parse::<u32>().ok()?;
        NonZeroU32::new(bytes).map(Self::from_bytes)
    }

    pub(crate) const fn bytes(self) -> u32 {
        self.0.get()
    }

    const fn is_exceeded_by(self, size: FunctionBodySize) -> bool {
        size.bytes() > self.0.get()
    }
}

impl core::fmt::Display for FunctionBodyBudget {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} bytes", self.bytes())
    }
}

/// Whether the opt-in full size report is requested.
pub(crate) fn emit_size_report_requested() -> bool {
    std::env::var_os(EMIT_SIZE_REPORT_ENV).is_some_and(|value| !value.is_empty())
}

/// Writes the full per-function size report to the file named by
/// [`EMIT_SIZE_REPORT_PATH_ENV`], if that variable is set to a non-empty value.
///
/// Called from `emit()` itself rather than from any caller, which is the whole
/// point: the `debug_dump` route has one printer several layers up and has been
/// silently dropped on the two paths that matter most (`lila build wasm` and the
/// Test262 wasm-aot backend). A failing write panics rather than being ignored —
/// an empty or absent report file is exactly the ambiguity this exists to
/// remove.
///
/// The env read is the *only* thing this wrapper does; the report text and the
/// write are [`write_size_report_file`], which is what
/// `the_size_report_file_is_the_same_traversal_as_the_typed_report` exercises.
/// The split exists because a test that sets `EMIT_SIZE_REPORT_PATH_ENV` would
/// be visible to every other test in the process, and the sink is the one part
/// of this module that was previously protected by review alone.
///
/// One hazard this deliberately does not hide: the variable names a single
/// path, so N concurrent compilations (the Test262 wasm-aot runner's workers,
/// for instance) all write the same file and the survivor is whichever finished
/// last. Give each worker its own path, or read the file as "some module's
/// report", never "this module's report".
pub(crate) fn write_size_report_file_if_requested(summaries: &[EmittedFunctionSummary]) {
    let Some(path) = std::env::var_os(EMIT_SIZE_REPORT_PATH_ENV) else {
        return;
    };
    if path.is_empty() {
        return;
    }
    write_size_report_file(path.as_ref(), summaries);
}

/// Renders and writes the report. Separated from the env read so it is directly
/// testable; see [`write_size_report_file_if_requested`].
pub(crate) fn write_size_report_file(path: &std::path::Path, summaries: &[EmittedFunctionSummary]) {
    let mut report = EmittedFunctionSummary::report_lines(summaries, usize::MAX).join("\n");
    report.push('\n');
    std::fs::write(path, report).unwrap_or_else(|err| {
        panic!("failed to write the {EMIT_SIZE_REPORT_PATH_ENV} report to {path:?}: {err}")
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{Instruction, ValType};

    fn body(instructions: usize) -> Function {
        let mut function = Function::new([(0, ValType::I64)]);
        for _ in 0..instructions {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::End);
        function
    }

    #[test]
    fn recorded_size_is_the_size_of_the_recorded_body() {
        let small = body(1);
        let expected = small.byte_len() as u32;
        let emitted = EmittedFunction::new(FunctionIdentity::Main, small);
        assert_eq!(emitted.body_bytes.bytes(), expected);
        assert_eq!(emitted.identity, FunctionIdentity::Main);
    }

    #[test]
    fn indices_follow_the_imported_function_count() {
        let mut code = ModuleCode::new(3);
        code.push(EmittedFunction::new(FunctionIdentity::Main, body(1)));
        code.push(EmittedFunction::new(
            FunctionIdentity::RuntimeHelper(RuntimeHelperId::HeapAlloc),
            body(2),
        ));
        let (_, table) = code.finish();
        let indices = table
            .records()
            .iter()
            .map(|record| record.wasm_index)
            .collect::<Vec<_>>();
        assert_eq!(indices, vec![3, 4]);
        assert_eq!(table.runtime_helper_count(), 1);
    }

    #[test]
    fn largest_is_the_largest_body() {
        let mut code = ModuleCode::new(0);
        code.push(EmittedFunction::new(FunctionIdentity::Main, body(1)));
        code.push(EmittedFunction::new(
            FunctionIdentity::RuntimeHelper(RuntimeHelperId::ObjectRead),
            body(64),
        ));
        let (_, table) = code.finish();
        let summaries = table.summaries();
        let largest =
            EmittedFunctionSummary::largest(&summaries).expect("table should not be empty");
        assert_eq!(largest.name, "helper::object_read");
        assert_eq!(largest.category, "runtime-helper");
        assert!(table.total_body_bytes() >= u64::from(largest.body_bytes.bytes()));
    }

    /// The typed report and the rendered report are the same traversal, so a
    /// row can never appear in one and not the other, and the `largest`
    /// attribution line can never name a function the table does not hold.
    #[test]
    fn the_report_and_the_typed_summaries_are_one_traversal() {
        let mut code = ModuleCode::new(0);
        code.push(EmittedFunction::new(FunctionIdentity::Main, body(1)));
        code.push(EmittedFunction::new(
            FunctionIdentity::RuntimeHelper(RuntimeHelperId::ObjectRead),
            body(64),
        ));
        code.push(EmittedFunction::new(
            FunctionIdentity::HostBuiltin(HostBuiltinId::Print),
            body(8),
        ));
        let (_, table) = code.finish();
        let summaries = table.summaries();
        assert_eq!(summaries.len(), table.records().len());
        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.wasm_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2],
            "summaries stay in code-section order"
        );

        let lines = EmittedFunctionSummary::report_lines(&summaries, usize::MAX);
        assert_eq!(lines.len(), summaries.len());
        let largest =
            EmittedFunctionSummary::largest(&summaries).expect("table should not be empty");
        assert!(
            lines[0].contains(&format!("name={}", largest.name)),
            "the first report row and `largest` must be the same function: {lines:?}"
        );
        assert_eq!(
            EmittedFunctionSummary::attribution_line("largest emitted function", Some(largest)),
            format!(
                "largest emitted function: {}",
                &lines[0]["emitted function: ".len()..]
            ),
        );
        assert_eq!(
            EmittedFunctionSummary::attribution_line("largest emitted function", None),
            "largest emitted function: none"
        );
    }

    #[test]
    fn budget_failure_names_the_function() {
        let mut code = ModuleCode::new(0);
        code.push(EmittedFunction::new(FunctionIdentity::Main, body(256)));
        let (_, table) = code.finish();
        let budget = FunctionBodyBudget::from_bytes(NonZeroU32::new(8).expect("nonzero"));
        let error = table
            .check_budget(budget)
            .expect_err("a 256-instruction body should exceed an 8 byte budget");
        let message = error.to_string();
        assert!(message.contains("lila::main"), "{message}");
        assert!(message.contains('8'), "{message}");
    }

    #[test]
    fn a_body_at_the_budget_is_accepted() {
        let mut code = ModuleCode::new(0);
        let single = body(1);
        let size = single.byte_len() as u32;
        code.push(EmittedFunction::new(FunctionIdentity::Main, single));
        let (_, table) = code.finish();
        let budget = FunctionBodyBudget::from_bytes(NonZeroU32::new(size).expect("nonzero"));
        assert!(table.check_budget(budget).is_ok());
    }

    #[test]
    fn declared_locals_are_read_back_from_the_body() {
        let mut function = Function::new([(7, ValType::I64), (3, ValType::I32)]);
        function.instruction(&Instruction::End);
        let emitted = EmittedFunction::new(FunctionIdentity::Main, function);
        assert_eq!(
            emitted
                .declared_locals
                .expect("an i64/i32 declaration must decode")
                .count(),
            10
        );
    }

    #[test]
    fn a_body_with_no_locals_declares_none() {
        let mut function = Function::new([]);
        function.instruction(&Instruction::End);
        let emitted = EmittedFunction::new(FunctionIdentity::Main, function);
        assert_eq!(
            emitted
                .declared_locals
                .expect("an empty declaration must decode")
                .count(),
            0
        );
    }

    #[test]
    fn realm_restoration_preserves_leaf_and_tail_only_bodies() {
        for tail_call in [false, true] {
            let mut function = Function::new([]);
            if tail_call {
                function.instruction(&Instruction::ReturnCall(6));
            }
            function.instruction(&Instruction::End);
            let emitted = EmittedFunction::new(FunctionIdentity::Main, function);
            let original = emitted.raw_body.clone();
            let transformed = emitted
                .with_active_realm_restoration(0, 17)
                .expect("a body without ordinary calls needs no Realm capture");
            assert_eq!(transformed.raw_body, original);
            assert_eq!(transformed.declared_locals.unwrap().count(), 0);
        }
    }

    #[test]
    fn active_realm_restore_follows_calls_but_not_tail_calls() {
        let mut function = Function::new([]);
        function.instruction(&Instruction::Call(5));
        function.instruction(&Instruction::I64Const(99));
        function.instruction(&Instruction::GlobalSet(17));
        function.instruction(&Instruction::Call(7));
        function.instruction(&Instruction::ReturnCall(6));
        function.instruction(&Instruction::End);
        let transformed = EmittedFunction::new(FunctionIdentity::Main, function)
            .with_active_realm_restoration(0, 17)
            .expect("Realm instrumentation should preserve the body");
        assert_eq!(
            transformed
                .declared_locals
                .expect("the capture local is measurable")
                .count(),
            1
        );

        let body =
            wasmparser::FunctionBody::new(wasmparser::BinaryReader::new(&transformed.raw_body, 0));
        let operators = body
            .get_operators_reader()
            .expect("instrumented body should parse")
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("every instrumented operator should parse");
        assert!(matches!(
            operators.as_slice(),
            [
                wasmparser::Operator::GlobalGet { global_index: 17 },
                wasmparser::Operator::LocalSet { local_index: 0 },
                wasmparser::Operator::Call { function_index: 5 },
                wasmparser::Operator::LocalGet { local_index: 0 },
                wasmparser::Operator::GlobalSet { global_index: 17 },
                wasmparser::Operator::I64Const { value: 99 },
                wasmparser::Operator::GlobalSet { global_index: 17 },
                wasmparser::Operator::GlobalGet { global_index: 17 },
                wasmparser::Operator::LocalSet { local_index: 0 },
                wasmparser::Operator::Call { function_index: 7 },
                wasmparser::Operator::LocalGet { local_index: 0 },
                wasmparser::Operator::GlobalSet { global_index: 17 },
                wasmparser::Operator::ReturnCall { function_index: 6 },
                wasmparser::Operator::End,
            ]
        ));
    }

    /// The `None` answer must be reachable, or `format_declared_locals`'s
    /// `unknown` arm is dead and `decode`'s validation set is decoration.
    #[test]
    fn an_undecodable_value_type_declines_to_guess() {
        // One group of one local whose value type is `(ref null $0)` (`0x63`),
        // a two-byte encoding this decoder deliberately refuses to step over.
        let raw_body = [0x01u8, 0x01, 0x63, 0x00, 0x0b];
        assert_eq!(FunctionLocalCount::decode(&raw_body), None);
        assert_eq!(format_declared_locals(None), "unknown");
    }

    #[test]
    fn every_identity_kind_produces_a_distinct_name() {
        let names = [
            FunctionIdentity::Main,
            FunctionIdentity::Script {
                id: "fn0".to_string(),
                name: "outer".to_string(),
            },
            FunctionIdentity::StandardBuiltin(StandardBuiltinId::MathMax),
            FunctionIdentity::StandardBuiltinStub(StandardBuiltinId::MathMax),
            FunctionIdentity::HostBuiltin(HostBuiltinId::Print),
            FunctionIdentity::RuntimeHelper(RuntimeHelperId::HeapAlloc),
        ]
        .iter()
        .map(FunctionIdentity::wasm_name)
        .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), 6);
    }

    #[test]
    fn an_anonymous_script_function_still_gets_a_name() {
        let identity = FunctionIdentity::Script {
            id: "fn7".to_string(),
            name: String::new(),
        };
        assert_eq!(identity.wasm_name(), "js::fn7");
    }
}
