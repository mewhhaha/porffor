# Computed UnicodeSets classes and finite strings

Computed `RegExp` construction and `compile` with `v` use the existing runtime
compiler to parse complete code-point and finite-string unions, intersections, subtractions,
ranges, nested classes and complements. The native property catalog, descriptor
format and matcher remain the same consumers. This is source implementation;
compilation, emitted Wasm validation and semantic execution are still pending.

The grammar owns each class's operation and operand phase. A range requires two
actual character operands; singleton nested classes, built-in classes, properties
and `\q` remain set operands. Operator chains cannot mix `&&`, `--` and adjacent
union operands. ClassSetCharacter decoding retains strict control, zero, fixed
hexadecimal and Unicode escape validation. Ordinary `u` and Legacy classes keep
their distinct grammar.

Each completed operand becomes a private `ClosedClassSetOperand` before the
workspace accepts it. Case closure uses the current modifier scope before union,
intersection, subtraction or complement. The first operand seeds a class before
the grammar selects an intersection or subtraction chain. Nested completion
copies its closed bitmap and complete finite-key set into actual operand
scratch and restores the parent frame. Singleton, multi-code-point and empty
members remain distinct through exact set algebra. q keys and all seven property
payloads fold and deduplicate before algebra, including scoped modifiers.

Class metadata is bounded by source units. Each active depth obtains a bitmap
once, lazily, and reuses it for subsequent classes at that depth. Both initial
workspace allocation and bitmap extensions consume a checked workspace-end
owner. They grow, initialize and release the same private compiler checkpoint;
no JavaScript call or allocation can escape from it. Syntax or resource failure
clears the complete tail, and successful descriptor publication reclaims scratch.
Failed `compile` preserves the receiver's installed descriptor and public slots.

`MayContainStrings` is a separate static fact: union uses OR, intersection uses
AND, subtraction preserves its left operand. Negated classes reject a true fact
even when algebra produces an empty set. q cardinality counts decoded characters,
including one paired astral character versus two braced surrogate atoms. The
finite domain separately owns actual surviving keys and an empty-key bit; static
negation legality cannot be inferred from the resulting set.

A completed finite atom replaces the former unsupported marker. Checked private
payload rows contain prepared existing literal/range instructions, ordered by
descending complete key length before singleton and empty alternatives. The
lowerer checks private spans, nullability and instruction expansion before
emitting the existing Split/Jump and character instructions. The final graph
validator checks actual range operands before descriptor publication.
Reverse matching consumes key positions in reverse order and retains UTF-16
capture indices. Existing nullable-quantifier progress handles empty alternatives.
Short property escapes do not expand thousands of keys into source-sized AST
nodes. No new matcher opcode, descriptor format or alternate matcher is added.

The complete pattern and capture/name inventory must be validated before
lowering and descriptor publication. Invalid later syntax and unknown names
remain SyntaxErrors; failed preparation preserves the installed descriptor.
The [finite catalog](runtime-regexp-finite-string-catalog.md) describes all seven
validated immutable sequence payloads and their existing provider authority.

Existing finite Engine controls cover code-point algebra, empty/full sets,
supplementary and lone surrogate membership, scoped closure, complements, strict
grammar and transactional recompilation. Three additional paired sources cover
complete string properties and q algebra, longest-priority backtracking, required
empty progress, iv folding, reverse bounds and receiver rollback. Former string
capability controls now require normal admission and matching. Sources remain
uncompiled and unexecuted; Rustfmt and ordinary source transport do not establish
matching or conformance. Focused regressions and broad verification remain due.

Primary authority: [ECMAScript 2026 UnicodeSets grammar](https://tc39.es/ecma262/2026/multipage/text-processing.html#prod-ClassSetExpression)
and [MayContainStrings](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-static-semantics-maycontainstrings).
