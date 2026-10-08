# Shared async-generator scheduling

Async-generator body entry, completed-generator queue draining and Promise job
draining each have one registered typed Wasm helper body. Plain async Await
reaction setup and the four async-generator Await continuations have shared
bodies too. Source functions,
generator methods and Promise reactions call those bodies. A nested Array
pattern still owns its original synchronous iterator and IteratorClose; its
suspension and queued request semantics do not require copies of the scheduler.

`AsyncGeneratorStartBody` receives the actual non-null activation, complete
incoming completion and trusted caller Environment. The completion operand owns
all five ABI parts together, including the original object reference and branch
target. The original body-start algorithm retains the previous active Realm,
calls the activation's saved body and preserves the incoming completion on exit.
Its completed paths call `AsyncGeneratorDrainQueue` rather than emitting that
queue algorithm again.

`AsyncGeneratorDrainQueue` receives the same activation and caller Environment.
Its original FIFO loop consumes actual retained requests, settles their
capabilities and suspends a Return request through its original Await reactions.
It returns the original Normal completion reset. Queue ownership, active request
identity and activation roots remain in the existing GC records.

`PromiseDrainJobs` receives the complete incoming completion and caller
Environment. The original job loop saves and restores that completion, the
active Realm and all three throw-diagnostic roots. Each job installs its retained
Realm before its existing reaction or thenable dispatch. Async-generator Await
and Return materialization uses the activation's explicit execution Realm, not
the helper's nonexistent callable FunctionContext or the caller Environment as a
substitute Realm.

`AsyncAwaitReactions` receives the actual async activation, awaited value and
caller Environment. Its original setup first publishes that Environment into
the activation's retained InvocationFrame, then obtains the execution Realm
from the activation. The one original PromiseResolve and reaction-pair algorithm
returns its complete synchronous failure to the source Await owner before that
owner commits suspension. Each Await emits a typed call rather than another
copy of that setup; module-import Await keeps its existing dedicated path.

`AsyncGeneratorAwaitReactions`, `AsyncGeneratorYieldReactions` and
`AsyncGeneratorYieldReturnReactions` fix their original callback kind in their
typed declaration and compile the same original Await algorithm. The separate
`AsyncGeneratorAwaitReturnReactions` preserves the original Return-specific
algorithm and writes its whole completion into the supplied result; it does not
replace the caller's current completion. All four derive PromiseResolve authority
from the retained generator activation. StartBody and DrainQueue call these
bodies instead of repeating PromiseResolve and reaction initialization branches.

The emitted-size regression compiles the original nested Array/default,
queued Normal/Return/Throw and awaited/yielding-finally family through the real
parser, lowerer and Wasm emitter. It checks that the eight helpers have one
emitted body and real external direct-call consumers, and bounds their encoded
bodies and the request-driving function at 512 KiB and bootstrap at 3 MiB. These
are regression ceilings, not measured baselines. The existing runtime lifecycle
cohorts remain the semantic execution checks. Source authoring alone does not
establish type, size or runtime acceptance.

The same fixture also consumes the registered conversion boundaries described
in [shared conversion bodies](shared-coercion-helpers.md). Property reads call
the shared PropertyKey body, which calls the shared String-hinted ToPrimitive
body. Other conversions use their own fixed hint. These calls prevent repeated
coercion lookup and object-descriptor dispatch from growing each source function
after its scheduling operations have already been shared.

Native bootstrap and source-created function metadata also use the original
shared [metadata and property publication](shared-function-metadata-publication.md)
bodies. The same artifact control checks their actual calls and reports the
Array.fromAsync Fulfilled native body without changing any size ceiling.

The complete fixture also requires every emitted body at or below 1 MiB before
a native retry, while retaining its stronger 512 KiB scheduler/helper/source
limits. It prints the largest bodies before asserting these bounds. All generic
object-header consumers use the
[shared exhaustive projection](shared-object-header-projection.md).
