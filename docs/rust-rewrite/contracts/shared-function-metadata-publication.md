# Shared function metadata publication

Native Realm bootstrap and source-function creation call one typed Wasm
`FunctionMetadataPublish` helper. Its operands are the actual completed
FunctionObject, exact length-number bits, pooled name String, metadata and
prototype flags, and the original defining Realm's selected parent prototype.
The existing closed function protocol and materialization policy select those
operands at the allocation owner. The helper does not infer a Realm from its
caller Environment or manufacture a callable execution context.

The original allocation owner still constructs ExecutableCode from its actual
typed function reference, installs immutable captures in FunctionContext,
constructs FunctionObject, and closes ACTIVE_FUNCTION before this call. Named
expressions still create their original self-cell before function allocation and
initialize that same cell with the completed function afterward. Native closure,
home object, private names, field keys and template owners remain unchanged.

The shared body publishes length then name, the original optional sloppy caller
property, and the original optional public prototype. It writes the completed
prototype cache before installing an ordinary constructor back-reference.
Generator and async-generator prototypes keep their original Realm parent and
absence of that back-reference. Bootstrap-supplied prototypes and HTMLDDA retain
their original exclusions. The Realm thrower's unconfigurable metadata becomes
complete before its ordinary header is made non-extensible.

`OrdinaryPropertyAppend` receives only the actual OrdinaryObject, checked key and
completed PropertyDescriptor. Its single original algorithm copies the retained
table in order, including holes, appends the new GC entry, and publishes the new
table before retiring temporary roots. Data and accessor callers retain their
original descriptor allocation. Runtime metadata flags use the existing checked
GC descriptor-word constructor; no raw scalar replaces a semantic GC reference.
Neither shared body calls its own facade. These operations execute no user hook
and introduce no completion or public object-model alternative.

The existing scheduling artifact control requires each helper exactly once,
actual calls from Realm bootstrap and source-function materialization, and the
metadata-to-append call. All existing 512 KiB helper/source and 3 MiB bootstrap
ceilings remain in place, with a stronger 1 MiB ceiling across every body in the
fixture. It also reports the Array.fromAsync Fulfilled body so
the combined checkpoint can attribute changes outside the source driver.
The first metadata/property extraction passed the parent's artifact controls,
but native compilation still exceeded the enforced memory limit. The additional
[shared header projection](shared-object-header-projection.md), ordinary
allocation and explicit-Realm ToObject source changes require a new capped
checkpoint. Reduced Cranelift memory and semantic acceptance remain unproven.
