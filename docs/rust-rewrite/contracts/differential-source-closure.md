# Differential replay source-closure contract

Status: normative for differential corpus schemas v1, v2 and v3.

## Integrity claim

The current corpus wire formats carry one entry source, one parse goal and no
module graph. Their stable case fingerprints cover that source and the corpus
metadata, but no dependency graph. A replay whose meaning can depend on a
host-loaded module is therefore not reproducible from its corpus bytes: the
same case and fingerprint could run different programs in different checkouts
or working directories.

Schemas v1, v2 and v3 consequently admit only Scripts whose outer parsed source
contains no module request. A Module goal is rejected because the format has no
closed representation for module identity, base URLs or dependency edges used
by static imports and export-from declarations. A Script containing
`import()`, `import.defer()` or `import.source()` is rejected because its target
graph is absent too. This is an admission invariant, not a claim that modules
or dynamic import are invalid ECMAScript.

Outer-source classification is not a complete runtime proof. `eval`, indirect
eval, a Function constructor, a child realm or an agent can create an
`import()` whose tokens are not in the entry AST. Differential replay therefore
also fixes the shared `ModuleLoadingPolicy` to `RejectAll` for both backends.
Wasm graph preparation uses a host loader that rejects before reading a
dependency. A Wasm agent carries one typed compile-policy value from its root
through the harness and group into both initial and cache-retry worker
compilations, so a dynamically supplied worker source cannot regain the
filesystem default. Spec-exec installs Boa's rejecting loader in the root,
created-realm and agent contexts; an import evaluated there settles through its
rejection path without reading the checkout or current working directory. AOT
dynamic-source compilation remains unsupported. Ordinary engine callers retain
the default `Filesystem` policy; this contract fixes the stricter policy
specifically at differential replay's compile-option boundary.

## Typed classification

`OuterScriptModuleDependency` is the closed IR classification used at the
corpus constructor and decoder boundary. The mandatory loader policy separately
closes runtime-created requests:

| Evidence | Classification | Corpus admission |
| --- | --- | --- |
| Script parses and its retained AST has no dynamic-import site | `None` | admit |
| Script parses and its retained AST has a dynamic-import site | `RequiresModuleGraph` | reject |
| Script does not parse and the ECMAScript-aware scanner finds no possible dynamic-import site | `None` | admit as a deterministic parse-failure probe |
| Script does not parse and the scanner finds a possible site or cannot scan the source | `Indeterminate` | reject conservatively |

For successfully parsed outer source, the retained AST is authoritative. An
object or class method named `import`, and a property access such as
`object.import()`, therefore remain ordinary outer-source-closed Script syntax.
The fallback is the existing lexer-like import-call scanner, not a regular
expression: it accounts for comments, strings, templates, regular-expression
literals, dotted property names and import phases. Its answer can only admit a
parse-failure probe when no program execution is possible; uncertainty is red
at the boundary.

## In-memory invariant and stable wire

`DifferentialCase` does not retain independently mutable `goal` and `source`
fields. Construction couples them once into the private
`DifferentialProgram::DependencySealedScript` variant. Execution exhaustively
matches that program type, so adding any future program form must also add an
explicit execution path. The replay-owned compile options select `RejectAll`;
callers cannot opt a validated case back into filesystem loading. The Wasm
program-cache key includes the policy, so a filesystem-enabled artifact cannot
be reused by a reject-all replay. Its `v2` key grammar uses fixed-width
discriminators plus presence tags and little-endian `u64` byte lengths for
every variable field. Filename, target, module root, source and absent/present
graph state therefore cannot shift across field boundaries and reinterpret a
policy discriminator. The domain string versions this grammar independently of
the compiled-artifact fingerprint.

The JSON schemas remain unchanged. For every previously valid admitted Script,
field names, field order, bytes and fingerprint inputs are unchanged.
The committed v1, v2 and v3 fixtures pin that compatibility. Module and direct
dynamic-import cases are rejected while decoding. Dynamic-source constructs
may still be admitted, but any import they create meets the fixed rejecting
loader instead of an ambient filesystem.

## Future module protocol

Module replay requires a new additive protocol rather than weakening this
contract. That protocol must carry a normalized, finite module graph; stable
virtual module identities and referrer URLs; every resolution edge and request
attribute; and a fingerprint over the complete graph. Both backends must
consume that same in-memory graph without consulting the checkout or current
working directory. The exact graph schema and loader are deliberately deferred.

## Durable witnesses

IR tests distinguish true literal/computed dynamic imports from a method named
`import`, and conservatively classify malformed possible imports. Corpus tests
exercise Module, direct-import and indeterminate rejection plus method-name
acceptance under every v1/v2/v3 protocol. Direct eval, indirect eval,
Function-constructor, created-realm and agent sources are admitted under every
protocol but pin the replay compile options to `RejectAll`. The feature-gated
spec-exec witness first requires the same on-disk module body and fulfillment
handler to run under `Filesystem` in the root, direct-eval, indirect-eval,
Function-constructor, created-realm and agent contexts. It then requires the
root plus all fifteen v1/v2/v3 generated-source pairs to take only the
`RejectAll` rejection handler. Thus neither a missing/invalid file nor a broken
positive loader can make the negative gate look green. A paired AOT agent
witness likewise requires one on-disk worker dependency to load under
`Filesystem` and fail graph linking under `RejectAll`; a pure policy test pins
the root-to-worker projection independently of backend support. The exact
cross-policy filename/source tuple that collided under the retired unframed
program-cache hash is a regression input for the framed key.

## Nonclaims

Source closure and reject-all loading do not establish whole-program semantic
equivalence or ECMAScript/Test262 conformance. They do not make time,
randomness, scheduling, agents or other admitted host effects deterministic.
They do not make Wasm-AOT support dynamic source compilation: current AOT
diagnostics remain honest, while spec-exec can execute the generated source and
must reject any import it creates. Cache-key framing removes structural tuple
ambiguity before hashing; it does not claim mathematical collision-freedom for
SHA-256 or make artifacts portable across compiler fingerprints or
architectures. This slice does not add module replay, object or Symbol
comparison, panic isolation, fuzz campaigns, performance budgets, CI scheduling
or new conformance results. The existing
protocol-specific observation contracts and their
`semantic_equivalence: not_established` report value remain authoritative.
