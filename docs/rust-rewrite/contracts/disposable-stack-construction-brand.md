# DisposableStack construction and brand

The GC draft validates constructor newTarget and completes the shared
GetPrototypeFromConstructor operation before allocating its ordinary header.
A private pending owner holds that selected header and a completed private
resource List. The sole finalizer consumes both, constructs the concrete
DisposableStack with pending state, and only then publishes a JavaScript value.
Prototype inheritance alone cannot forge the brand; native methods require the
actual concrete non-null GC record. The intrinsic prototype is an ordinary
object without the stack's internal slots. Proxies do not forward the brand.

Move consumes a pending-state witness and one non-Copy transfer owner. The
source's original private List is transferred intact, including resources added
by an in-flight getter after transfer. The source receives a new empty List and
becomes disposed. Destination construction uses the executing move builtin's
Realm prototype, preserving borrowed-method behavior without public lookups.

Historical linear-slot/construction checks cover the preceding representation.
The new three-control source is authored and unrun; no executable acceptance or
conformance counts are inherited. Verification follows complete all-task dry
implementation under the confirmed 4096 MiB process-tree cap.
