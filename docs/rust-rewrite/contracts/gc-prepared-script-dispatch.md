# Prepared global Script GC dispatch

The 2026-10-07 workspace check passes. Paired native indirect-eval completion/
Realm restoration and fresh lexical-cell controls pass, along with the nested
direct-eval context control. Realm Script conversion exposes an admission gap
for a known Symbol: ToString must throw before source dispatch. Its typed repair
passes the full IR suite and the paired native conversion control. Runs retain the
confirmed aggregate 4096 MiB cap, swap disabled and one worker.

A closed GlobalPreparedScriptKind selects indirect eval or Realm Script.
Direct eval cannot enter this owner: its existing caller-context operation owns
lexical and private environments, this, and NewTarget. The global path compares
actual GC String contents with the finite compiler-owned source table and calls
the independently compiled prepared Script entry. No parser, interpreter or VM
is emitted. Unmatched runtime source retains its explicit typed Wasm-AOT
unsupported operation.

The owned execution record roots the captured callable Realm, saved caller
Realm, global environment, global this and empty private/direct-eval contexts.
It supplies only the actual typed PreparedScriptInputs ABI. Both compiled
completion and deferred SyntaxError stay whole until the record restores the
caller Realm. SyntaxError uses the saved target Realm intrinsic. Module preludes
consume the same global execution owner from the current execution Realm,
propagate original Throw only after restoration, and discard Normal results as
required by the containing module entry.

Indirect eval returns every non-String argument unchanged, without observable
conversion, including boxed Strings. Realm Script performs exactly one ordinary
ToString through the existing whole-completion conversion owner before dispatch.
That owner retains the original conversion Throw and the actual callable Realm
for generated errors. Public global replacement does not change a captured
callable's target Realm.

A private `ProvenRealmScriptConversionThrow` can be constructed only for a
non-spread first argument whose sole possible kind is Symbol. Exhaustive call
resolution retains the actual host invocation and its captured-Realm error,
without inventing a prepared source. Unknown String sources keep their existing
typed AOT rejection. Finite object-coercion candidates still execute their actual
hooks before guarded prepared-source selection.

Four paired strict/sloppy controls cover non-String identity, foreign Script
completions and errors, conversion ordering and captured Realm, and fresh
escaped lexical cells. Non-String identity now passes both modes. The conversion
fixture captures its Symbol in a const before effectful calls, so mutable global
Symbol lookup does not turn the intended conversion-error input into unknown
source. The complete fixture passes both native modes in 246.96 seconds;
all four controls now have paired native passes across the scoped checkpoints.
Existing finite prepared Script controls remain
the broader acceptance source. No full task acceptance or published conformance
count is inferred from these focused results.
