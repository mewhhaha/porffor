//! The only path from an emitter to a Wasm function body.
//!
//! Every raw structured-control instruction contributes to the real label
//! stack, including frames that the JS control-flow builder does not manage.
//! A branch target records both its stack position and its identity: a closed
//! frame must not become a valid target again when a sibling reuses its depth.
//!
//! These checks are unconditional, including in release builds used for
//! conformance runs. They validate emission structure, not Wasm operand types;
//! the Wasm validator remains responsible for the latter.

use std::sync::atomic::{AtomicU64, Ordering};

use wasm_encoder::{Catch, Instruction, ValType};

/// A live label's position and identity, not a relative branch immediate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LabelDepth {
    depth: u32,
    identity: u64,
}

impl LabelDepth {
    /// Synthetic positions for control-target tests, never emission handles.
    #[cfg(test)]
    pub(crate) const fn for_test(depth: u32) -> Self {
        Self { depth, identity: 0 }
    }
}

/// Constructed only after checking that the target label is still live.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BranchDepth(u32);

impl BranchDepth {
    const fn immediate(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameKind {
    Function,
    Block,
    Loop,
    IfThen,
    IfElse,
    TryTable,
    LegacyTry,
    LegacyCatch,
    LegacyCatchAll,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Frame {
    identity: u64,
    kind: FrameKind,
}

// Identities never enter the encoded module. Global allocation also rejects
// foreign-function handles and labels opened independently after cloning a
// partially emitted body. A clone intentionally retains its live prefix.
static NEXT_LABEL_IDENTITY: AtomicU64 = AtomicU64::new(1);

impl Frame {
    fn new(kind: FrameKind) -> Self {
        let identity = NEXT_LABEL_IDENTITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .expect("Wasm label identity space exhausted");
        Self { identity, kind }
    }
}

/// The exact ordered local declaration consumed by one function body.
///
/// The body owns this declaration from construction onward. Typed allocation
/// appends exact scalar or reference types, and reuse accepts only the same
/// declared type; publication emits that complete declaration.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LocalDeclarations {
    runs: Vec<(u32, ValType)>,
    count: u32,
}

impl LocalDeclarations {
    pub(crate) fn from_types<L>(locals: L) -> Self
    where
        L: IntoIterator<Item = ValType>,
    {
        let mut runs: Vec<(u32, ValType)> = Vec::new();
        let mut count = 0_u32;
        for local_type in locals {
            count = count.checked_add(1).expect("Wasm local count exceeds u32");
            if let Some((run_count, run_type)) = runs.last_mut() {
                if *run_type == local_type {
                    *run_count += 1;
                    continue;
                }
            }
            runs.push((1, local_type));
        }
        Self { runs, count }
    }

    /// Preserve deliberately ungrouped declarations used by decoder fixtures.
    #[cfg(test)]
    pub(crate) fn from_runs<L>(locals: L) -> Self
    where
        L: IntoIterator<Item = (u32, ValType)>,
    {
        let runs: Vec<_> = locals.into_iter().collect();
        let count = runs.iter().fold(0_u32, |count, (run_count, _)| {
            count
                .checked_add(*run_count)
                .expect("Wasm local count exceeds u32")
        });
        Self { runs, count }
    }

    fn push(&mut self, ty: ValType) {
        self.count = self
            .count
            .checked_add(1)
            .expect("Wasm local count exceeds u32");
        if let Some((count, last)) = self.runs.last_mut() {
            if *last == ty {
                *count += 1;
                return;
            }
        }
        self.runs.push((1, ty));
    }

    fn type_at(&self, index: u32) -> Option<ValType> {
        let mut offset = index;
        for (count, ty) in &self.runs {
            if offset < *count {
                return Some(*ty);
            }
            offset -= count;
        }
        None
    }

    fn encode_body(&self) -> wasm_encoder::Function {
        wasm_encoder::Function::new(self.runs.iter().copied())
    }

    /// Only cloning a complete body may fork its private declaration plan.
    fn fork_for_body(&self) -> Self {
        Self {
            runs: self.runs.clone(),
            count: self.count,
        }
    }
}

/// A function body and its open frames, including the implicit function label.
///
/// Re-exported as `Function` from the crate root so all emitters use the same
/// accounting. An empty frame stack means the final `end` has been emitted.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Function {
    body: wasm_encoder::Function,
    locals: LocalDeclarations,
    frames: Vec<Frame>,
    parameters: Vec<ValType>,
    initial_local_count: u32,
    initial_prefix_len: usize,
    free_locals: Vec<u32>,
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            body: self.body.clone(),
            locals: self.locals.fork_for_body(),
            frames: self.frames.clone(),
            parameters: self.parameters.clone(),
            initial_local_count: self.initial_local_count,
            initial_prefix_len: self.initial_prefix_len,
            free_locals: self.free_locals.clone(),
        }
    }
}

impl Function {
    pub(crate) fn new(locals: LocalDeclarations) -> Self {
        Self::with_parameters(locals, Vec::new())
    }

    pub(crate) fn with_parameters(locals: LocalDeclarations, parameters: Vec<ValType>) -> Self {
        let initial_local_count = locals.count;
        let body = locals.encode_body();
        let initial_prefix_len = body.byte_len();
        Self {
            body,
            locals,
            frames: vec![Frame::new(FrameKind::Function)],
            parameters,
            initial_local_count,
            initial_prefix_len,
            free_locals: Vec::new(),
        }
    }

    pub(crate) fn parameter_type(&self, index: u32) -> Option<ValType> {
        self.parameters.get(index as usize).copied()
    }
    pub(crate) fn parameter_types(&self) -> &[ValType] {
        &self.parameters
    }
    pub(crate) fn allocated_local_count(&self) -> u32 {
        self.locals.count
    }

    pub(crate) fn reserve_typed_local(&mut self, ty: ValType) -> u32 {
        assert!(
            !self.frames.is_empty(),
            "cannot reserve a local after function publication"
        );
        if let Some(position) = self.free_locals.iter().position(|index| {
            self.locals.type_at(*index - self.parameters.len() as u32) == Some(ty)
        }) {
            return self.free_locals.swap_remove(position);
        }
        let index = u32::try_from(self.parameters.len())
            .expect("parameter count overflow")
            .checked_add(self.locals.count)
            .expect("local index overflow");
        self.locals.push(ty);
        index
    }

    pub(crate) fn release_typed_local(&mut self, index: u32) {
        let base = u32::try_from(self.parameters.len()).expect("parameter count overflow");
        assert!(
            index >= base + self.initial_local_count && index < base + self.locals.count,
            "only this body's dynamically reserved locals can be released"
        );
        assert!(!self.free_locals.contains(&index), "local released twice");
        self.free_locals.push(index);
    }

    fn depth(&self) -> u32 {
        u32::try_from(self.frames.len()).expect("Wasm label depth exceeds u32")
    }

    fn check_branch(&self, label: u32) {
        assert!(
            label < self.depth(),
            "branch immediate {label} is out of range at label depth {}",
            self.depth()
        );
    }

    /// Account for an instruction before appending its bytes.
    pub(crate) fn instruction(&mut self, instruction: &Instruction<'_>) -> &mut Self {
        if self.frames.is_empty() {
            if matches!(instruction, Instruction::End) {
                panic!(
                    "wasm `end` with no open label: the emitter closed more frames than it opened"
                );
            }
            panic!("instruction emitted after the function body's final end");
        }

        match instruction {
            Instruction::Block(_) => self.frames.push(Frame::new(FrameKind::Block)),
            Instruction::Loop(_) => self.frames.push(Frame::new(FrameKind::Loop)),
            Instruction::If(_) => self.frames.push(Frame::new(FrameKind::IfThen)),
            Instruction::TryTable(_, catches) => {
                // Handler labels name enclosing frames, not the new try_table
                // frame. Check EVERY clause before changing either the frame
                // stack or encoded bytes, including clauses after catch_all.
                for catch in catches.iter() {
                    let label = match catch {
                        Catch::One { label, .. }
                        | Catch::OneRef { label, .. }
                        | Catch::All { label }
                        | Catch::AllRef { label } => *label,
                    };
                    self.check_branch(label);
                }
                self.frames.push(Frame::new(FrameKind::TryTable));
            }
            Instruction::Try(_) => self.frames.push(Frame::new(FrameKind::LegacyTry)),
            Instruction::Catch(_) | Instruction::CatchAll => {
                let frame = self
                    .frames
                    .last_mut()
                    .expect("an open frame was checked above");
                assert!(
                    matches!(frame.kind, FrameKind::LegacyTry | FrameKind::LegacyCatch),
                    "wasm `catch`/`catch_all` must belong to an open legacy try before catch_all"
                );
                // A handler replaces the arm, not its label identity. Multiple
                // tagged catches are legal; catch_all is necessarily last.
                frame.kind = if matches!(instruction, Instruction::CatchAll) {
                    FrameKind::LegacyCatchAll
                } else {
                    FrameKind::LegacyCatch
                };
            }
            Instruction::Delegate(label) => {
                assert_eq!(
                    self.frames
                        .last()
                        .expect("an open frame was checked above")
                        .kind,
                    FrameKind::LegacyTry,
                    "wasm `delegate` must terminate a legacy try body before any catch"
                );
                // Delegate substitutes for end. Its immediate is relative to
                // the enclosing stack AFTER removing this try. Ordinary block
                // and function labels are legal delegation destinations too.
                let enclosing_depth = self.depth() - 1;
                assert!(
                    *label < enclosing_depth,
                    "delegate immediate {label} is out of range at enclosing label depth {enclosing_depth}"
                );
                self.frames.pop();
            }
            Instruction::Rethrow(label) => {
                self.check_branch(*label);
                let frame = &self.frames[self.frames.len() - 1 - *label as usize];
                assert!(
                    matches!(
                        frame.kind,
                        FrameKind::LegacyCatch | FrameKind::LegacyCatchAll
                    ),
                    "wasm `rethrow` must target a live legacy catch handler"
                );
            }
            Instruction::End => {
                self.frames.pop();
            }
            Instruction::Else => {
                let frame = self
                    .frames
                    .last_mut()
                    .expect("an open frame was checked above");
                assert_eq!(
                    frame.kind,
                    FrameKind::IfThen,
                    "wasm `else` must belong to an unmatched `if`"
                );
                // Both arms share a label; only its structural state changes.
                frame.kind = FrameKind::IfElse;
            }
            Instruction::Br(label)
            | Instruction::BrIf(label)
            | Instruction::BrOnNull(label)
            | Instruction::BrOnNonNull(label) => self.check_branch(*label),
            Instruction::BrTable(labels, default) => {
                self.check_branch(*default);
                for label in labels.iter() {
                    self.check_branch(*label);
                }
            }
            Instruction::BrOnCast { relative_depth, .. }
            | Instruction::BrOnCastFail { relative_depth, .. } => {
                self.check_branch(*relative_depth);
            }
            _ => {}
        }
        self.body.instruction(instruction);
        self
    }

    /// Capture immediately after opening the frame that will be targeted.
    pub(crate) fn label_depth(&self) -> LabelDepth {
        let frame = self
            .frames
            .last()
            .expect("a finished body has no live label");
        LabelDepth {
            depth: self.depth(),
            identity: frame.identity,
        }
    }

    /// Resolve a live target in this body to a relative branch immediate.
    ///
    /// Testing depth alone misses a closed block followed by a sibling block
    /// at the same depth. Identity must be checked at the recorded position.
    pub(crate) fn branch_depth_to(&self, label: LabelDepth) -> BranchDepth {
        let frame = label
            .depth
            .checked_sub(1)
            .and_then(|index| self.frames.get(index as usize));
        assert!(
            frame.is_some_and(|frame| frame.identity == label.identity),
            "branch target label is not open at this point: its frame was closed or belongs to another body"
        );
        BranchDepth(self.depth() - label.depth)
    }

    pub(crate) fn branch_to_label(&mut self, label: LabelDepth) {
        let depth = self.branch_depth_to(label);
        self.instruction(&Instruction::Br(depth.immediate()));
    }

    pub(crate) fn branch_if_to_label(&mut self, label: LabelDepth) {
        let depth = self.branch_depth_to(label);
        self.instruction(&Instruction::BrIf(depth.immediate()));
    }

    #[cfg(test)]
    pub(crate) fn byte_len(&self) -> usize {
        self.body.byte_len()
    }

    pub(crate) fn into_body_named(
        self,
        context: &dyn core::fmt::Display,
    ) -> wasm_encoder::Function {
        assert!(
            self.frames.is_empty(),
            "function body for {context} has an unclosed control frame: {} label(s) still open",
            self.frames.len()
        );
        let raw = self.body.into_raw_body();
        let mut body = self.locals.encode_body();
        body.raw(raw[self.initial_prefix_len..].iter().copied());
        body
    }

    #[cfg(test)]
    pub(crate) fn into_body(self) -> wasm_encoder::Function {
        self.into_body_named(&"an unnamed test body")
    }
}

// The complete declaration enters the same owner as its emitted body.
const _: fn(LocalDeclarations) -> Function = Function::new;

// Standalone Wasm fixtures live outside the product module-assembly boundary.
#[cfg(test)]
#[path = "../tests/unit/code_sink.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/code_sink_exceptions.rs"]
mod exception_tests;
