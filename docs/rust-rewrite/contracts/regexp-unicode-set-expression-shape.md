# UnicodeSets class-expression shape

Status: normative for the Rust RegExp parser's `v`-mode character-class seam.

## Semantic boundary

An otherwise non-empty `ClassContents[+UnicodeSetsMode]` is exactly one
`ClassSetExpression`. ECMA-262 22.2.1 gives that expression three mutually
exclusive shapes:

- `ClassUnion` is a sequence of `ClassSetOperand` or `ClassSetRange` nodes;
- `ClassIntersection` contains at least two `ClassSetOperand` nodes separated
  only by `&&`; and
- `ClassSubtraction` contains at least two `ClassSetOperand` nodes separated
  only by `--`.

An intersection or subtraction operand is one operand, not an implicit union.
Consequently `[ab&&c]`, `[a&&bc]`, mixed `[a&&b--c]`, and an operator with a
missing side are syntax errors. An empty class remains the separate empty
`ClassContents` production.

Each operand is a `NestedClass`, `ClassStringDisjunction`, or
`ClassSetCharacter`; `NestedClass` includes both a nested bracketed class and
an escaped `CharacterClassEscape`. The raw-character alternative has two
lexical exclusions that ordinary character classes do not:

- `ClassSetSyntaxCharacter` (`( ) [ ] { } / - \\ |`) cannot occur raw as a
  character operand; and
- no `ClassSetReservedDoublePunctuator` — the doubled forms of
  `& ! # $ % * + , . : ; < = > ? @ ^`, grave accent, or `~` — may be consumed
  as two adjacent characters.

A single member of that doubled-punctuator set remains legal when it is not
doubled. A `ClassSetReservedPunctuator` such as `&` is also legal when escaped,
so `[a&&\&]` is a grammatical intersection rather than a missing or third
operator.

The `CharacterEscape :: 0` alternative has a negative `DecimalDigit`
lookahead. `\0` is one validated character; `\01` is not `\0` followed by a
raw `1`. The same restriction applies inside a `\q{…}` class string.

## Closed parser protocol

`ClassSetOperator` has exactly `Intersection` and `Subtraction`. It owns both
the source delimiter and the exhaustive range-set operation. No string value or
catch-all chooses set semantics.

The parser consumes one typed `ClassSetOperand` before it chooses a production:

- absence of an operator commits to the union parser, which may consume more
  operands and character-only ranges but rejects a later `&&` or `--`; or
- an operator commits to the homogeneous operation-tail parser, which requires
  a right operand and may continue only with the same operator.

The private `ClassSetCharacter` newtype can be constructed only by the
UnicodeSets validator. `ClassSetOperand` then distinguishes that validated
character from a nested set and a `ValidatedClassStringDisjunction`. The
distinction makes both endpoints of `ClassSetRange` explicit: neither a nested
class, character-class escape nor class string can silently become a range
endpoint, and a raw character cannot bypass the UnicodeSets lexical exclusions
through the ordinary-class parser.

Malformed expression shape is `InvalidSyntax` carrying the dedicated
`ClassSetExpression` rule. Invalid raw character operands carry
`ClassSetCharacter`; malformed `\q{…}` delimiters carry
`ClassStringDisjunction`. A negated class whose completed contents have
`MayContainStrings = true` carries `NegatedClassMayContainStrings`. Those four
rules each cite their exact ECMA-262 22.2.1 production or early error and have
a pinned witness.

`\q` is not a capability escape hatch. The parser first requires the exact
`\q{` prefix, consumes alternatives through an unescaped closing `}`, and
validates every non-`|` body member as a `ClassSetCharacter`. Empty strings and
empty alternatives are grammatical. A closed three-state `ClassStringLength`
domain computes the specified static semantics: an empty or multi-character
alternative may contain strings, while an exactly one-character alternative
does not.

Successful local validation produces a typed class-string operand; it does not
return `UnsupportedFeature`. `ClassSetValue` retains both the need for string
semantics and the exact `MayContainStrings` value while the union,
intersection, or subtraction tail is parsed. Its exhaustive operations mirror
ECMA-262 22.2.1.8: union uses Boolean OR, intersection uses AND, and subtraction
uses its left operand. The parser then requires the enclosing `]`, checks range
bounds and operators, and applies the negated-class early error. A fully valid
outer class that still needs string-set matching produces the typed
`RequiresClassStringSemantics` marker rather than an error.

That marker remains in the parsed term tree through the complete Pattern pass.
The parser must close every containing group, reject stray closing
parentheses, and validate named-backreference and duplicate-name early errors.
Parser-side matcher restrictions also query the atom subtree for this marker:
in particular, an unbounded quantifier over an otherwise nullable group cannot
return its own capability error first. `ParsedPatternCapability` is a closed
choice between `MatcherReady` and the earliest class-string marker. The
lowerer's exhaustive atom match treats the marker as syntax-only and cannot
return a matcher program for it; `RegExpProgram::compile` reports
`UnsupportedFeature` only after the entire Pattern is globally syntax-valid.

Consequently `[\q{a}` (missing `]`), `[\q{a}&&]`, `[\q{a}-b]`,
`[a-\q{b}]`, and `[\q{a}!!]` are syntax errors rather than capability gaps.
`[^\q{ab}]` and `[^\q{}]` are the `MayContainStrings` early error, while
`[^\q{a}]`, `[^\q{ab}&&a]`, and `[^a--\q{ab}]` are syntactically valid and
remain unsupported.

The same ordering applies beyond the class: `[\q{a}])` is a stray closing
parenthesis, `[\q{a}](` is an unclosed group, and
`[\q{a}]\k<missing>` names no group. Valid controls such as `[\q{a}]b` and
`([\q{a}])` finish the global syntax pass and remain capability gaps. The
nullable quantified control `(?:[\q{a}]|)*` does too, while appending `)`, `(`,
or `\k<missing>` still reaches the corresponding whole-Pattern early error.

## Durable witnesses

The IR regressions require the dedicated syntax rule for:

- mixed intersection and subtraction in either order;
- a union on either side of an operation;
- a missing left or right operand;
- a third `&` after an intersection delimiter;
- raw syntax characters and all reserved double punctuators, including the
  previously accepted `[a&&-]`, `[a---]`, and `[!!]`; and
- malformed `\q` prefixes, missing closing braces, forbidden raw body
  characters, doubled punctuators, and character-class escapes inside a class
  string;
- a locally valid class string followed by an unclosed class, malformed
  operation, forbidden range, or reserved double punctuator;
- a locally valid class string followed by a stray parenthesis, unclosed group,
  or unknown named backreference elsewhere in the Pattern;
- the same three trailing early errors after a marker-bearing, nullable group
  with an unbounded quantifier;
- negated class strings whose exact `MayContainStrings` result is true; and
- `\0` followed by a decimal digit both directly and inside `\q{…}`.

Positive witnesses preserve empty and ordinary unions, character ranges,
homogeneous chained intersections and homogeneous chained subtractions with
exact resulting range sets, escaped `\&` operands, raw singleton punctuators,
valid `\0`, and syntactically legal class-string expressions that remain
unsupported only after their enclosing class validates. A focused Wasm fixture
exercises both statically resolved constructor arguments and computed
runtime-table entries, with legal siblings preventing a blanket rejection from
passing.

## Nonclaims and deferred gates

This seam validates but does not implement `\q` class-string matching. It does
not implement Unicode properties of strings, arbitrary runtime pattern
compilation, full UnicodeSets conformance, matcher opcodes, resource limits,
UTF-16 cursor behavior, `lastIndex`, or the RegExp object protocol. It changes
no published conformance count and does not close T19 or T26.

Static freeze gates are `rustfmt --check` for the touched Rust files,
`node --check` for the fixture, a focused source scan proving the raw string
operator state is gone, `git diff --check`, and manual exhaustive-match review.
Cargo, fixture execution, focused pinned UnicodeSets trees, and the broad batch
ladder remain deferred until the frozen patch is independently reviewed.
