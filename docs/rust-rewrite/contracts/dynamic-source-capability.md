# Dynamic-source AOT capability boundary

## Decision

Lila's Wasm-AOT artifact never contains a parser, interpreter, or VM. Source
that is not known until execution therefore remains a compiler capability gap,
not an ECMAScript rejection and not a passing conformance result. Source proven
at AOT time may eventually be compiled only through the ordinary
front-end-to-Wasm pipeline.

`%eval%` does not always evaluate source. `PerformEval` returns its argument
unchanged before selecting a realm or environment when the argument is not a
primitive String, and `%eval%` called with no arguments returns `undefined`.
Those branches are ordinary builtin execution, not dynamic-source support, and
are admitted by the closed proof below.

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
directly resolved call or construction first produces
`UnsupportedFeature::DynamicSource`, and a program with that diagnostic is
rejected before backend planning. Forwarding through `call`, `apply`,
`Reflect.apply`, `Reflect.construct`, bound functions or proxies does not yet
carry the underlying dynamic-source identity and remains explicit accounting
debt.

## Product-path invariants

1. A diagnostic is emitted only after lowering has resolved a call target to a
   compiler-owned intrinsic identity. Identifier spelling is not proof.
2. Direct eval is distinguished from indirect eval only when the resolved
   `%eval%` target remains the direct global-reference call form. Aliases,
   property calls, comma calls, and optional calls are indirect.
3. A source proof is constructed from syntax before lowering. It accepts
   primitive string literals, no-substitution templates, parentheses and pure
   concatenations of those forms. Static string facts obtained by folding any
   other expression are not proofs because its evaluation or coercions may be
   observable.
4. Function-family arguments are AOT-known only when every argument has that
   syntax proof and there is no spread. Parameter and body strings will still
   require separate parser goals before any subset can be enabled.
5. The typed diagnostic is a compiler gap. It has no early-error code or native
   error type and cannot satisfy a negative Test262 expectation.
6. The existing zero-argument Function-constructor shortcut is not static
   compilation: it manufactures the wrong callable. It is rejected through the
   same typed boundary until the real target-realm path exists.
7. Generator, async and async-generator function object shapes carry their
   respective constructor identity through the intrinsic prototype's
   `constructor` property. The identity follows aliases and property reads; a
   source identifier named `GeneratorFunction` is not evidence by itself.
8. Optional calls retain the same pre-lowering source proof as ordinary calls.
   Reanalysis of an already-lowered optional-chain prefix is marked as already
   accounted, so it cannot silently downgrade or duplicate the diagnostic.
9. The Test262 harness obtains realm `evalScript` from one typed host builtin
   admitted by `HostSurfacePolicy::Test262`. Product lowering cannot resolve
   that global, and the harness stores the resolved function value directly on
   `$262`, preserving literal-source proof at its eventual call site. The host
   body is a defensive throw only; a resolved call is rejected by the compiler
   diagnostic before backend planning.

## Proven no-source `%eval%`

The lowering boundary classifies each resolved dynamic-source call exactly once
as either `EvalPassThrough(ProvenEvalPassThrough)` or
`Unsupported(DynamicSourceGap)`. The pass-through proof has private
constructors and exists only for direct or indirect intrinsic `%eval%` when:

- the call has no spread and no arguments; or
- the call has no spread and its lowered first argument has a nonempty
  `KindSet` that excludes primitive `String`.

An empty kind set is not evidence. A set containing `String`, any spread,
realm `evalScript`, and every Function-family identity remain typed gaps. For a
multi-target call, every dynamic-source target must independently produce the
pass-through proof; one unsupported target rejects the call.

The proof permits lowering to retain the ordinary indirect call. It never
replaces the call with its first argument or `undefined`: the evaluated callee
and every argument remain in source order, an overwritten `eval` still wins,
and abrupt argument completion is unchanged. Its result fact is `undefined`
for no arguments and the first argument's exact `ValueInfo` otherwise.

This is intentionally not an AOT-known textual subset. No String source is
parsed, compiled, or executed by this branch, and it establishes none of the
caller-environment, target-realm, declaration-instantiation, or deferred-error
capabilities required by static Script evaluation.

## Current producer coverage

| Operation | Compiler-owned identity today | Accounting |
| --- | --- | --- |
| direct/indirect `%eval%` on directly resolved call paths | `StandardBuiltinId::EvalFunction` | no-argument/proven non-String pass-through; typed diagnostic whenever String remains possible |
| ordinary `%Function%` on directly resolved call/construct paths | `StandardBuiltinId::FunctionConstructor` | typed diagnostic |
| Generator/Async/AsyncGenerator Function constructors on directly resolved call/construct paths | `DynamicSourceIntrinsic::Function(..)` carried by the function prototype shape | typed diagnostic |
| directly resolved `$262.evalScript` calls | `HostBuiltinId::RealmEvalScript`, mapped to `DynamicSourceIntrinsic::RealmEvalScript` and exposed only by `HostSurfacePolicy::Test262` | typed diagnostic |

There is no lexical Test262 pre-gate for these operations. Unsupported
accounting begins only after lowering resolves one of the identities above.
This does not claim static-source support: literal strings still produce
caller- or target-realm-environment debt, while values that may be primitive
Strings produce runtime-compilation debt. Only proven no-source `%eval%` calls
avoid the dynamic-source diagnostic.

Forwarding builtins and wrapper callables are not yet identity-transparent for
this accounting boundary. Calls that reach these operations through
`Function.prototype.call`/`apply`, `Reflect.apply`/`construct`, a bound function
or a Proxy can still fall through to a generic backend diagnostic; closing that
gap requires a typed forwarding target, not source-spelling recognition.

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
