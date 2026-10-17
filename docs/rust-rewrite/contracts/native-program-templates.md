# Native program code and data

Native Function, global Script and direct eval still parse their exact source,
perform early errors, lower new IR and emit real Wasm for each admitted request.
The public AOT artifacts remain self-contained. Native execution can derive an
opaque code/data template only from one of those typed emitted artifacts.

The transform separates passive packed data and adjacent constant operands of
packed `array.new_fixed` instructions into immutable per-program byte images.
It replaces each allocation instruction with a typed constructor call at the
same execution position. The constructor allocates a fresh array on every call,
including for mutable arrays, preserves packed UTF16 bits and byte offsets, and
retains array bounds traps. It does not import a previously allocated result.
Unsupported data operators use the unchanged self-contained native module.

Unused function imports are pruned through an actual reference scan. Code,
exports, globals, element references and standard function/local/label name
maps use the same index mapping. Original GC type indices stay unchanged; the
new byte-image and constructor types are appended. Module start functions
remain forbidden in native P artifacts.

The native binder checks the actual Wasmtime Engine and exact immutable R Arc,
resolves R/host imports in the original Store, and creates fresh data and a fresh
instance. Calling program, defining Realm, caller records, exact UTF16 witness,
source metadata, GC roots, initialization and factory signatures keep their
existing ownership checks. All phases and constructor calls use the original
armed execution deadline. Template failures are compiler/transport errors and
cannot become JavaScript syntax errors or unsupported-language answers.

Compiled template code uses the existing aggregate module-cache limits and
actual compiled-image accounting. Its byte/mode key also requires the actual
Engine identity. A small execution-context cache retains at most eight typed
constructor functions; they capture types and execution control, never source
bytes, mutable selector state, or result arrays.

The source component has authored transform, GC allocation, cache and real AOT
regressions. They have not been compiled or run. The earlier 96-request phase
trace is a baseline measurement; this component establishes no speedup or
conformance result.
