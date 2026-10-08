# String symbol-hook operation

## Current complete source batch — 2026-10-04

The new private `string/symbol_method.rs` owner retains the original receiver
and the method obtained by one actual property Get. Its fields are private,
it is non-Copy, and its consuming dispatch is the sole callable/nullish policy
consumer. Callers cannot extract a raw method to repeat lookup or replace
IsCallable with a Function-tag shortcut. A closed optional-GetMethod versus
required-Invoke policy owns absence and native TypeError behavior.

The five named shared String entries keep their closed operation domain;
split keeps its separate standard entry and uses the same observed-method
owner. All six preserve original arguments and uncoerced String receivers
through actual callable Proxy apply. MatchAll/ReplaceAll perform the existing
IsRegExp and global-flags work before observing their hook. Primitive patterns
still bypass symbol lookup. The [current String algorithm](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.matchall)
uses one optional original hook and a required invocation on the created
RegExp after absence; null/undefined on that created receiver must throw.

The inherited matchAll own-probe/prototype retry is removed. The created
match/search/matchAll paths get their actual receiver once through the same
owner. Match's prior Proxy-aware invocation remains supported. The synthetic
receiver uses the called builtin's intrinsic RegExp prototype rather than the
mutable public RegExp constructor or the entry Realm's prototype. The source
and global flags retain their existing initialization; RegExpCreate's pattern
ToString is distinct from the constructor's source-cloning algorithm. The sole
zero-start/custom-invoked handshake is retired; independently live iterator
from-start consumers remain.

Native non-callable and global-flags errors use the current function's intrinsic
TypeError. Original getter/apply/coercion throws propagate unchanged. Existing
Engine and CLI controls cover all-six Proxy hooks, one getter, original versus
created receivers, required nullish Invoke, order, abrupt identity and both
borrowed intrinsic Realm directions. Existing domain/routing guards remain;
obsolete implementation-count and retry pins are retired without a new mirror
suite. The module inventory attaches the consumed private leaf.

This complete source batch passed the ref93 combined workspace/all-target Rust
type check. It remains unexecuted and requires emitted-Wasm validation plus
grouped focused/pinned/broad checkpoints. It does not claim all RegExpCreate initialization, descriptors,
pattern grammar, full String/RegExp closure or new published counts. Historical
verification below applies only to the preceding source.

## Historical preceding operation-domain seam

Status: implemented for `String.prototype.match`, `matchAll`, `replace`,
`replaceAll` and `search`.

## Boundary

The shared symbol-hook emitter accepts only the sibling-visible, non-copyable
`StringSymbolHookOperation::{Match, MatchAll, Replace, ReplaceAll, Search}`
domain. Standard builtin dispatch maps those five exact methods to named rows.
`String.prototype.split` dispatches directly to its separate split emitter and
cannot enter this domain.

Six borrowed exhaustive matches in the shared emitter own the well-known
symbol key, second-argument read, global-RegExp validation, `matchAll` own-hook
probe and retry, and callable-hook argument vector. The private fallback takes
the operation by reference and uses a seventh exhaustive match to select its
five existing literal/RegExp algorithms. No broad builtin ID, projected
Boolean, wildcard, default or impossible split arm remains.

## Durable evidence

`string_symbol_hook_operation_structure.rs` pins the exact domain, all seven
policy matches, semantic anchors, the private exhaustive fallback, five typed
standard producers, direct split dispatch and recursive source censuses. The
adjacent literal-replacement structure guard continues to pin the replace and
replace-all operation arms to their fixed semantic scope wrappers while the
raw scope remains private to its child owner.

## Verification

The structure target passes `4/4`, the adjacent literal-replacement guard
passes `3/3`, and the complete symbol-hook CLI fixture passes `1/1`. One exact
pinned leaf for each of `match`, `matchAll`, `replace`, `replaceAll`, `search`
and the direct `split` path passes both variants (`12/12`) with every failure
and unsupported bucket at zero. `cargo xc` is green. No semantic golden was run
because all operation choices occur while Rust emits the unchanged instruction
sequences.

## Deferrals

This invariant changes no hook lookup order, `IsRegExp`, RegExp global-flags
validation, fallback matching/replacement semantics, split semantics, dynamic
pattern compilation or broader String/RegExp conformance.

## Batch AY dispatcher boundary

The operation domain and raw emitter are now private to `string.rs`. Standard
dispatch reaches them only through five fixed String symbol-hook entries. The
frozen 306-line domain/emitter selection has SHA-256
`06636af9cd91f1e237e7cb08d47132941a9976c712a818073d1c208ce1271c26`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The symbol-hook, literal-replacement and RegExp
result-mode structure targets pass `5/5`, `3/3` and `3/3`; the complete
symbol-hook Wasm-AOT CLI fixture passes `1/1`. No Test262 leaf or Wasm golden
was required for this source-equivalent boundary, which claims no new String behavior,
conformance result or published-count change.
