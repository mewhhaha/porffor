# UnicodeSets finite string algebra

Status: normative implementation contract for direct `\q{…}` operands and
exact finite Unicode properties of strings in the Rust RegExp matcher-program
producer.

## Exact conformance boundary

The selected raw Test262 cohort is the 27 generated files under
`built-ins/RegExp/unicodeSets/generated` whose names combine a direct
`string-literal` with only finite code-point operands:

- `character-{union,intersection,difference}-string-literal.js`;
- `character-class-{union,intersection,difference}-string-literal.js`;
- `character-class-escape-{union,intersection,difference}-string-literal.js`;
- `character-property-escape-{union,intersection,difference}-string-literal.js`;
- `string-literal-{union,intersection,difference}-character.js`;
- `string-literal-{union,intersection,difference}-character-class.js`;
- `string-literal-{union,intersection,difference}-character-class-escape.js`;
- `string-literal-{union,intersection,difference}-character-property-escape.js`;
  and
- `string-literal-{union,intersection,difference}-string-literal.js`.

That is 27 physical files and 54 strict/non-strict executions. The complete
cohort is source-proven red at the selected head because the parser retains
every valid direct class string as `RequiresClassStringSemantics` and
`RegExpProgram::compile` returns the explicit `` `\q` string literals are
unsupported `` capability error before any operator-specific matcher path.
Focused representatives were executed; the full 54-execution refresh remains
an integration verification step rather than a claimed fresh measurement.

This contract supersedes the deferred direct-`\q` matcher boundary in
`regexp-unicode-set-expression-shape.md`. That contract remains authoritative
for malformed-`\q` grammar, complete-Pattern validation, and negated-class
early-error ordering.

The six adjacent generated files whose name contains
`property-of-strings-escape` are not part of this original 27-file cohort.
Their `\p{Emoji_Keycap_Sequence}` operand requires property-owned Unicode data,
not inference from the finite source literal. The finite keycap extension below
now supplies that exact table and verifies the broader source-derived keycap
inventory separately.

## Finite set domain

`ValidatedClassStringDisjunction` owns every parsed alternative as a sequence
of validated Unicode code points. Validation still happens before semantics:
the exact `\q{` delimiter, escaped/raw `ClassSetCharacter` rules, closing `}`,
and empty alternatives are unchanged.

The private `FiniteClassSet` is the exact-value component accepted by
UnicodeSets union, intersection, subtraction, nesting, and matcher-atom
construction. Its private fields maintain one canonical product:

- normalized, sorted, disjoint inclusive code-point ranges; and
- a sorted, duplicate-free finite set of strings whose lengths are either zero
  or at least two code points.

A one-code-point `\q` alternative is moved into the range component at the
sole constructor. It is therefore the same `CharSetElement` as an ordinary
character, class range, character-class escape, or code-point property escape.
There is no parallel "singleton string" representation whose intersection or
subtraction could disagree with the code-point algebra.

The algebra is exact and exhaustive:

- union unions normalized ranges and finite strings;
- intersection intersects normalized ranges and finite strings; and
- subtraction subtracts normalized ranges and finite strings.

`ClassSetValue` separately retains the specification's static
`MayContainStrings` witness. A direct class-string disjunction derives its
initial witness from its actual alternative lengths, but set operations then
apply the specified conservative rules: union uses OR, intersection uses AND,
and subtraction uses the left operand's witness. Negation is gated on this
static witness, not on the exact product after algebra. Thus
`[^\q{ab}--\q{ab}]` remains a syntax error even though its exact finite product
is empty. Empty strings are retained as real set members and make an admitted
matcher atom nullable.

`ClassSetValue` also carries sticky direct-`\q` provenance through every set
operation. Any `iv` class which used a direct class-string operand remains an
explicit `RequiresUnicodeSetStringCaseFolding` capability even if algebra
eliminates the strings or normalizes every singleton into ranges. This is the
honest conservative boundary for cases such as `[\q{a}&&A]`: applying case
closure only after raw set algebra is not equivalent to the specification.

## Closed matcher-atom lowering

The private `FiniteClassSetAtom` is created only from a completed
`FiniteClassSet` after set algebra and negation. It has private fields for:

1. multi-code-point instruction sequences sorted by descending code-point
   length;
2. one combined code-point range-set instruction, including an empty range set
   when no singleton exists; and
3. whether the empty string is present.

Equal-length string order is unobservable because the alternatives have no
captures and consume the same number of input code points. The fixed producer
shape nevertheless makes the observable priority non-negotiable:
multi-code-point strings first, the combined singleton matcher next, and the
empty alternative last. This is the `CompileAtom` longest-string-first rule.

`ParsedAtom::FiniteClassSet` is a real matcher atom, not a syntax-only marker.
`ProgramLowerer` emits its alternatives with ordinary `Split`/`Jump` control:
each split owns the existing input/capture snapshot, and every generated split,
jump, and literal passes through `ProgramLowerer::push`, preserving the 4096
instruction resource bound. It must not use nullable-quantifier
`ProgressSplit`: only an enclosing optional quantifier owns that progress
check. `atom_nullable` reads the atom's retained empty-member bit, so such an
enclosing quantifier selects the already-typed nullable progress path.

Backward/lookbehind lowering preserves the same alternative priority and
reverses only the code-point evaluation order inside each multi-code-point
sequence. Forward and reverse compilation therefore consume the same
`FiniteClassSetAtom` rather than independently reconstructing its algebra.

The generic `RequiresClassStringSemantics` marker is removed. A complete valid
direct `\q` set can no longer compile while silently bypassing matcher
emission: the exhaustive `ParsedAtom` matches in nullability, lookbehind
admission, named-backreference traversal, forward lowering, and reverse
lowering all name `FiniteClassSet`.

Unicode properties of strings without exact finite tables remain the distinct
typed capability `RequiresUnicodePropertyOfStrings`. Set-expression parsing
propagates that capability while it finishes syntax and `MayContainStrings`
early errors. A property with an explicit finite table instead enters the same
`FiniteClassSet` algebra as direct class strings; it is never inferred from a
source `\q` operand.

### Finite keycap property extension

Unicode 17 `Emoji_Keycap_Sequence` is the exact twelve-member set
`[#*0-9] FE0F 20E3`. Its property parser constructs those complete
three-code-point strings directly, so a bare
`\p{Emoji_Keycap_Sequence}` atom and UnicodeSets union, intersection and
subtraction all consume the canonical finite product above. The property sets
`MayContainStrings` independently of the post-algebra product, preserving the
negated-class early error.

`Basic_Emoji` and the remaining `RGI_Emoji*` properties retain
`RequiresUnicodePropertyOfStrings`. Adding another property requires its own
revision-pinned table and focused raw inventory; recognizing its name is not
permission to approximate it with code-point ranges.

## Case-insensitive boundary

This cohort uses `v` without `i`. Code-point-only results with no direct-`\q`
provenance keep the existing case closure. Any `iv` set which used a direct
class string remains the distinct typed
`RequiresUnicodeSetStringCaseFolding` capability until operand-local
MaybeSimpleCaseFolding is represented. It must not emit a post-algebra range
closure or exact-literal matcher and pretend that it implements `Canonicalize`.

The finite keycap property is an explicit identity-fold exception: `#`, `*`,
ASCII digits, FE0F and 20E3 are all unchanged by simple case folding, so its
`iv` atom is bytecode-identical to `v`. The constructor is named
`finite_case_invariant_property_of_strings`, the IR witness compares both
programs, and the Wasm fixture executes the `iv` form. A future finite property
containing a casable code point must not use that constructor; it needs the
operand-local folding representation above.

## Explicit nonclaims

This contract implements only the exact finite `Emoji_Keycap_Sequence`
property table. It does not implement `Basic_Emoji`, the remaining
`RGI_Emoji*` properties, arbitrary runtime pattern compilation, or broad
UnicodeSets conformance. It does not change malformed `\q` syntax or
negated-class early errors. It does not add a new Wasm matcher opcode or data
pool: finite source and property strings lower to the existing ordered matcher
bytecode. Global/sticky wrappers, `lastIndex`, RegExp subclass behavior,
dynamic source generation, and unrelated property escapes remain outside the
batch.

## Durable producer invariants

The focused `lila-ir` witness must prove:

- singleton `\q` alternatives and ordinary ranges participate in the same
  union/intersection/subtraction component;
- duplicate strings are removed and multi-code-point strings are ordered by
  descending length;
- the empty alternative is retained, is emitted last, and makes the atom
  nullable;
- forward and reverse programs preserve alternative priority while reversing
  only each string sequence;
- nested set operations produce the same canonical product as top-level ones;
- direct finite strings no longer produce a capability error, while an `iv`
  string set remains explicit unsupported; and
- the exact twelve-member keycap table enters the same direct-atom and set
  algebra path while other string properties remain typed unsupported;
- the source-derived 37-file keycap inventory stays unflagged, contains exactly
  three parse-negative files, and has no runner, shortcut, or known-failure
  mask; and
- every emitted branch and literal is subject to `REGEXP_MAX_INSTRUCTIONS`.

## Verification

Producer/static stage:

```sh
cargo fmt --all -- --check
cargo test -p lila-ir unicode_sets_finite_string_algebra
git diff --check
./scripts/check-module-boundaries.sh
```

Integrated focused stage:

```sh
./target/debug/lila test262 run built-ins/RegExp/unicodeSets/generated \
  --suite-root test262/vendor/test262 --execution-backend wasm-aot \
  --snapshot-name regexp-unicode-set-finite-strings \
  --timeout-ms 180000 --threads 1

rg -lF 'Emoji_Keycap_Sequence' \
  test262/vendor/test262/test/built-ins/RegExp/property-escapes/generated/strings \
  test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated | sort
```

The keycap search must return exactly 37 unflagged files: four direct-property
files and 33 UnicodeSets algebra files, for 74 sloppy/strict executions. Run
those exact relative paths independently under `wasm-aot`; the three
`Emoji_Keycap_Sequence-negative-{CharacterClass,P,u}.js` files must remain
parse-time `SyntaxError` successes. Publication reports that inventory
separately from the original 27-file/54-execution direct-`\q` cohort and from
the complete generated directory.
