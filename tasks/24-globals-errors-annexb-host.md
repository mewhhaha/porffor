# T24 — Globals, native errors, Annex B and remaining host-visible builtins

**Status:** Blocked on core semantic foundations  
**Parallel group:** Feature lane; split by errors, globals and Annex B  
**Depends on:** T04, T06, T07, T09, T10; dynamic evaluation uses T13; strings/RegExp use T18/T19  
**Blocks:** Remaining builtins/Annex B/harness portions of T26

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
cargo test -p porffor-ir annex_b --quiet
cargo test -p porffor-aot-wasm error_ --quiet
cargo test -p porffor-aot-wasm global_ --quiet
cargo test -p porffor-cli wasm_error --quiet
./target/debug/porf test262 run built-ins/Error --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/porf test262 run built-ins/AggregateError --execution-backend wasm --timeout-ms 120000 --threads 4
./target/debug/porf test262 run annexB --execution-backend wasm --timeout-ms 180000 --threads 4
```

Also run URI/global-value/global-function filters, SuppressedError, AbstractModuleSource and IsHTMLDDA-focused cases.