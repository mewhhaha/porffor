# Bytes modules

The filesystem loader accepts `with { type: "bytes" }` and preserves every
input byte, including a BOM and invalid UTF-8. Bytes requests use their own
module-map key namespace. Repeated imports share the same namespace and
default export; importing the same file as JavaScript selects a different
module record.

`ModuleSourceIr::bytes` carries host-owned provenance through graph construction
and lowering. Its synthetic initializer constructs a Uint8Array with an
immutable ArrayBuffer. Numeric-length construction and integer-indexed stores
avoid user iterators and prototype setters. Constructor, buffer getter and
immutable-transfer calls refer to exact intrinsics, so replacing global
constructors or prototype methods cannot intercept module initialization.

The canonical private-activation path and retained source-phase graph path
both preserve that provenance. Temporary bindings belong to the initializer's
own function scope. Identical source loaded as ordinary JavaScript cannot
acquire the private intrinsic markings. Duplicate host keys must agree on
both source and module type.

Program-cache identity includes every module's canonical key, URL, source and
provenance, the entry index, and canonical host resolution edges. Repointing
an import symlink must invalidate the artifact even when both possible targets
were already loaded and their contents are unchanged. The same rule preserves
entry `import.meta.url` when the entry filename is a symlink. Resolution keys
remain phase-free under `ModuleRequestsEqual`; source text retains each
phaseful occurrence.

Focused regressions are in `lila-engine`'s `aot_import_bytes` target and
`lila-ir`'s `module_instantiation` and `module_unit_structure` targets. The
upstream family is `language/import/import-bytes`. Verification of the current
batch is recorded in the [discovery checkpoint](../failure-discovery-20260926.md).
