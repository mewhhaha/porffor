# Host builtin surface catalog

Lila has host-backed callables in four different semantic groups:

- ECMAScript globals implemented by the host path: `parseInt` and
  `parseFloat`;
- product extensions: `print` and `gc`;
- Test262 capabilities: the `__lila*` assertion, realm, HTMLDDA, detach and
  agent hooks, including the typed realm-evaluation boundary;
- the internal `IsHTMLDDA` callable, which is emitted as an implementation
  dependency but is not a global binding.

Those groups used to be implicit in repeated string and enum lists. In
particular, AOT name lookup excluded `IsHTMLDDA` with a one-off inequality,
the lowerer repeated all 17 then-visible spellings, and created realms separately
listed the two host-backed ECMAScript globals. A new host builtin could compile
while being visible through only some of those paths.

## Closed row source

`lila_ir::HostBuiltinId` is now generated from one macro-backed catalog. Every
row must provide:

1. its Rust identity;
2. its callable name;
3. its compiler function id;
4. a `HostBuiltinSurface`.

`HostBuiltinSurface` has only two shapes. `Global` requires both a
`HostBuiltinExposure` and a `HostBuiltinRealmScope`; `InternalCallable` cannot
carry either a global exposure or a realm installation scope. The realm scope
is also closed: `EntryRealmOnly` or `EveryRealm`.

The current classification is:

| Exposure | Realm scope | Rows |
| --- | --- | --- |
| ECMAScript global | every realm | `parseInt`, `parseFloat` |
| product extension | entry realm only | `print`, `gc` |
| Test262 capability | entry realm only | all globally named `__lila*` rows |
| internal callable | no global scope | `IsHTMLDDA` |

There are 19 catalog rows: 18 globally named callables and the one internal
`IsHTMLDDA` callable. `RealmEvalScript` is a Test262-only global whose call is
classified by the compiler as dynamic-source debt; its defensive AOT body is
not product support for source evaluation.

The catalog derives `HostBuiltinId::ALL`, function-id round trips,
`global_name`, `from_global_name`, the global-row iterator and the
every-realm iterator. This makes a new row without a surface decision a compile
error and prevents an internal callable from entering global lookup by a
forgotten string exclusion.

## Product consumers

The catalog is consumed immediately by:

- lowerer global-property knowledge;
- lowerer identifier-to-host-builtin resolution;
- script-global host binding construction;
- AOT host name lookup and complete builtin/stub iteration;
- created-realm installation of host-backed ECMAScript globals.

Host builtin arity remains an exhaustive `HostBuiltinId` match in AOT
planning. Callable bodies remain an exhaustive match in AOT emission. Those
are semantic domains of their own and do not need another string catalog.

## Authority boundary

`HostSurfacePolicy` is the closed authority over the catalog's exposure
classification:

- `Product` admits `EcmaGlobal` and the deliberate `ProductExtension` rows;
- `Test262` admits those rows plus `Test262Capability`;
- neither policy can expose an `InternalCallable` as a global.

`CompileOptions` defaults to `Product`. The engine carries the selected policy
through script, module and script-with-module-graph lowering, includes it in
the whole-program cache key, and copies it into every Test262 agent worker
compilation. The Test262 runner is the explicit `Test262` opt-in. This keeps
the choice at a typed compilation boundary: it is never inferred from a
filename, source spelling or the presence of a `__lila*` identifier.
The CLI likewise defaults to product authority; its explicit
`--host-surface test262` option exists for conformance fixtures, and the CLI
integration harness passes it rather than teaching fixture filenames special
meaning.

The AOT emitter does not re-authorize a spelling after lowering. Its two raw
identifier fallback sites may materialize only host builtins present in the
IR-derived compiled-host set; the complete host registry still supplies
stable stub/table identities, but a stubbed Test262 row is not exposure
authority.

The IR still represents an unresolved identifier normally when product source
spells a Test262-only name; it simply does not mint a host binding or host
function dependency for that name. Normal ECMAScript reference semantics then
determine the result at execution. This contract governs reachability of host
capabilities, not parser acceptance of their former spellings.

The catalog says which host names exist; it does not decide what a source
declaration does with one of those names. That decision belongs to the unique
`GlobalBindingPlan` described in `global-binding-plan.md`, including
pre-existing-property collisions and Annex-B function-copy targets.
