# Callable non-simple parameters plus `ContainsUseStrict` early errors

## Decision

One closed pre-evaluation condition owns the conjunction shared by callable
productions:

`EarlyErrorCode::CallableNonSimpleParametersContainUseStrict`

Its sole wire spelling is
`E_CALLABLE_NON_SIMPLE_PARAMETERS_CONTAIN_USE_STRICT`. The condition is true
only when the callable's own body contains a Use Strict Directive and its own
parameter list is non-simple. It is a `SyntaxError` before evaluation.

This is not ambient strictness. A simple parameter list may accompany a Use
Strict Directive, a non-simple list may occur without one, and a directive in a
nested callable does not contribute to the enclosing body's `ContainsUseStrict`
result. Expression-bodied arrows cannot contain a directive prologue.

## Grammar boundary

The shared condition appears on ordinary, generator, async and async-generator
declarations and expressions; ordinary and async arrows; ordinary, generator,
async and async-generator methods; and setters. The productions spell the body
operation as `FunctionBodyContainsUseStrict`, `ConciseBodyContainsUseStrict`,
`AsyncConciseBodyContainsUseStrict` or the corresponding body's
`ContainsUseStrict`, but the conjunction and rejection are the same. Getters
have an empty parameter list by grammar and therefore cannot satisfy it.

Pinned `boa_parser-0.21.1` began this batch with this exact raw message at
eighteen textual sites:

```text
Illegal 'use strict' directive in function with non-simple parameter list
```

`LexError::Syntax` appends the source position, so the classifier's complete
stable fragment is the raw message followed by ` at line`. The sites group as
one shared callable-declaration parser, five class-element branches, five
object/method parsers, four function/generator expression parsers and three
arrow paths.

Those raw counts were not a sound typed boundary:

- the class private-getter branch permissively parses
  `UniqueFormalParameters` even though the grammar requires `()`; its message
  can therefore describe malformed getter syntax rather than this condition;
- the direct `ArrowFunction` combinator is called only after recognizing one
  `BindingIdentifier`, so its non-simple-list branch cannot execute; and
- the private and public class-setter branches use unrestricted
  `UniqueFormalParameters`, allowing malformed zero-, multi- or rest-parameter
  setters to reach a condition that assumes `PropertySetParameterList`.

The parser boundary is now normalized: private getters require `()`, class
setters parse exactly one ordinary formal parameter, and the direct
binding-identifier arrow path no longer carries an impossible non-simple-list
check. Sixteen parser literals remain, each on an executable, spec-conforming
production path. A source inventory test embeds the ten reviewed files and pins
their per-file counts and the dead direct-arrow count of zero. The identical
literal in `boa_engine`'s dynamic `Function` constructor is not a `lila-front`
parser producer and remains outside this AOT classifier.

## Typed encoding

- The one variant and wire name above extend `EarlyErrorCode::ALL` from 52 to
  53, so an omitted or duplicated domain row fails to compile.
- One `PARSE_FAILURE_RULE_TABLE` row uses the complete rendered prefix. The row
  count is 52 rather than encoding Boa's sixteen source locations as sixteen
  semantic variants. A const ownership witness makes deleting this row while
  retaining the variant a compile failure.
- `lila-ir`'s exhaustive `rejection_kind` match maps the variant to
  `IrDiagnosticKind::EarlyError`. `ParseClassified` remains the only parse-stage
  carrier, so entry and retained-dependency parsing derive the same
  `Early`/`SyntaxError` pair.
- The existing const proofs retain populated and disjoint rows, witness
  classification, ordered and injective wire names, parse-table reachability
  and parse-to-IR phase consistency.

The parser repairs are part of the invariant, not merely test setup. Every
remaining fixed-message producer starts from a grammar-valid parameter shape,
and the dead arrow producer no longer inflates the measured message domain.

## Durable regressions

The front-end source matrix exercises each of the sixteen remaining producer
sites under Script and Module goals. Shared method parsers need one object or
class representative per source site rather than duplicate tests for every
caller. Every rejection must carry the new code, `Early` phase,
`SyntaxError`, and a nonempty span.

Positive and negative boundaries separately prove the conjunction and the
grammar repairs:

- simple parameters plus a Use Strict Directive remain valid;
- non-simple parameters without the directive remain valid;
- a nested callable's directive and a post-prologue string do not count;
- private getters accept only `()` and malformed getter parameters remain an
  unclassified parse error; and
- class setters accept exactly one non-rest parameter, while a default or
  destructuring parameter can still reach this early-error condition.

One retained exported callable proves that the real dependency ParseError is
projected through `module_parse_failure_diagnostic`; a hand-built diagnostic is
not sufficient.

## Exact pinned cohort

The exact current-pin source inventory is every `.js` file under
`test262/vendor/test262/test/language` containing
`FunctionBodyContainsUseStrict` or `ContainsUseStrict`: 110 physical files.
All 110 declare a parse-phase `SyntaxError`; 96 are generated and none has a
mode flag, so the harness expands them to 220 sloppy/strict Wasm-AOT
executions. The similarly worded
`built-ins/Function/StrictFunction_reservedwords_with.js` exercises dynamic
source construction and is deliberately excluded.

No pre-change focused execution snapshot was captured: changing an untyped
parse rejection into a typed early error may leave the observable negative
pass count unchanged. The focused result therefore establishes bounded
no-regression, not a pass gain without separate baseline evidence.

## Nonclaims

This batch does not implement dynamic `Function` or direct `eval`, runtime
parameter environments, mapped `arguments`, general setter/method execution,
every callable grammar error, T07 closure or aggregate Test262 closure. It does
not label arbitrary strict-mode failures or source-interpolating parser errors
with this code.

## Evidence

Landed and focused-verified on `2026-08-23` under the repository's capped,
serial verification policy:

- `cargo check -p lila-front -p lila-ir --all-targets` passes.
- `cargo test -p lila-front --lib -- --test-threads=1` passes `85/85`.
- `cargo test -p lila-ir modules::early::tests -- --test-threads=1` passes
  `38/38`, including a real retained exported-function rejection.
- `cargo test -p lila-ir early_error -- --test-threads=1` passes `3/3`.
- `cargo xc` is green, and the independently reviewed taxonomy, producer
  inventory and grammar repairs have no outstanding findings.
- The isolated 110-file cohort passes all `220/220` sloppy/strict Wasm-AOT
  executions. The verified snapshot contains 220 `Success` outcomes, zero
  failures, zero timeouts and zero `NotImplemented`, `Crash` or `Bug` outcomes.
  Its exact completed execution-ID set matches the two expected modes for each
  inventoried path. The temporary suite shape separately verifies 110 test
  links, 110 resolved files, zero dangling links and the vendored harness, then
  removes the scratch tree.

This is a bounded no-regression result. There is no focused pre-change
snapshot, so it is not evidence of a pass gain.
