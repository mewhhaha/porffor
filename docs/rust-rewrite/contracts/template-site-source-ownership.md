# Parsed template sites and source executions

Status: dry implementation; compilation and runtime verification pending.
T13/T06 remain open. Historical published counts and detail hashes are unchanged.

[ECMA-262 GetTemplateObject](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-gettemplateobject)
associates template identity with an actual parse node in the executing Realm.
A new eval or Function-constructor parse has new nodes, while repeated calls
into one constructed function retain its nodes and template objects.

`ParsedScriptIdentity` projects the retained AST/interner allocation that the
frontend already owns. Clones preserve it; equal text parsed again does not.
The joined allocation state assigns dense `TemplateSourceId` names to those
actual allocations, retaining them to prevent address reuse. Neither text,
filename, a content hash nor a process-global counter provides the identity.
A completed AST template census constructs `TemplateSourceIr`; private
`TemplateSiteId` construction checks each node against that census and supplies
its cache slot. Script, function and prepared-unit IR carry the owning plan.

The backend creates a source execution instance containing the defining Realm
and one initially empty pointer slot per template site. Main entry sources are
owned by the Realm's template registry. Each finite prepared eval or Realm
Script thunk invocation allocates a fresh instance. Ordinary, generator, async
and async-generator Function construction allocate a fresh instance in the
active constructor's Realm, after NewTarget prototype resolution. Repeated
calls to that function and functions it creates retain the same instance.
These are allocations for precompiled finite source variants; emitted Wasm
contains no parser, interpreter or VM.

A planned body-local token cannot be passed to function allocation. Entry
initialization produces the private `TemplateSourceExecution` token. Function
materialization accepts only a matching source execution; nested functions
capture it in the existing immutable function context. That record gains one
pointer field. Static field/block context allocation carries the defining source owner.
Class element execution copies the defining class's owner even
when another source invokes the constructor. Generator/async activation paths
retain the original function context through their existing function-env slots;
no duplicate owner slot is added to suspension frames.

Template expressions lazily allocate their frozen raw/cooked arrays in the
selected execution's slot and reuse it thereafter. Both prototypes are assigned
from that owner's Realm intrinsic table before either array is exposed. The
existing descriptor/freezing path preserves undefined cooked entries, indexed
property attributes, nonextensibility, length attributes and the immutable
nonenumerable raw property. Global template slots and eager Main allocation are
removed. Module guard/record global indexes now follow only the fixed globals.

The current linear record gains a Realm registry pointer at offset 64 and an
immutable function-context pointer at offset 48. Their closed layout metadata
and allocation initializers are updated. Each cache instance has source ID,
Realm and next-entry pointers followed by template-array pointer slots. This
uses the current heap representation; the separate T05 atomic Wasm-GC ABI
cutover must carry these edges when replacing that representation.

Regression sources cover independent parsed units, repeated eval and Function
construction, escaping and sibling closures, borrowed foreign-Realm functions,
Realm Script invocations, class instance/static field/block contexts, generator and async resumptions,
undefined cooked entries and frozen descriptors. None has been executed for
this packet. After the whole dry batch is ready, compile once, run these sources
and the existing template regression, then the six exact historical cache files
/ 12 modes and complete 48-mode tagged-template leaf. Report actual results;
source review and formatting provide no runtime PASS or conformance count.
