# String invocation References, result facts and synchronous effects

Status: the complete bounded source batch is authored after the ref105 passing
Rust checkpoint. This successor needs its grouped all-target check, emitted-Wasm
and runtime controls, broad/pinned acceptance and publication. Full T18, T19 and
T26 remain open.

## Acquired function and raw receiver

Thirty-three selected String targets now use the existing acquired-callee owner:
Substr; CharAt, CharCodeAt, CodePointAt, At, PadStart, PadEnd, Repeat, Normalize,
LocaleCompare, ToLocaleLowerCase, ToLocaleUpperCase, ToLowerCase, ToUpperCase,
IsWellFormed, ToWellFormed; the thirteen HTML methods; Trim, TrimStart, TrimEnd;
and Split. TrimLeft/TrimRight retain their existing TrimStart/TrimEnd identities.
The primitive String CharCodeAt and Split source gates are also retired.

Native target knowledge describes the function acquired from the original
property. It cannot replace a transferred alias with a canonical property name.
The existing PropertyRead/GetV, explicit-method argument owner and indirect
publication retain that function, the once-materialized raw receiver and the
complete real argument vector. Acquisition precedes arguments; invocation
precedes receiver and argument coercions inside the selected native body.
Ignored extra operands retain evaluation, and spread retains its actual iterator
record, cached next and abrupt completion. Callable Proxies use general Call.

The final Wasm exact Match/Split/Slice interception was already retired in the
[preceding batch](invocation-shortcut-retirement.md). This successor needs no
new backend adapter, opcode, classifier or runtime representation. Existing
native algorithms and their defining-Realm errors remain their implementation
owners.

## Truthful result facts

String MatchAll joins the existing arbitrary-result group in both live builtin
call analysis and the spread-aware signature. An observed Symbol.matchAll hook
can return any normal ECMAScript value. Match, Replace, ReplaceAll, Search and
Split already retain that same all-runtime-tag domain. These results claim no
invented shape or exact function target. The primitive Split Array-only claim
disappears with its early source gate.

The signature consumed by real spread also corrects CharCodeAt from String to
Number, and CodePointAt from String to Number-or-Undefined. This matches their
existing live result factory; CodePointAt cannot claim an in-range result solely
from source syntax. Other String result domains retain their actual owners.

## Mandatory generic effects

The actual catalog has forty-nine String prototype rows. Only ToString and
ValueOf are nongeneric internal-brand checks. Every other row can observe user
code through a borrowed receiver, symbol hook or argument coercion. All
forty-seven carry SYNCHRONOUS_USER_CODE: forty-two flags are added, and five
existing synchronous rows retain their flags, including LocaleCompare's
INTL_HOST bit.

The existing consumed const catalog validation loop now requires this flag for
all generic String prototype rows. A later generic row without the flag fails
Rust compilation. Existing native-name validation remains in the same loop;
IDs, ordinals, installers and native names retain their values. Direct, indirect
and spread-aware analysis consume the same flags and existing invocation-effect
owner, so captured kind, shape and element facts cannot survive a synchronous
coercion or hook on the assumption that it is pure. Concat consumes this same
effect authority through its previously corrected invocation Reference.

## Meaningful controls

Independent literal lowering cases exercise all thirty-three native aliases
with poisoned canonical properties. They check the original acquired key and
native identity, matching receiver storage and truthful result domains. Further
cases retain argument replacement, real spread and ignored operands; numeric
CharCodeAt/CodePointAt signatures; all six arbitrary hook result roles; and
captured-flow invalidation. The maintained intrinsic-method control now requires
the real acquired CharCodeAt Reference and Number result rather than its retired
inline form.

Three finite Engine cohorts cover References/arguments, hook results/effects and
Realms/abrupt completion. They use paired strict/sloppy WasmAot with the Test262
host, one compilation worker, a 30,000 ms run bound, exact Normal Number262 and
one exact cohort print. They retain all thirty-three literal native cases,
callable Proxy dispatch, cached-next spread mutation, acquisition before
prototype replacement, arbitrary Number/Function/Symbol hook values, captured
kind/shape effects, both borrowed error Realm directions and original foreign
throws with precise cutoffs. Nullish Split.call evaluates all supplied operands
before native rejection. These authored expectations require execution.

## Remaining boundaries

The complete [Number hook and dispatch successor](number-string-hook-and-dispatch-retirement.md)
retires the primitive borrowed Match/Split gates, their copied tracking and sole
flag-setting recognizer. It also removes the unproduced StringCharCodeAt IR form,
its sole emitter and HTML-name-only metadata, preserving live native bodies and
this generic-effect invariant. Both successors remain type/runtime unverified
until the full-task source pass reaches its capped verification checkpoint.
The complete [remaining invocation successor](remaining-invocation-reference-ownership.md)
retires ArrayIteratorNext/TypedArray.from/of canonical calls, the ArrayBuffer
species forwarding bypass and both literal folds, preserving actual forwarding
analysis and fixing coupled facts/effects. Optional/suspended calls, broader
Function protocol work and the wider flag catalog remain separate owners.

This is invocation and fact ownership, rather than full native algorithm
acceptance. Locale case controls use default ASCII only; complete locale
selection/case mapping remains open. RegExpCreate initialization, pattern
grammar, descriptors, all String algorithms, executable controls and fresh
pinned acceptance remain required. No skip, interpreter or conformance count
change is introduced.
