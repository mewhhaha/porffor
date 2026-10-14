# Source activation depth and observable exhaustion

Every executing source callable enters the shared runtime's checked source
activation depth before parameter initialization and before publishing legacy
reflection properties. Ordinary calls, constructors, generator body advances,
async bodies and async generator body advances use the same owner. Native
forwarding bodies and prepared eval Scripts do not independently count as source
callables. There is no source spelling, function name, syntax case or Test262 ID
in admission.

The implementation admits at most 128 simultaneously executing source bodies.
A rejected entry creates a fresh RangeError using the source callable's defining
Realm and returns a whole Throw completion. It neither executes its parameters
or body nor installs an activation which could retire its caller's state. The
runtime module declares the opaque scalar once; program modules import the same
scalar through the sealed RuntimeSchema.

Each admitted SourceActivation owns a non-copyable SourceActivationDepth with a
saved previous depth. Every existing source activation exit restores that saved
depth: ordinary return or fallthrough, whole Throw, generator initialization,
yield, resume completion, async suspension and proper tail call retirement.
Local cleanup consumes the owner only after all emitted exit branches exist.
This deliberately reuses the source activation lifecycle used by legacy caller
and arguments reflection. Suspended generator/async records retain captures but
consume no active depth. Proper tail calls retire their caller before the next
source body enters.

The existing product policy retains a 32 MiB Wasm native stack within the 64 MiB
real execution worker stack. Portable Wasm does not expose remaining native
stack space, and source body count does not measure arbitrary generated native
frames or internal helper recursion. The 128-body source resource bound is a
conservative policy, not a proof that every unusually large source/native frame
can be entered without a native trap. Do not claim general native trap recovery.
Keep the original JavaScript deadline, compiler body-size bound, Wasmtime stack
policy and test execution supervision unchanged.

The exact original pinned staging/sm/extensions/recursion.js is copied byte for
byte into the native fixture. A native control checks that identity and executes
both original recursive phases with the ordinary Test262 gc binding. Additional
controls require repeated catch recovery, all entered finalizers, captured
invocation identity across GC, parameter and constructor exhaustion, foreign
Realm RangeError prototypes, generator and async retirement, and 100000-step
proper tail calls. Retain original native and Test262 pass IDs separately from
these new IDs. The focused controls and exact original replay remain pending
until the coordinated batch compiles and executes them; formatting and patch
checks alone do not establish semantic success.
