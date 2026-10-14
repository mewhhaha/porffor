# Source activation completion exits

An admitted source callable owns its actual FunctionObject, source depth and
saved optional caller/arguments values. Its ordinary completion return must
restore that owner exactly once, regardless of how many observable operations
can throw inside its body.

CompletionExit admits a source epilogue only when supplied the non-constructible
SourceActivation owner. One empty-result Wasm block surrounds parameter/body
execution and fallthrough result normalization. Explicit Return and uncaught
Throw retain their complete completion, retire held References and statement
values, unwind their actual lexical environment depth, and branch past
fallthrough normalization to the single activation-restoration epilogue.

Caught Throws continue to their real handler and keep the activation live.
Initialization and Yield/Await exits preserve their entire completion, including
async resume targets, and use the same sink. Normal suspension retains saved
Reference and loop/iterator cells; only Return/Throw retires abandoned statement
state. The completion branch performs lexical unwind directly instead of the
general statement-branch retirement. Proper tail calls retain explicit retirement
before target dispatch. Native
and helper bodies have no admitted SourceActivation, keep their direct return
ABI, and retain the existing actual NativeCallerBoundary behavior. Prepared
Scripts keep their separate source-transparent execution policy.

The optimization removes duplicate source restoration from generated operation
error leaves; it does not suppress an operation, alter an observable getter or
Proxy trap, cache a With lookup, change callable identity or raise a code-size
limit. The existing bounded With-growth test and shared-helper call-edge checks
remain unchanged. Native controls cover first/middle/last getter throws,
complete returned argument objects, actual getter/inner/outer caller identities
and inactive reflection restoration.

The 750c4d74c source-bound AOT library run measured nine With reads growing the
probe from 5,599 to 17,668 bytes (12,069-byte growth), above its unchanged 9,000
byte gate. Post-repair size and native acceptance belong to the subsequent
actual verification owner.

A source-bound partial diagnostic compiles only the candidate AOT epilogue
against the unchanged750 optimized Front/IR/library artifacts. It validates the
three program modules, measuring4588/5383/11743 bytes at1/2/10 reads and the same
87 locals:7155-byte incremental growth, below the unchanged9000-byte gate.
The candidate AOT library compiles in96.349 seconds and its fresh stripped
diagnostic links in1.072 seconds; emission/validation takes1.521 seconds.
No JavaScript executes. Full successor native semantics and pass retention
remain mandatory; the diagnostic is not native Wasm acceptance.
