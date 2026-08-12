# Dynamic-source AOT capability boundary

## Decision

Lila's Wasm-AOT artifact never contains a parser, interpreter, or VM. Source
that is not known until execution therefore remains a compiler capability gap,
not an ECMAScript rejection and not a passing conformance result. Source proven
at AOT time may eventually be compiled only through the ordinary
front-end-to-Wasm pipeline.

This contract does not implement that static subset. It makes the current gap
compiler-visible without inferring support from test paths or source snippets.

## Closed domain

`DynamicSourceKind` names the semantic operation:

- direct `eval`;
- indirect `eval`;
- realm `evalScript`;
- one of the four Function-family constructors.

`DynamicSourceGap` has private fields. Its constructors derive the requirement
from the operation:

- source not proven at AOT time requires runtime compilation;
- AOT-known direct eval requires a caller-environment lowering seam;
- AOT-known indirect eval, realm evaluation, and Function-family construction
  require a target-realm environment lowering seam.

Call sites cannot construct a mismatched pair such as Function construction
plus a caller eval environment. `UnsupportedFeature::DynamicSource` is carried
by `IrDiagnostic`; consumers classify the typed value and do not parse its
display text.

`DynamicSourceIntrinsic` is the closed identity catalog for dynamic-source
operations that are callable compiler intrinsics but are not executable AOT
builtins. It maps the four Function-family constructors and realm
`evalScript` to stable function IDs. The ordinary Function row reuses
`StandardBuiltinId::FunctionConstructor`; the three derived rows and realm
evaluation are compiler-only identities. They acquire no Wasm emitter: every
resolved invocation first produces `UnsupportedFeature::DynamicSource`, and a
program with that diagnostic is rejected before backend planning.

## Product-path invariants

1. A diagnostic is emitted only after lowering has resolved a call target to a
   compiler-owned intrinsic identity. Identifier spelling is not proof.
2. Direct eval is distinguished from indirect eval only when the resolved
   `%eval%` target remains the direct global-reference call form. Aliases,
   property calls, comma calls, and optional calls are indirect.
3. A source proof accepts only lowered primitive string literals. Static string
   facts obtained by folding arbitrary expressions are not proofs because the
   expression and its coercions remain observable.
4. Function-family arguments are AOT-known only when every argument is a
   primitive string literal and there is no spread. Parameter and body strings
   will still require separate parser goals before any subset can be enabled.
5. The typed diagnostic is a compiler gap. It has no early-error code or native
   error type and cannot satisfy a negative Test262 expectation.
6. The existing zero-argument Function-constructor shortcut is not static
   compilation: it manufactures the wrong callable. It is rejected through the
   same typed boundary until the real target-realm path exists.
7. Generator, async and async-generator function object shapes carry their
   respective constructor identity through the intrinsic prototype's
   `constructor` property. The identity follows aliases and property reads; a
   source identifier named `GeneratorFunction` is not evidence by itself.
8. The Test262 harness obtains realm `evalScript` from one typed host builtin
   admitted by `HostSurfacePolicy::Test262`. Product lowering cannot resolve
   that global, and the harness stores the resolved function value directly on
   `$262`, preserving literal-source proof at its eventual call site. The host
   body is a defensive throw only; a resolved call is rejected by the compiler
   diagnostic before backend planning.

## Current producer coverage

| Operation | Compiler-owned identity today | Accounting |
| --- | --- | --- |
| direct/indirect `%eval%` on resolved call paths | `StandardBuiltinId::EvalFunction` | typed diagnostic |
| ordinary `%Function%` on resolved call/construct paths | `StandardBuiltinId::FunctionConstructor` | typed diagnostic |
| Generator/Async/AsyncGenerator Function constructors | `DynamicSourceIntrinsic::Function(..)` carried by the function prototype shape | typed diagnostic |
| `$262.evalScript` | `HostBuiltinId::RealmEvalScript`, mapped to `DynamicSourceIntrinsic::RealmEvalScript` and exposed only by `HostSurfacePolicy::Test262` | typed diagnostic |

There is no lexical Test262 pre-gate for these operations. Unsupported
accounting begins only after lowering resolves one of the identities above.
This does not claim static-source support: literal strings still produce
target-realm-environment debt, while non-literals produce runtime-compilation
debt.

## Static-subset prerequisites

`precompiled-realm-scripts.md` defines the implementation contract for
AOT-proven Script source. In particular, static realm evaluation means a
separate precompiled Script thunk, deferred ECMAScript parse/early errors and
runtime GlobalDeclarationInstantiation. It never means splicing statements
into the caller or manufacturing a declaration-free host result.

Direct eval additionally needs an eval parse goal, delayed parse/early errors,
runtime `%eval%` identity checking, caller variable and lexical environments,
strictness, `this`, `new.target`, private environment, declaration
instantiation, completion values, and realm-correct errors.

Function-family construction needs separate parameter/body parse goals and
combined early errors, ordinary argument evaluation and coercion order, runtime
constructor identity and `newTarget`, and function allocation against the
constructor realm's global environment. Until those seams exist, a literal
source is classified as AOT-known debt rather than compiled by a shortcut.
