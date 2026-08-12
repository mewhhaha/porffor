# Precompiled realm Script evaluation

This contract defines the only permitted Wasm-AOT implementation shape for
source text that is known while Lila compiles the containing program. It does
not permit a parser, interpreter or compiler inside the emitted artifact.

The first consumer is Test262's `$262.evalScript`, but the representation is a
general compiler facility. It must not inspect a test path, assertion text or
harness-specific source fragment.

## Why source splicing is wrong

A string passed to realm evaluation is a separate ECMAScript Script. Appending
its statements to the containing program would change all of these observable
facts:

- its parse and early errors would occur before the call rather than when the
  call executes;
- its global declarations would be instantiated too early and against the
  wrong current global state;
- its completion value would be lost or merged with the caller's completion;
- declared functions would not be freshly allocated for each evaluation;
- its native errors, intrinsics and global bindings could belong to the caller
  rather than the callable's realm;
- arguments after the source and an overwritten `evalScript` property could be
  skipped.

Consequently, a compile-time source proof produces a precompiled Script unit,
not syntax inserted into the outer Script.

## Closed compiler representation

`AotSourceText` is a private syntax-derived proof. The initial grammar may
admit only primitive string literals, no-substitution template literals,
parentheses and recursively pure concatenations of those forms. A lowered or
folded `ExprIr::String` is not sufficient evidence because obtaining that
value may have observable evaluation or coercion.

Each proven source receives a `StaticScriptId` in one
`PrecompiledScriptRegistry`. Its prepared result is closed:

- `Executable` owns the parsed Script, early-error-free IR and its runtime
  declaration plan;
- `DeferredSyntaxError` owns enough typed information to create the error in
  the target realm when the call executes.

Parser or compiler implementation failures remain compiler diagnostics. Only
an ECMAScript parse or early error becomes a deferred JavaScript `SyntaxError`.

The call IR retains the evaluated callee reference, receiver and every
argument. It may specialize to `RealmEvalStatic` only after resolving the
compiler-owned intrinsic identity and proving the source syntax. Emission must
still evaluate the callee, receiver and all arguments exactly once in source
order, then check the runtime callable identity. Replacing `$262.evalScript`
must call the replacement and never execute the precompiled unit.

## Runtime GlobalDeclarationInstantiation

The existing `GlobalBindingPlan` is the compile-time authority for a fresh
entry Script. Re-evaluating a Script needs the same unique name vocabulary but
cannot reuse its eager mutation policy: the target global object and global
environment may have changed since artifact initialization.

`RuntimeGlobalDeclarationPlan` therefore carries the precomputed declaration
sets and exact source function identities, while runtime code performs the
observable declaration checks:

1. `HasLexicalDeclaration` and `HasVarDeclaration` conflicts;
2. `CanDeclareGlobalFunction` in reverse declaration order;
3. `CanDeclareGlobalVar` for each remaining `var` name;
4. creation of lexical bindings;
5. creation of fresh function objects in the target realm;
6. creation of remaining `var` bindings.

All checks finish before any target-realm mutation. Their only successful
result is a must-use `ValidatedGlobalDeclarationInstantiation` token, and only
consuming that token may create bindings. A failed lexical collision therefore
cannot leave a partial `var` property behind.

Per-realm global binding state distinguishes at least `Absent`, `VarDeclared`,
`LexicalUninitialized` and `LexicalInitialized`. The global object's property
storage remains separate from the declarative record. Descriptor and
extensibility decisions go through the general object internal methods; the
precompiled Script path must not duplicate a Test262-only property table.

## Realm execution capability

A precompiled Script thunk receives a proven target RealmRecord/global
environment and returns the normal four-slot completion ABI. It never receives
source bytes.

The realm-evaluation builtin derives the target realm from the callable's
defining-realm slot. A must-use realm-execution token installs that realm and
its global environment and restores the caller's context on every normal or
abrupt exit. Body completion and arbitrary thrown values pass through
unchanged. Parse and declaration errors use the target realm's intrinsics.

Created realms cannot consume this facility until they allocate and retain a
real global environment. Their `evalScript`, `getGlobal` and `destroy`
functions must be realm-local function objects, not entry-realm wrappers. A
zero `HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET` or a fallback to the singleton
`SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX` is an invalid target, not permission to
use the entry realm.

The first coherent implementation slice may target the entry realm, but it
must already use the precompiled thunk, deferred error and runtime declaration
contracts. A declaration-free shortcut is not a useful substitute: the
current Annex B failure family is specifically about descriptor-sensitive
GlobalDeclarationInstantiation and no-partial-mutation behavior.

## Verification obligations

Durable contracts must prove:

- syntax proof accepts literal/template/pure-literal-concatenation forms and
  rejects merely folded or runtime strings;
- parse and early errors are deferred until invocation;
- callee, receiver and every argument evaluate once and in order;
- an overwritten callable wins over the specialization;
- all declaration checks precede every mutation;
- a failed collision leaves no partial binding;
- repeated evaluation shares realm bindings but creates fresh function
  objects;
- normal completion and arbitrary thrown identity survive;
- target global, intrinsics, defining realm and captured global environment
  agree;
- caller realm/global context is restored on normal and abrupt exits.

The focused real-suite checkpoint begins with the current Annex B
`$262.evalScript` cluster, then all `language/global-code` realm-evaluation
cases, and finally the cross-realm Proxy function-realm case. Generic runtime
source remains the typed `RuntimeCompilation` gap and is not a failure of this
static contract.
