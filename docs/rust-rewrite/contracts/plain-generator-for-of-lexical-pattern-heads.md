# Plain-generator eager lexical-pattern for-of heads

Status: production, types, meaningful controls and documentation are authored
and independently source-reviewed. Compilation, emitted Wasm, runtime, pinned
acceptance and full T08/T15/T26 closure remain pending. Finish all remaining
task source before the capped serial verification checkpoint.

## Source and initialization authority

The existing generator for-of gate admits synchronous plain generators with
eager Let/Const array or object binding patterns and a checked yielded body.
Yield/Await in the head, suspension in the iterable, Var/assignment patterns,
for-await, async-generator ownership, resource heads and foreign control owners
remain refused. Every source statement retains parsing, lowering and real Wasm
emission; no source replacement or alternate runtime is introduced.

`ValidatedResumableSyncForOfLexicalPatternIr` is the consumed shared constructor
for Async and Generator lexical heads. It requires the actual Let/Const mode,
matching TDZ placeholder names, distinct storage names, complete iteration
environment layout and semantic BindingInitialization. An empty pattern carries
no source binding cells. Nonempty patterns require every cell even if uncaptured.
Analysis's actual Generator+Yield branch supplies that complete environment;
the earlier Async+Await branch retains its supported initialization shapes.

The generator object-pattern producer uses `ObjectDestructure` to own one
property observation before deciding whether to evaluate a default. Array
patterns retain the actual binding protocol, including nested patterns, rest,
inner iterator close and earlier-binding availability to later defaults.
The shared object-binding successor now retires the older ordinary/Async
optimized producer. Generator and Async heads consume that same canonical
initializer. Its GetV backend also preserves the original primitive getter
Receiver; see [the successor contract](object-binding-single-get.md).

## Consumed head and body

The non-Copy `GeneratorForOfLexicalPatternIr` input requires a private compiler
sink with Dynamic/all-runtime-tag reads. It rejects a spellable or persistent
name, capture/owned-environment retention, nested operand use and invalid
BindingInitialization. The mandatory iterator-plan constructor consumes it and
requires the exact initializer prefix at the start of `GeneratorForOfBodyIr`;
resumed body statements and their metadata cannot reference the sink.

Only this completed head exposes EntryLocal iterator-value storage. It supplies
the actual source mode through `head_binding_environment()` independently from
sink storage. The shared backend's existing head setup, entry-local pair and
fresh iteration environment consume these projections; it does not invent a
binding mode for the sink. Existing checked entry/body/exit states, pending
completion slots and activation layout remain the owners.

Initialization runs once after successful cached iterator stepping and value
extraction, inside the live close region and before the body lexical scope.
Resume keeps the entered iteration's cells and does not repeat Get, computed
keys, defaults or inner iteration. Continue finishes selected yielding cleanup,
retires the iteration environment and steps without close. Break/Return/Throw
wait for finalizer selection before the existing close/dispatch route.
Head initialization failures close the outer iterator while preserving Throw;
inner pattern close precedes outer close. Acquisition/step/done/value failures
retain the earlier no-close behavior.

## Authored controls

Existing generator constructor and `generator_branch_lexicals` targets gain
actual proof/lowering witnesses for wrong/missing cells or modes, Dynamic sink
typing, exact prefix/body pairing, capture/retention/refused shapes, state spans,
empty/nested/rest/computed/default patterns and body lexical shadowing.

The existing `aot_generator_for_of_continuations` target retains eighteen older
cohorts and adds three. Its paired strict/sloppy observations require WasmAot,
one compilation worker, a 30-second timeout, exact Normal Number262 and one
`ok` print per execution. These are authored expectations, not passing results:

| Fixture | Expected witness |
| --- | --- |
| `lexical-pattern-capture-interleaved.js` | One Get/default decision, computed keys, cached next across replacement, fresh captured head/body/rest cells, interleaved generators, dependent defaults and primitive String array binding. |
| `lexical-pattern-errors-close.js` | Original foreign Get/default/key throws, inner-before-outer close, empty-array close, native nullish/TDZ errors and iterator-step errors without head work or close. |
| `lexical-pattern-local-finalizers.js` | Continue/Break and explicit/injected Return/Throw across two cleanup yields, preserved mutable cells, fresh later iterations, Throw precedence over close errors and body shadowing. |

The maintained existing source guard follows the real five-field Async head
projection, shared constructor consumers, eager source gate and generator mode
accessor. Module inventory binds the reviewed backend projection and measured
actual owner budgets. No new structural mirror target is added. No tests,
compiler, guards, runtime, forced collection or status publication ran.
