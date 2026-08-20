//! The only path from an emitter to a Wasm function body, and the only place
//! that knows how deep the label stack is.
//!
//! # Why this type exists
//!
//! A Wasm `br` immediate is a *label* index: it counts the `block`/`loop`/`if`
//! frames that are open at the branch, outermost-last, with the function body
//! itself as the outermost label. Nothing else counts — not scopes, not
//! statements, not this compiler's `control_stack`.
//!
//! Until this module existed, the branch arithmetic used a different number.
//! `FunctionBuilder::depth_to` counted only the frames pushed through
//! `push_control`, and the 7,399 raw `Instruction::If`/`Block`/`Loop` values
//! written directly into the body by individual emitters were invisible to it.
//! Each call site that opened one was expected to declare it by hand through an
//! `extra_depth` argument, and the declarations had drifted: two arms of one
//! `if`/`else` chain in `builtins/array.rs` passed `0` and `3` for the same
//! frame count, and several emitters declared nothing at all.
//!
//! The consequence was a live, Crash-class defect. A property read that throws
//! inside a `for` loop branched to the loop's back edge instead of the JS
//! handler — the loop spun and eventually trapped — and the same read inside a
//! `switch` had its throw discarded silently. Neither wasm validation nor the
//! golden-byte gate can see it: both the right and the wrong label index are in
//! range, and a `br` immediate is the same width either way.
//!
//! # What it does
//!
//! [`Function`] wraps [`wasm_encoder::Function`] and maintains `depth`, the
//! real number of open labels. Because *every* instruction in this crate is
//! written through [`Function::instruction`], a frame opened by any emitter —
//! declared or not — is counted, with no edit at the 77,558 emission sites.
//!
//! The two figures a branch is built from are newtypes with private fields
//! ([`LabelDepth`], [`BranchDepth`]) constructible only here, so a bare integer
//! cannot be used as a branch immediate, and the arithmetic that turns a label
//! into a branch immediate exists exactly once
//! ([`Function::branch_depth_to`]).
//!
//! # What it deliberately does not do
//!
//! Raw `Instruction::Br(n)` is still expressible, and is still correct for a
//! self-contained region that opens and closes its own frames (the regexp
//! matcher's backtracking loops, the sloppy-mode abandon branch on a
//! non-extensible write). Those immediates are relative to the current
//! position, which this change does not move. What is no longer expressible is
//! a *`ControlTarget`-relative* branch whose immediate was hand-corrected for
//! frames the builder could not see.

use wasm_encoder::{Instruction, ValType};

/// A position on the Wasm label stack: the number of labels open at the moment
/// a frame was entered, counting the function body as label 1.
///
/// Recorded by [`Function::label_depth`] when a control frame is opened, and
/// stored in `ControlTarget`. It is *not* a branch immediate: subtracting one
/// from the other is what produces a branch immediate, and
/// [`Function::branch_depth_to`] is the only subtraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LabelDepth(u32);

impl LabelDepth {
    /// Builds a `LabelDepth` without a sink. Test-only on purpose: in product
    /// code the only source of a `LabelDepth` is [`Function::label_depth`], so
    /// a target's label can never be a number someone counted by hand.
    #[cfg(test)]
    pub(crate) const fn for_test(depth: u32) -> Self {
        Self(depth)
    }
}

/// A Wasm `br`/`br_if` immediate: how many labels to exit.
///
/// Constructible only by [`Function::branch_depth_to`], and consumable only by
/// [`Function::branch_to_label`] / [`Function::branch_if_to_label`], so it
/// cannot be built from a literal, adjusted, or forwarded through a helper as
/// a bare `u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BranchDepth(u32);

impl BranchDepth {
    const fn immediate(self) -> u32 {
        self.0
    }
}

/// A Wasm function body under construction, together with its real label
/// depth.
///
/// Named `Function` and re-exported from `lib.rs` in place of
/// [`wasm_encoder::Function`] so that the whole crate — every `&mut Function`
/// parameter and every `function.instruction(..)` call — goes through it
/// without changing a single one of those signatures or call sites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Function {
    body: wasm_encoder::Function,
    /// Open labels, including the function body itself. `1` for a fresh body;
    /// `0` once the body's terminating `End` has been written.
    depth: u32,
}

/// The function body is itself a label: `Br(0)` in an otherwise flat body
/// returns from the function.
const FUNCTION_BODY_LABEL_DEPTH: u32 = 1;

impl Function {
    /// Mirrors [`wasm_encoder::Function::new`] — the run-length form.
    ///
    /// Test-only, and marked so rather than left `pub(crate)`: no product path
    /// declares locals in runs of differing types (every emitted body is one
    /// run of `i64`), and an unmarked constructor here would compile with no
    /// call site, which is the shape `AGENTS.md` asks to make impossible.
    /// `emitted_function`'s decoder tests need mixed runs, so it survives for
    /// them alone.
    #[cfg(test)]
    pub(crate) fn new<L>(locals: L) -> Self
    where
        L: IntoIterator<Item = (u32, ValType)>,
        L::IntoIter: ExactSizeIterator,
    {
        Self {
            body: wasm_encoder::Function::new(locals),
            depth: FUNCTION_BODY_LABEL_DEPTH,
        }
    }

    /// Mirrors [`wasm_encoder::Function::new_with_locals_types`]. The only
    /// constructor reachable from the product path.
    pub(crate) fn new_with_locals_types<L>(locals: L) -> Self
    where
        L: IntoIterator<Item = ValType>,
    {
        Self {
            body: wasm_encoder::Function::new_with_locals_types(locals),
            depth: FUNCTION_BODY_LABEL_DEPTH,
        }
    }

    /// Writes one instruction, accounting for what it does to the label stack.
    ///
    /// This is the whole mechanism. `Block`/`Loop`/`If` open a label, `End`
    /// closes one, and `Else` closes and reopens the same label so the depth
    /// does not move.
    pub(crate) fn instruction(&mut self, instruction: &Instruction<'_>) -> &mut Self {
        match instruction {
            Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) => {
                self.depth += 1;
            }
            Instruction::End => {
                self.depth = self.depth.checked_sub(1).expect(
                    "wasm `end` with no open label: the emitter closed more frames than it opened",
                );
            }
            // `else` ends the `then` arm and starts the `else` arm of the
            // *same* `if` label. Depth is unchanged, and the matching `End`
            // below closes it.
            Instruction::Else => {}
            // `assert!`, not `debug_assert!`, and deliberately so. The root
            // `[profile.release]` sets only `debug = "line-tables-only"`; it
            // does not enable debug assertions. A `debug_assert!` here is
            // therefore absent from `./target/release/lila` — the binary that
            // runs the 53,131-case sweep, which is the only place this check
            // would ever have caught something. It would have been a
            // protection that exists exactly where it is not needed. The two
            // neighbouring protections (`End` underflow above, `into_body`
            // below) are unconditional, and the cost here is one comparison
            // per branch instruction against a stream that already pays a
            // match per instruction.
            Instruction::Br(label) | Instruction::BrIf(label) => {
                assert!(
                    *label < self.depth,
                    "branch immediate {label} is out of range at label depth {}",
                    self.depth
                );
            }
            Instruction::BrTable(labels, default) => {
                assert!(
                    *default < self.depth && labels.iter().all(|label| *label < self.depth),
                    "br_table immediate is out of range at label depth {}",
                    self.depth
                );
            }
            // Exception-handling frames would move the label stack, and the
            // two `br_on_*` forms carry label immediates this sink does not
            // range-check. A census over the crate finds none of them emitted
            // anywhere outside this file: the only control instructions any
            // emitter writes are `End`, `If`, `Else`, `Br`, `Block`, `BrIf`
            // and `Loop`. That is the claim doing the work, so it is stated
            // without per-opcode totals — those move with every batch that
            // touches an emitter, and a stale number reads as a measurement.
            // Recheck the claim, not the counts:
            //
            //   rg -o 'Instruction::(Try|TryTable|Delegate|Catch|CatchAll\
            //     |Rethrow|BrOn\w+|BrTable)\b' \
            //     --glob '!code_sink.rs' crates/lila-aot-wasm/src
            //
            // Silently miscounting is exactly the failure mode this type
            // exists to remove, so adding one must fail loudly here first
            // rather than shifting every branch below it by one.
            Instruction::Try(_)
            | Instruction::TryTable(_, _)
            | Instruction::Delegate(_)
            | Instruction::Catch(_)
            | Instruction::CatchAll
            | Instruction::Rethrow(_)
            | Instruction::BrOnNull(_)
            | Instruction::BrOnNonNull(_) => {
                panic!(
                    "code_sink does not account for this control instruction yet; \
                     teach `Function::instruction` about it before emitting it"
                )
            }
            _ => {}
        }
        self.body.instruction(instruction);
        self
    }

    /// The number of labels open right now, counting the function body.
    ///
    /// Record this immediately *after* writing a frame's opening instruction
    /// to get the label a branch should name; `FunctionBuilder::open_frame`
    /// (`control_flow.rs`) is the only product-code caller, which is what keeps
    /// "emit the frame, then record it" from being a convention someone can
    /// forget.
    pub(crate) fn label_depth(&self) -> LabelDepth {
        LabelDepth(self.depth)
    }

    /// The `br` immediate that reaches `label` from the current position.
    ///
    /// The one subtraction in the compiler that turns two label positions into
    /// a branch immediate. Panics rather than wrapping if `label` is not
    /// currently open — a `Br` to a closed frame is a miscompile, and the
    /// alternative is an in-range immediate naming the wrong block.
    pub(crate) fn branch_depth_to(&self, label: LabelDepth) -> BranchDepth {
        BranchDepth(self.depth.checked_sub(label.0).expect(
            "branch target label is not open at this point: its frame was closed before the branch",
        ))
    }

    /// Writes `br` to `label`.
    pub(crate) fn branch_to_label(&mut self, label: LabelDepth) {
        let depth = self.branch_depth_to(label);
        self.instruction(&Instruction::Br(depth.immediate()));
    }

    /// Writes `br_if` to `label`.
    pub(crate) fn branch_if_to_label(&mut self, label: LabelDepth) {
        let depth = self.branch_depth_to(label);
        self.instruction(&Instruction::BrIf(depth.immediate()));
    }

    /// Mirrors [`wasm_encoder::Function::byte_len`]. Test-only for the same
    /// reason as [`Function::new`]: the product path measures a body after
    /// [`Function::into_body`], through `EmittedFunction`.
    #[cfg(test)]
    pub(crate) fn byte_len(&self) -> usize {
        self.body.byte_len()
    }

    /// Hands the finished body over, asserting that every frame it opened was
    /// closed, and *naming the body* if one was not.
    ///
    /// An unbalanced body is invalid Wasm, but wasmtime reports it as a module
    /// validation failure with no function name, inside whichever of the 53,131
    /// Test262 cases happened to compile it. This turns that into a panic at
    /// the function boundary, in the process that built it.
    ///
    /// The `context` argument is why this takes one at all. A nameless
    /// "function body has an unclosed control frame" is the same anonymous
    /// diagnostic the wasmtime failure is, only earlier — it tells an operator
    /// that *some* body among several thousand is unbalanced. The product
    /// caller (`EmittedFunction::new`) passes the `FunctionIdentity` it is
    /// building, so the panic names the symbol that goes into the Wasm `name`
    /// section.
    ///
    /// The sibling protection in [`Function::instruction`] — the `End`
    /// underflow — genuinely cannot name the body: the sink is constructed
    /// before any identity is known at several of its call sites, so there is
    /// nothing to name. Its panic is deliberately phrased as a statement about
    /// the emitter's bracketing rather than about a particular function.
    pub(crate) fn into_body_named(
        self,
        context: &dyn core::fmt::Display,
    ) -> wasm_encoder::Function {
        assert_eq!(
            self.depth, 0,
            "function body for {context} has an unclosed control frame: {} label(s) still open",
            self.depth
        );
        self.body
    }

    /// [`Function::into_body_named`] with no name to give.
    ///
    /// Test-only: the product path always has a `FunctionIdentity` in hand by
    /// the time a body is finished, and an unnamed panic there is the failure
    /// this module set out to stop reporting.
    #[cfg(test)]
    pub(crate) fn into_body(self) -> wasm_encoder::Function {
        self.into_body_named(&"an unnamed test body")
    }

    /// Rewrites the body's local declaration from `planned_local_count` down to
    /// `emitted_local_count`, preserving the label depth.
    ///
    /// This lives here rather than in `emit.rs::finish_function` because it is
    /// the one operation that rebuilds a body from raw bytes: doing it through
    /// the public constructors would silently reset `depth` to
    /// [`FUNCTION_BODY_LABEL_DEPTH`], and this is called on bodies that are
    /// already closed.
    pub(crate) fn rewrite_local_declaration(
        self,
        planned_local_count: u32,
        emitted_local_count: u32,
    ) -> Self {
        let depth = self.depth;
        let local_declaration =
            wasm_encoder::Function::new([(planned_local_count, ValType::I64)]).into_raw_body();
        let mut body_bytes = self.body.into_raw_body();
        assert!(
            body_bytes.starts_with(&local_declaration),
            "function local declaration does not match planned local count {planned_local_count}"
        );
        let instruction_bytes = body_bytes.split_off(local_declaration.len());
        let mut body = wasm_encoder::Function::new([(emitted_local_count, ValType::I64)]);
        body.raw(instruction_bytes);
        Self { body, depth }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::BlockType;

    fn empty_body() -> Function {
        Function::new_with_locals_types(std::iter::empty())
    }

    #[test]
    fn a_fresh_body_is_one_label_deep() {
        // `Br(0)` in a flat body returns from the function, so the body itself
        // must count as a label or every branch is one too shallow.
        assert_eq!(empty_body().label_depth(), LabelDepth(1));
    }

    #[test]
    fn raw_frames_are_counted_without_being_declared() {
        let mut function = empty_body();
        function.instruction(&Instruction::Block(BlockType::Empty));
        let outer = function.label_depth();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::If(BlockType::Empty));

        assert_eq!(function.label_depth(), LabelDepth(4));
        assert_eq!(function.branch_depth_to(outer), BranchDepth(2));
        assert_eq!(
            function.branch_depth_to(function.label_depth()),
            BranchDepth(0),
            "a branch to the frame you are standing in is `br 0`"
        );
    }

    #[test]
    fn else_closes_and_reopens_the_same_label() {
        let mut function = empty_body();
        function.instruction(&Instruction::If(BlockType::Empty));
        let then_arm = function.label_depth();
        function.instruction(&Instruction::Else);

        assert_eq!(
            function.label_depth(),
            then_arm,
            "`else` must not move the depth: it is the same `if` label"
        );

        function.instruction(&Instruction::End);
        assert_eq!(function.label_depth(), LabelDepth(1));
    }

    #[test]
    fn a_loop_label_is_the_back_edge() {
        let mut function = empty_body();
        function.instruction(&Instruction::Block(BlockType::Empty));
        let exit = function.label_depth();
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let back_edge = function.label_depth();
        function.instruction(&Instruction::If(BlockType::Empty));

        // This is the shape the defect showed up in: a throw inside the `if`
        // must reach `exit`, not `back_edge`.
        assert_eq!(function.branch_depth_to(back_edge), BranchDepth(1));
        assert_eq!(function.branch_depth_to(exit), BranchDepth(2));
    }

    #[test]
    #[should_panic(expected = "unclosed control frame")]
    fn into_body_rejects_an_unclosed_frame() {
        let mut function = empty_body();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::End);
        // The body's own terminating `End` is missing.
        let _ = function.into_body();
    }

    #[test]
    fn into_body_accepts_a_balanced_body() {
        let mut function = empty_body();
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        assert_eq!(function.label_depth(), LabelDepth(0));
        let _ = function.into_body();
    }

    #[test]
    #[should_panic(expected = "closed more frames than it opened")]
    fn an_extra_end_is_not_silently_absorbed() {
        let mut function = empty_body();
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    #[test]
    #[should_panic(expected = "not open at this point")]
    fn branching_to_a_closed_frame_is_rejected() {
        let mut function = empty_body();
        function.instruction(&Instruction::Block(BlockType::Empty));
        let inner = function.label_depth();
        function.instruction(&Instruction::End);
        let _ = function.branch_depth_to(inner);
    }

    #[test]
    fn rewriting_the_local_declaration_keeps_the_depth() {
        let mut function = Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, 4));
        function.instruction(&Instruction::Block(BlockType::Empty));
        let inner = function.label_depth();
        let rewritten = function.rewrite_local_declaration(4, 2);

        assert_eq!(rewritten.label_depth(), inner);
        assert_eq!(
            rewritten.branch_depth_to(inner),
            BranchDepth(0),
            "the rebuilt body must still know which labels are open"
        );
    }
}
