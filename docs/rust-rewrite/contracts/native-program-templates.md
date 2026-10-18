# Native program code and data

Native Function, global Script and direct eval parse their exact source,
perform early errors, lower new IR and emit real Wasm for each admitted request.
Public AOT artifacts remain self-contained. Native execution derives an opaque
code/data template only from a typed emitted artifact.

The transform separates passive packed data and adjacent constant operands of
packed `array.new_fixed` instructions into immutable byte images. It replaces
each allocation instruction with a typed constructor call at the same execution
position. Every call allocates a fresh array, including mutable arrays, preserves
packed UTF16 bits and odd byte offsets, and retains bounds traps. It never
imports an already allocated result. Unsupported data operators retain the
self-contained module and fresh native Instance path.

ABI10 adds an immutable source-payload reference to each fresh ProgramInstance.
Its concrete GC array type has physically immutable packed I8 elements. The
payload contains images, checked image ranges and literal words; the original
program-metadata field and logical image bounds retain their separate meanings.
Directories and the versioned footer cannot become readable source-image bytes.
The raw public initializer remains `[] -> P`. Only a checked native template
initializer receives the actual payload as an explicit typed argument.

The emitter annotates each source-sensitive function with its actual owner P
local after binding the calling or defining program. The transform validates
the concrete P and immutable payload field types, bootstrap export/signature,
non-parameter owner-local index/type and initialization boundary, and the order
of all source/literal sites. Bootstrap signature rewrites reject old-signature
callers. Invalid ownership witnesses are compiler errors. Physical Wasm owner
locals may be nullable; initialized Rust GC handles and emitted binding order
provide the live owner at every admitted read.

Source allocations and literal reads load the payload from that actual P.
Saved functions and closures therefore retain their defining program's data
through later compilations, nested callbacks and GC. There is no ambient
"current payload" selector. Initializers allocate fresh payload arrays and fresh
ProgramInstances even when their native Instance is reused.

Unused function imports are pruned through an actual reference scan. Code,
exports, globals, element references and standard function/local/label name maps
use the same index mapping. Original GC type indices remain unchanged; helper
types are appended. Private ownership annotations are consumed. Module start
functions remain forbidden in native P artifacts.

Checked stateless program-owned templates give defined JavaScript functions
stable `js::native_function_<index>` Wasmtime symbols. A shared native body must
not report the name of the first source to populate the cache. Public artifacts
retain their original source-specific Wasm symbols; JavaScript function names,
self bindings and `toString()` still use the actual/defining program's data.
Low-level native trap reports identify the shared code index rather than a
source-specific function name. Other templates retain their original names.

Instance reuse is restricted to checked program-owned templates with no defined
globals, tables, memories or active/passive element state. Stateful data
operators are ineligible. Imported R state retains the same shared lifetime as
the former fresh-Instance path. The cache belongs to the actual Store and keys
on mode, digest, full immutable template bytes, the actual Wasmtime Engine and
an unforgeable native-source-context identity. Hash equality alone never admits
reuse, and another context's linker/import authority cannot be substituted.

Both compiled templates and Store-owned Instances use the existing aggregate
module-cache limits and actual compiled-image accounting. Eviction limits
retained entries; it does not claim Wasmtime frees a live Store's Instance slots.
The repeated same-template path must therefore demonstrate reuse through the
original 10,000-Instance limit. A separate allocation-context cache retains at
most eight typed constructors, capturing types and execution control rather
than source bytes, mutable selectors or result arrays.

All phases and constructor calls use the original armed execution deadline.
Template failures remain compiler/transport errors, never JavaScript syntax
errors or unsupported-language answers. ABI9 artifacts and runtime packages
cannot pass ABI10 admission. Diagnostic type/emission/native checks, ordinary
Cargo tests and actual release-CLI conformance each retain their own source
identities and acceptance scope; one does not substitute for another.
