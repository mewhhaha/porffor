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

Native code sharing also removes duplicate defined JavaScript bodies after all
payload and literal rewrites. Eligibility requires the checked stateless owned
path, equal concrete function type index, equal locals/instructions and equal
actual-P local/initialization witness. Bootstrap, imported R and helper/factory
entries cannot alias. Unknown custom relocation/debug formats preserve the
original module. One closed mapping updates function declarations, bodies,
calls, references, exports, elements and function/local/label name owners.

This mapping never rewrites numeric declaration identities. Each independently
allocated FunctionObject, ExecutableCode and FunctionContext retains its original
SOURCE_ID, constructor element plan, source text, name and captured environment.
Only the Wasm executable entry is shared. Raw public artifacts retain every
original function and index. Structural size controls, native identity/class
plan controls and original CLI deadlines remain separate acceptance gates.

Runtime and program type declarations may share immutable encoded type sections
and assigned layout indices. Their two declaration graphs stay distinct. Every
module still owns fresh globals and prelude state, and finalization consumes its
own matching type/global package once. No source data or mutable Wasm state is
shared through this declaration storage.

Template planning collects actual type/signature/export facts during its header
pass and checks ownership against the same decoded operator slice used to plan
each body. The complete owner witness is constructed only after every body and
payload site has been checked. Fusing these passes does not relax actual-field,
initializer, owner-local, readiness or relocation requirements, nor raw Wasm
validation. Original-deadline native verification and exact emitted-byte controls
are required independently of metadata checks.

## Proven empty direct eval

After validating the actual caller/source witness, the host still parses every
UTF-16 request under its actual caller grammar and completes normal lowering.
Only `RuntimeDirectEvalProgram::empty_completion` can mint an
`EmptyRuntimeDirectEval` borrow. It requires a genuinely empty parsed statement
list and an empty executable direct-eval root, with no source declarations,
observable bindings, functions, nested prepared sources or lexical block. The
existing intrinsic lookup catalog and exact unspellable immutable caller-context
capture are dead metadata; arbitrary captures or source initializers are rejected.
Empty-looking declaration nodes and directive prologues do not qualify.

This proof allows dead allocation elimination: an empty source creates no
observable function, closure, template site or value that could retain P or its
source/environment. Its only executable operation is normal undefined
completion. `NativeProgramTemplate::for_empty_direct_eval` emits that completion
as a real stateless Wasm function, including the normal completion/Entered ABI.
The host calls this function; it does not synthesize JavaScript completion.
The template has no imports, globals, memories, tables, data, elements or start.

The native-source context retains one such immutable template under its exact
validated R owner. Its actual Store Instance uses the existing bounded cache,
full code equality, actual Wasmtime Engine and context authority. The original
execution deadline covers parsing, lowering, lookup, invocation and reply.
Every request still gets its own syntax decision. In particular, LF, CR, LS
and PS can terminate a line comment and expose executable code, which must use
the ordinary source-owned path. Invalid comments remain parser SyntaxErrors.
This optimization uses no fixture names, source-text patterns or changed limits.
