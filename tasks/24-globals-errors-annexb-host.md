# T24 — Globals, native errors, Annex B and remaining host-visible builtins

**Status:** In progress — errors/globals/URI/host exotics are broad but not fully closed

**Parallel group:** Feature lane; split by errors, globals and Annex B  
**Depends on:** T04, T06, T07, T09, T10; dynamic evaluation uses T13; strings/RegExp use T18/T19  
**Blocks:** Remaining builtins/Annex B/harness portions of T26

## Current repository state

Native errors, global constants/functions, URI codecs, Annex B builtins,
IsHTMLDDA and AbstractModuleSource have dedicated runtime/backend paths and
many focused real-suite results. The Wasm backend now validates every
hand-written runtime-error name at its string boundary and immediately carries
`NativeErrorKind`; one exhaustive mapping owns both the global and per-realm
prototype locations, the realm snapshot entries derive from
`NativeErrorKind::ALL`, and the static `instanceof` fast path consumes the same
mapping. A new error family omitted from that authority is therefore a compile
error, and an invalid internal spelling can no longer silently fall back to
`%Object.prototype%`.

The 19 host-backed callables now also come from one macro-backed
`HostBuiltinId` row source. Each row must classify its global exposure and
realm scope, so the compiler derives complete/global/every-realm iteration and
name lookup without parallel lists or a raw `HTMLDDA` exclusion. Lowering,
AOT lookup/stub planning and created-realm installation consume that catalog.
The nineteenth row is the Test262-only realm-eval callable: T13 rejects every
resolved invocation as typed dynamic-source debt, and its defensive AOT body
exists only so the harness can store a valid function value, not as support.
`HostSurfacePolicy` is now the authority over those classifications. Product
compilation, including the CLI/default engine path, admits ECMAScript globals
and the deliberate `print`/`gc` extensions but does not resolve Test262-only
`__lila*` globals. The Test262 harness opts in explicitly, the policy is part
of the whole-program cache key, every IR lowering entry point receives it, and
agent workers inherit the root compilation's policy. The focused authority
acceptance requirement is therefore closed. CLI conformance fixtures use the
explicit `--host-surface test262` opt-in; product CLI compilation retains the
default `Product` policy. AOT raw-identifier fallbacks are also limited to the
IR-derived compiled-host set, so the backend cannot re-authorize a filtered
spelling through its complete stub registry. See
`docs/rust-rewrite/contracts/host-builtin-surface.md`.

Global declaration instantiation now has one typed IR authority as well.
`GlobalBindingPlan` owns a unique map instead of allowing lowering to append
duplicate predefined/`var`/function rows for AOT to collect last-wins. Property
initialization and declaration claims are separate: `var Infinity` retains the
immutable predefined value and descriptor, while duplicate source functions
carry the exact last `FunctionId`. Global lexical names stay in the separate
declarative-name set. Annex-B copies carry an exhaustive owner-binding versus
script-global target, and script-global writes reuse the plan's existing-
property policy rather than unconditionally overwriting raw object storage;
every mirrored Set then resynchronizes the frame cache from the authoritative
global property, including properties hardened after instantiation.
See `docs/rust-rewrite/contracts/global-binding-plan.md`.

Error/global/Annex B metadata and legacy behavior still appears in other
exact-path materializations, dynamic-source cases remain visible exclusions,
and the full assigned trees lack current complete Wasm-AOT closure.

## Objective

Complete the foundational global properties/functions, native error families, Annex B extensions and standardized host-facing objects that do not belong to another feature lane. Keep ECMAScript semantics distinct from Test262 shell capabilities.

## Native Error objects

Implement the exact constructor/prototype behavior for every error family in the pinned suite:

- `Error`, `EvalError`, `RangeError`, `ReferenceError`, `SyntaxError`, `TypeError`, `URIError`;
- `AggregateError` and iterator consumption;
- `SuppressedError` and explicit-resource-management integration;
- any current standardized error additions in the pin.

Cover:

- call/construct and custom new target;
- realm-correct prototype fallback and cross-realm throws;
- `message`, `cause`, `errors`, `error` and `suppressed` own-property creation/order/descriptors;
- constructor/prototype `name`, `message`, `toString` and descriptors;
- arbitrary thrown JavaScript values through the T04 completion ABI;
- subclassing, proxies and side-effecting options/iterables.

A non-standard `stack` property may be offered as an extension only if it does not change standard own-key/descriptor tests or error construction order. Document and feature-gate its format.

## Global values and functions

Complete and install exact descriptors for:

- `globalThis`, `Infinity`, `NaN`, `undefined`;
- `isFinite`, `isNaN`, `parseFloat`, `parseInt`;
- `encodeURI`, `encodeURIComponent`, `decodeURI`, `decodeURIComponent`;
- Annex B `escape` and `unescape` where required;
- `eval` through T13;
- current standardized global constructors/functions not owned elsewhere.

URI algorithms must operate on UTF-16 code units/code points as specified, reject malformed surrogate/escape sequences with realm-correct `URIError`, and never delegate to platform URL encoding.

## Annex B grammar and semantics

Coordinate syntax/early errors with T07 and implement every Annex B feature enabled by the pinned suite, including as applicable:

- block-level function declaration web-legacy semantics and global instantiation interactions;
- catch-parameter/`var` compatibility exceptions;
- legacy octal literals/escapes and string/regexp grammar interactions;
- object-literal `__proto__` and `Object.prototype.__proto__` getter/setter;
- `Object.prototype.__defineGetter__`, `__defineSetter__`, `__lookupGetter__`, `__lookupSetter__`;
- String HTML wrapper methods;
- RegExp legacy `compile` and other Annex B RegExp semantics present in the pin;
- `escape`/`unescape`.

Do not enable Annex B semantics in modules or strict contexts where prohibited.

## Standardized host-facing objects

Implement or assign explicitly any pinned host-facing standard objects such as:

- `%AbstractModuleSource%` and source-phase import support, coordinated with T12;
- IsHTMLDDA behavior required by Test262, represented as a documented host exotic rather than ordinary truthy/falsy special cases scattered through operators;
- `$262` capabilities through T03, without installing Test262-only globals in normal product realms.

Host-only objects must use typed host capability interfaces and the same object/call/realm protocols as standard objects.

## Intrinsic/descriptor registry completion

Audit the declarative intrinsic registry from T06 against the current specification and suite manifest. Every global, constructor, prototype, method, accessor, symbol property and alias must have:

- owning realm/intrinsic ID;
- function `name` and `length`;
- callable/constructable flags;
- exact writable/enumerable/configurable attributes;
- constructor/prototype links and required identity aliases.

Generate a test that fails when Test262 references a known standard global absent from the registry.

## Acceptance criteria

- Full pinned native-error, global-function/global-value and Annex B trees are green.
- Error creation and throwing preserve exact realm/prototype/property order.
- URI functions pass malformed-surrogate, percent-escape and reserved-character cases.
- Annex B behavior is correctly gated by parse goal/strictness/context.
- IsHTMLDDA is implemented through one host-exotic contract, not operator-specific test hacks.
- Test262 shell globals are absent from ordinary product realms unless explicitly enabled.
- No focused static error/global/Annex B materialization remains for covered semantics.

## Required tests

```sh
cargo test -p lila-ir annex_b --quiet
cargo test -p lila-aot-wasm error_ --quiet
cargo test -p lila-aot-wasm global_ --quiet
cargo test -p lila-cli wasm_error --quiet
./target/debug/lila test262 run built-ins/Error --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run built-ins/AggregateError --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/lila test262 run annexB --execution-backend wasm --timeout-ms 180000 --threads 4
```

Also run URI/global-value/global-function filters, SuppressedError, AbstractModuleSource and IsHTMLDDA-focused cases.
