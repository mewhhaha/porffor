# Computed RegExp lookbehind direction

The runtime compiler uses the existing matcher program and reverse matcher;
it does not evaluate a stored JavaScript AST or dispatch to a host matcher.
Each parser node stores its containing sequence's direction. Ordinary groups
inherit it, while an assertion independently selects forward lookahead or
reverse lookbehind for its child sequences. Width validation rejects any
direction word outside that closed two-value domain before code emission.

The lowerer consumes a closed child-order choice: alternatives retain source
priority, and sequences require their validated direction. Reverse sequences
assign descending instruction intervals to their source-linked terms. Reverse
captures emit the end boundary before the body and the start boundary after it.
Lookaround success and failure both restore the containing direction using the
existing matcher operand bits. Nested assertions and huge-bound rewrites use
the same ownership rule; no second matching state or opcode representation is
introduced.

This follows [ECMAScript 2026 CompileAssertion](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-compileassertion):
lookbehind compiles its Disjunction backwards, while the continuation retains
the incoming end index. Alternatives keep source order in both directions.
Annex B quantified lookahead remains separate; quantified lookbehind is a
SyntaxError. Computed Legacy, `u` and `v` patterns use this same directional
route with the validated character-mode owner. Unicode code-point comparison,
folding and finite-set lowering retain their existing consumers; no separate
lookbehind capability gate remains.

The authored computed-pattern fixture covers reverse greediness and capture
indices, alternatives, numbered/named references, nested orientation, negative
capture restoration, anchors, modifiers, surrogate pairs, nullable/huge bounds,
sticky lastIndex, recompile publication and genuine syntax errors. It is not
executed. Rust source checks do not verify the emitted matcher. Focused runtime
regressions and broad verification remain pending; no Test262 or runtime PASS
is claimed. Full T19 acceptance remains open.
