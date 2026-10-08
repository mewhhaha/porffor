# UnicodeSets finite string algebra

## Current candidate verification

Seven new IR controls and nine direct Engine fixtures pass, with the Engine
fixtures covering both Script modes. Ten selected adjacent RegExp files pass
20 pinned modes, including both circled-M property files and the Unicode
case-mapping neighbor. The 33 direct class-string files/66 modes describe the
/v-only inventory; the whole inventory was not replayed by this selection.
There are no selected direct /iv pins. The runtime pattern compiler is unchanged.

The candidate continuation revalidates 84 focused stages with 1,856 selected Rust test invocations on the exact same Source. Compilation,
one separate both-engine startup invocation and the default-features CLI
build belong to the original focused run. The continuation freezes that
CLI unchanged and executes all 151 selected pinned modes from 82 files.
The original pin-identity validation failure remains recorded.

This is candidate verification. MAIN installation and a fresh complete
MAIN broad checkpoint remain required. The earlier session 30300 is
INCOMPLETE without an owned terminal; its exit and cause remain unknown.
Full pinned Test262 conformance and task acceptance remain open. The
published status span is unchanged. The authentic continuation terminal is `bb22495c5187c67c269adeffd0e8efd91c3cbe97be18a79fb86c263945bc2798`; its Root-owned exit is `b399f9e5c16d0847ff4854a885e40c1b0b55f5428791d0186919db6dc64d0b1a`. The revalidated same-Source prefix is `7d0ce67ed3ce18e2646639461ba9eeb7f5d4e0793275b0937cf687ec96bf5bc9`; its original enclosing Root1 is `12a3deab22796d658bebdce50eaf263cf2a1443b1f03f0269dcdba951f9c77c1`.

The preparation and dated verification statements below retain their
original scope and failures. This checkpoint supersedes only the
unexecuted state of the named selected controls described above.


Status: normative implementation contract for direct `\q{…}` operands and
exact finite Unicode properties of strings in the Rust RegExp matcher-program
producer. The dated/source-cohort evidence below records the original direct-q
and keycap batches. The current producer expands all seven closed provider
properties of strings; historical keycap-only capability limits are superseded.

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
cohort was source-proven red at the original selected pre-batch head because the parser retained
every valid direct class string as `RequiresClassStringSemantics` and
`RegExpProgram::compile` returned the explicit `` `\q` string literals are
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

The prepared `/iv` successor applies Unicode simple folding to each direct
class-string and finite-property operand before algebra. Multi-code-point keys are canonicalized
character by character and deduplicated. Singletons enter the same fold-closed
range representation as ordinary characters, so intersections, subtractions
and nested complements agree with ordinary set operands. The static
`MayContainStrings` witness still follows the original syntax, including empty
and multi-code-point alternatives eliminated by algebra.

Each surviving string position then uses the existing scoped-modifier emitter
to match its complete simple-fold equivalence class. This reuses existing ASCII
class and range-pool instructions, retaining fallible range and instruction
bounds. Simple folding preserves code-point lengths and lone surrogates; it
does not expand sharp s into two letters. Fold-invariant provider strings keep
their existing instructions. The retired direct-string capability and its
syntax-only placeholder chain are removed rather than left as unreachable
states. No Wasm opcode, program descriptor or runtime compiler capability is
added.

The source proposal is uncompiled and unexecuted. Seven added IR regressions and
nine paired Engine controls remain required, together with retained compiler
and matcher controls. The circled-M controls cover bare property matching,
property/direct intersection and both subtraction directions, `/v` sensitivity,
and scoped modifiers for `Basic_Emoji` and `RGI_Emoji`. The pin has 33 direct class-string files (66 Script modes),
all `/v`-only: they provide adjacent coverage, not direct `/iv` evidence. No
conformance count or historical execution result changes.

Normative obligations come from [CompileToCharSet](https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-compiletocharset),
[MaybeSimpleCaseFolding](https://tc39.es/ecma262/multipage/text-processing.html#sec-maybesimplecasefolding),
[CompileAtom](https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-compileatom),
and [MayContainStrings](https://tc39.es/ecma262/multipage/text-processing.html#sec-static-semantics-maycontainstrings).

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
`[#*0-9] FE0F 20E3`. The pinned `regress` provider already owns those twelve
complete three-code-point sequences. The Lila property parser borrows that
table and converts it once into the private finite-set product, so a bare
`\p{Emoji_Keycap_Sequence}` atom and UnicodeSets union, intersection and
subtraction all consume the canonical finite product above. The property sets
`MayContainStrings` independently of the post-algebra product, preserving the
negated-class early error.

At this historical keycap-only checkpoint, the following capability limits
were still present; the current all-seven projection supersedes them.

`Basic_Emoji` and the remaining `RGI_Emoji*` properties retain
`RequiresUnicodePropertyOfStrings`. Adding another property requires its own
revision-pinned table and focused raw inventory; recognizing its name is not
permission to approximate it with code-point ranges.

### Closed provider boundary

The vendored provider exposes only `UnicodeStringProperty`, its strict name
parser and a read-only sequence accessor from the crate root. The generated
table module remains private. `UnicodeStringProperty` is the single closed
seven-variant authority for the exact ECMAScript property names, and the
provider's sequence accessor projects all seven variants exhaustively.

Lila parses the source spelling into that domain once at the RegExp boundary.
At the historical authority-hardening checkpoint, its catch-all-free match
had one semantic arm per provider variant:
`EmojiKeycapSequence` consumes the provider sequences and enters
`FiniteClassSet`; `BasicEmoji` and each of the five `RGIEmoji*` variants enter
the explicit `RequiresUnicodePropertyOfStrings` capability. No raw property
name is matched again inside Lila, and no handwritten keycap table remains.
Adding a provider variant therefore makes both the provider table projection
and the Lila semantic projection fail to compile until the new case is owned.
At that authority-hardening checkpoint, admitted keycap behavior and the six
other capability outcomes were unchanged. The current exhaustive projection
lowers all seven provider variants through the finite property constructor.

## Case-insensitive boundary

The original admitted cohort used `v` without `i` and retained direct `/iv`
class strings as a capability boundary. The prepared source-only successor now
represents operand-local `MaybeSimpleCaseFolding` before set algebra and matches
each surviving position through its simple-fold equivalence class. Verification
of that successor remains pending; the historical `/v` results do not prove its
new `/iv` behavior. Post-algebra closure alone cannot implement `Canonicalize`
for strings or normalize singleton aliases before set operations.

The provider operands use `finite_property_of_strings(strings, folding)` with
the active scoped modifier context. `Basic_Emoji` and `RGI_Emoji` contain
U+24C2 U+FE0F; the first code point simple-folds to U+24DC. Their string keys must
therefore fold before intersection or subtraction with a direct class string,
and their `/iv` matching instructions differ from `/v`. The other five pinned
provider tables are fold-invariant. The original keycap witness checks `#`, `*`,
ASCII digits, FE0F and 20E3, and its `iv` atom remains bytecode-identical to `v`.
The two pinned circled-M property files use `/v` only; their four Script modes
are adjacent controls and provide no direct `/iv` execution evidence.

## Explicit nonclaims

The current static producer implements the finite tables for
`Emoji_Keycap_Sequence`, `Basic_Emoji`, `RGI_Emoji_Flag_Sequence`,
`RGI_Emoji_Modifier_Sequence`, `RGI_Emoji_Tag_Sequence`, `RGI_Emoji_ZWJ_Sequence`
and `RGI_Emoji`. It does not imply arbitrary runtime property-pattern compilation
or broad UnicodeSets conformance. It does not change malformed `\q` syntax or
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
- direct `/iv` operands fold before algebra, including singleton aliases,
  and surviving string positions match their simple-fold equivalence classes;
- all seven provider tables enter the same finite direct-atom and set algebra
  path; the original keycap table retains its twelve-member witness;
- the strict provider parser and exhaustive seven-arm provider/Lila
  projections are the sole property-name and sequence authorities, with every
  arm using the provider sequence accessor;
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

The 2026-08-27 invariant-only provider refresh passed both focused IR witnesses
(`1/1` finite algebra and `1/1` strict closed names), the dedicated bounded
provider-domain structure target `3/3`, and the retained finite-string
structure target `7/7`. `cargo check -p lila-ir`, formatting and diff checks
were green with only the repository's existing warnings. No Wasm fixture,
golden or Test262 status was rerun because the admitted behavior did not
change.
