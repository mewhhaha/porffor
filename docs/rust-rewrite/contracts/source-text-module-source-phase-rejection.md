# Source Text Module source-phase rejection

The original T12 correction admits `LoadedModuleKind::Source(String)` as genuine
ECMAScript Source Text Module Records. The later JSON batch adds
`LoadedModuleKind::Json(String)` and strict parsed synthetic default-export
records; [native JSON evaluation](json-module-native-default-export.md) shares
canonical environments without providing a proposal source representation.
Both current record kinds have no module-source representation. Resolved static
source bindings and forwarded source exports reject during linking with
`ModuleSourceUnavailable`/SyntaxError. Dynamic `import.source` rejects with a
fresh native Realm SyntaxError after its target loads/parses, before loading
that target's dependencies or evaluating its body. Existing missing-target,
options/coercion and parse/early-error routes remain primary.

The normative source-phase rules are from the separate [Source Phase Imports
Stage 3 draft, March 25, 2026](https://tc39.es/proposal-source-phase-imports/),
particularly `ContinueDynamicImport`, `ContinueModuleLoading`, `ResolveExport`
and `InitializeEnvironment`. The proposal initializes ECMAScript Source Text
Module Records' `[[ModuleSource]]` to empty (no source representation); source
loading visits one target record, while other phases load its dependencies. Static source initialization checks the resolved
record; dynamic source continuation rejects before `LoadRequestedModules`.
This proposal is distinct from [ECMAScript
2026](https://tc39.es/ecma262/2026/multipage/ecmascript-language-scripts-and-modules.html)
and the newer [ECMAScript Module Phase Imports
proposal](https://tc39.es/proposal-esm-phase-imports/), whose ModuleSourceRecord
model is not silently adopted here.

Actual ownership crosses three existing boundaries. `ModuleSourceIr` exposes a
phaseful loading projection from the retained AST; its rows coalesce by the
same phase-free host key, promoting a source-only occurrence when a non-source
occurrence also needs the target. `ImportPhaseIr::loads_dependencies` owns the
exhaustive loading policy consumed by that projection, the actual loader and
static admission closure. The loader loads/parses source-only targets once,
then opens a cached target exactly once if Evaluation/Defer later needs it.

Admission retains each target's real parse outcome. Source-only jobs are
rejected without admitting their target or linking its children. A target that
also belongs to an ordinary/deferred closure keeps its canonical activation;
the Source dispatcher still rejects before any evaluator, namespace cell or
continuation. It uses the same trusted intrinsic constructor boundary as
existing load failures, so mutable global Promise/SyntaxError properties cannot
replace its native identities. The dispatcher preserves specifier/options
operand order and synchronous coercion/abrupt-value identity.

`ModuleInstantiationGraph` owns the actual link-diagnostic rejection before
minting the private canonical source-assembly owner. All successful Module and
Script graphs use the existing activation graph: the old eligibility had only
Source exclusions, with no async/TLA/defer/cycle/Script exclusion. The former
merged driver, retained entry variant, synchronous entry-kind variant and
corresponding backend arms are removed. Default export declaration identity,
NamedEvaluation and original callable source still use their consumed canonical
owners. Namespace and dynamic source-inspection APIs remain; they construct no
fake module source object. The old ordinary null-prototype tag lookalike and its
storage-name role, aliases and public tag constant are removed.

Meaningful witnesses are authored in the existing module/Script AOT targets and
real filesystem loading owner: fresh intrinsic errors and Promises under global
replacement; ordered operands/options and exact coercion throw; source rejection
before dependency-link reactions; static/forwarded source failures; missing and
parse-failing target categories; private module context; Script globals and
completion; source-only loading without invalid/missing children; cached target
promotion; duplicate-row promotion retaining a child parse failure supplied by
another agreeing host row; and deferred Get followed by ordinary import evaluating
once. Existing IR semantic expectations and affected source-owner assertions are maintained.
No new structure-test framework or mirroring suite is added.

The native [`%AbstractModuleSource%` intrinsic](native-abstract-module-source-intrinsic.md)
is implemented, including its prototype, native tag getter and defining-Realm
intrinsic slots. Supported Source Text and JSON records still have empty
`[[ModuleSource]]`; neither loader invents a source object. A positive concrete
source loader is an optional host extension, not a missing acceptance condition
of this pin. The proposal permits a record kind without a source representation,
and [`HostGetModuleSourceModuleRecord`](https://tc39.es/proposal-source-phase-imports/#sec-hostgetmodulesourcemodulerecord)
defaults to `not-a-source`.

The current Unicode namespace regression retains two distinct Unicode namespace
alias spellings for one canonical module. Its source-alias child retains both
Unicode source bindings for the same JavaScript target, but now requires the
actual SyntaxError from linking instead of an invented null-prototype source
object. Two dynamic imports reject, and explicit counters prove neither the
source-binding child nor the source-only target body evaluated. The ordinary
namespace remains a single non-extensible null-prototype object. These corrected
native assertions are prepared for the joined executable checkpoint; they have
not run during the frozen sweep.

The current product graph consumes rooted GC ModuleRecord and ModuleRegistry
owners through the semantic GC schemas; record-state reads take rooted owners.
Full module acceptance and joined executable verification remain open. Any future
positive module-source loader/object facility must use the same semantic object
model, without an integer-to-GC bridge or a parallel graph representation.

No compile, JS parser invocation, tests, runtime, Wasm validator, Node oracle,
generator or status refresh ran for the original dry packet. Rustfmt/source/hash/apply checks
are syntax and transport checks only. Mandatory focused compilation and module
regressions, then the broad verification checkpoint and pinned proposal witnesses,
remain for the authorized executable-verification phase. The staging pinned
`import-source-source-text-module.js` expects SyntaxError for existing JS targets.
The language `module-code/source-phase-import/import-source.js` fixture instead
uses missing targets and expects a host-loading error; it is not evidence for a
successful source object or the same rejection category.
