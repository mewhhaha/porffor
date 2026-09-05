//! The only path from an emitter to a Wasm function body.
//!
//! Every raw `block`, `loop` and `if` contributes to the real label stack,
//! including frames that the JS control-flow builder does not manage. A branch
//! target records both its stack position and its identity: a closed frame must
//! not become a valid target again when a sibling reuses its depth.
//!
//! These checks are unconditional, including in release builds used for
//! conformance runs. They validate emission structure, not Wasm operand types;
//! the Wasm validator remains responsible for the latter.

use std::sync::atomic::{AtomicU64, Ordering};

use wasm_encoder::{Instruction, ValType};

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

/// A function body and its open frames, including the implicit function label.
///
/// Re-exported as `Function` from the crate root so all emitters use the same
/// accounting. An empty frame stack means the final `end` has been emitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Function {
    body: wasm_encoder::Function,
    frames: Vec<Frame>,
}

impl Function {
    /// The run-length constructor is needed only by decoder tests.
    #[cfg(test)]
    pub(crate) fn new<L>(locals: L) -> Self
    where
        L: IntoIterator<Item = (u32, ValType)>,
        L::IntoIter: ExactSizeIterator,
    {
        Self {
            body: wasm_encoder::Function::new(locals),
            frames: vec![Frame::new(FrameKind::Function)],
        }
    }

    pub(crate) fn new_with_locals_types<L>(locals: L) -> Self
    where
        L: IntoIterator<Item = ValType>,
    {
        Self {
            body: wasm_encoder::Function::new_with_locals_types(locals),
            frames: vec![Frame::new(FrameKind::Function)],
        }
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
            // No product emitter currently uses these exception-control forms.
            // Keep them explicitly rejected until their catch/delegate label
            // semantics are implemented, rather than silently miscounting them.
            Instruction::Try(_)
            | Instruction::TryTable(_, _)
            | Instruction::Delegate(_)
            | Instruction::Catch(_)
            | Instruction::CatchAll
            | Instruction::Rethrow(_) => {
                panic!(
                    "code_sink does not account for this control instruction yet; \
                     teach `Function::instruction` about it before emitting it"
                );
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
        self.body
    }

    #[cfg(test)]
    pub(crate) fn into_body(self) -> wasm_encoder::Function {
        self.into_body_named(&"an unnamed test body")
    }

    /// Replace the local declaration without changing frame identity or state.
    pub(crate) fn rewrite_local_declaration(
        self,
        planned_local_count: u32,
        emitted_local_count: u32,
    ) -> Self {
        let Self { body, frames } = self;
        let local_declaration =
            wasm_encoder::Function::new([(planned_local_count, ValType::I64)]).into_raw_body();
        let mut body_bytes = body.into_raw_body();
        assert!(
            body_bytes.starts_with(&local_declaration),
            "function local declaration does not match planned local count {planned_local_count}"
        );
        let instruction_bytes = body_bytes.split_off(local_declaration.len());
        let mut body = wasm_encoder::Function::new([(emitted_local_count, ValType::I64)]);
        body.raw(instruction_bytes);
        Self { body, frames }
    }
}

// Standalone Wasm fixtures live outside the product module-assembly boundary.
#[cfg(test)]
#[path = "../tests/unit/code_sink.rs"]
mod tests;
