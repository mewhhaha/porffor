# Declared synchronous foreign modules

This implements the first synchronous scalar milestone of [issue #54](https://github.com/mewhhaha/scriptw/issues/54).
The compiler, persisted artifact and ordinary custom Linker path passed the
[verified checkpoint below](#verified-checkpoint-2026-10-24). This boundary targets the pinned experimental
Wasmtime GC/reference/EH capability set used by Scriptw's single Wasm backend.

An embedder can declare `dd:host` as a JavaScript module exporting `add`, backed
by a Wasm import `dd` / `add` with `(f64, f64) -> f64`. The JavaScript program is
ordinary module source:

```js
import { add } from "dd:host";
print(add(20, 22));
```

The compiler does not discover functions from a Linker, filesystem, global
object or closure table. Compilation receives a validated
`CompileOptions::foreign_modules` registry, separate from `HostSurfacePolicy`
and the closed builtin catalog. An empty registry preserves the ordinary
compiler and module-loading behavior.

## Declarations and scalar ABI v1

`scriptw_ir::foreign` exposes the validated declaration types:

```rust
let registry = ForeignModuleRegistry::new(vec![
    ForeignModuleDeclaration::new("dd:host", vec![
        ForeignFunctionDeclaration::new(
            "add", "dd", "add",
            vec![ForeignScalarType::Number, ForeignScalarType::Number],
            ForeignResultType::Number,
        )?,
    ])?,
])?;
```

Parameters accept strict JavaScript Numbers. Each wrapper evaluates its
parameter reads in order, validates every required argument before entering
the host, and passes the exact Number payload as Wasm `f64`. Missing arguments
and non-Numbers throw `TypeError`; no `ToNumber`, `valueOf` or `toString` runs.
Extra JavaScript arguments retain their normal evaluation and abrupt effects,
then are ignored by the declared native signature. The receiver is ignored.
`NaN`, infinities and negative zero remain Numbers rather than sentinel values.
Results are one Number or `ForeignResultType::Void`, which returns `undefined`.

Registry construction rejects duplicate module specifiers, duplicate export
names and duplicate or conflicting native module/name pairs. Names must be
nonempty and contain no NUL. A module declares at least one function. Native
namespaces `scriptw_host`, `scriptw_runtime` and `scriptw_foreign` are reserved.
Unsupported asynchronous declarations, parameters/results and ABI versions
are rejected when decoding persisted declarations; the Rust scalar enums
cannot represent those unsupported types.

`to_normalized_json` emits deterministic, sorted declaration bytes carrying ABI
version 1 and optional expected-error authority. `from_normalized_json` validates
the closed schema and requires those exact canonical bytes, including rejecting
unknown fields and noncanonical duplicate-key spellings. This API decodes
artifact requirements; it is not a permissive JSON configuration loader.

## Modules and first-class functions

The registry loader resolves exact declared specifiers before its fallback
loader, including when filesystem loading is disabled. Foreign modules accept
unattributed ordinary module requests. Missing modules or exports retain clear
compile/link rejections. A complete embedded catalog can add declared native
targets without filesystem access; conflicting explicit embedded resolutions
are rejected rather than silently changing authority.

Foreign modules use normal module environments, namespace bindings and
evaluation. Their compiler-owned scaffold goes through parsing, module linking
and lowering. Retained provenance replaces only the declared wrappers' exact
return operations with `ForeignCall` IR. Matching user identifiers or source
text cannot mint that operation or authorize a native import.

Each export is a strict, nonconstructable ordinary callable. Its `name` is the
declared JavaScript export and its `length` is the required parameter count.
`Function.prototype.toString` reports a native function representation, not the
private scaffold. Imports and aliases remain first-class function values:
closures, regular function arguments, `call`, `apply` and `bind` follow the
existing JavaScript call machinery. `new` rejects the call-only wrapper.

Compilation declarations are available authority. P retains only linked
foreign modules, so an unrelated declaration does not force a native import.
The normalized available registry participates in compilation cache identity;
the normalized linked requirements and ABI version persist in P metadata and
portable bundles. Source-free artifact loading validates those requirements
and actual import signatures without retaining compiler-process Rust closures.

Foreign imports, signatures and wrappers belong to P. The program index plan
places native imports after common host and imported R functions, before
program definitions. R's bytes, own indices and export names do not acquire an
application's foreign capabilities. Distinct function-index types prevent a
native import rank from being used as a runtime or program definition index.

## Embedder-owned Wasmtime instances

Rust callers must resolve the same Wasmtime crate instance as Scriptw. The
workspace pins version 47.0.0 and supplies its checked dependency patches in
[Cargo.toml](../../../Cargo.toml). An external Cargo workspace must apply that
patch set with paths pointing to this checkout; dependency patches are owned
by the top-level Cargo workspace. The runnable example uses that pinned setup.

`configure_wasm_runtime(&mut Config)` replaces the supplied configuration with
Scriptw's required capabilities and defaults. Apply custom compilation, fuel
and interruption settings **after** this call. The embedder then creates its
Wasmtime `Engine` and calls
`EmbeddedRuntime::compile(&engine, &artifact)` to admit Portable P and R plus
their persisted foreign requirements.

`EmbeddedRuntime::new_state(realm, user_data)` creates opaque Scriptw state with
typed embedder data. The caller owns the `Store`, `Linker`, resource limiter,
fuel policy and epoch deadlines. `register` installs the checked Scriptw host
functions/resources and any explicitly declared error tag. The embedder adds
its own ordinary `linker.func_wrap("dd", "add", ...)` implementation.
`instantiate` checks required native function types before R/P initialization.
`EmbeddedInstance::evaluate` consumes the one-shot entry handle, preventing
accidental replay of module evaluation through that handle.

Synchronous evaluation temporarily borrows the Store on Scriptw's scoped
stack thread and joins before returning. Host callbacks execute on that thread;
Store data is owned and `Send`. This does not provide async owning-thread
settlement or retained host-to-JavaScript invocation.

Host closures can use checked `user_data::<T>()` and `user_data_mut::<T>()`
downcasts on the current Store's state. Separate Stores retain separate host
data and Wasm Realms. Immutable compiled modules can be reused without pooling
mutable Stores or exposing Scriptw's private GC object layout. The runnable
example is [foreign_modules.rs](../../../crates/scriptw-engine/examples/foreign_modules.rs).

Expected host-operation failures require explicit registry-global authority:
`registry.with_error_tag(ForeignErrorTag::code_v1())`. ABI v1 imports only the
fixed `scriptw_foreign` / `error_v1` Wasm exception tag with one `i32` payload.
`throw_foreign_error(caller, code)` throws that actual declared exception. The
wrapper catches only that tag and produces a catchable JavaScript `Error` with
the numerical `.code`. Ordinary host errors and Wasm traps remain fatal traps;
they are not converted into successful results or catchable operation errors.
Without the tag declaration, no expected-error tag is imported or registered.

## Follow-up acceptance boundaries

This milestone does not define strings, bytes, arbitrary JavaScript references,
host-to-JavaScript callbacks or asynchronous host operations. String/byte work
needs a defined encoding, ownership, bounds, lone-surrogate and GC contract.
Callbacks need retained-instance export/invocation authority. Async services
need owning-Store-thread settlement, rooted pending values, cancellation and
job-queue/quiescence semantics. These require separate declared ABIs and
acceptance tests rather than ad hoc private-layout or integer-handle transfers.

Required verification includes registry admission/normalization, actual import
and wrapper edges, first-class JavaScript use, strict argument checks, expected
errors versus fatal traps, missing/mismatched Linker entries, two Store-local
host states, shared R across different registries and source-free bundle
round-trips. Existing empty-registry native/conformance controls remain required.

## Verified checkpoint, 2026-10-24

Source `559151e323dfba7a85d046dc0e1394d83798df69` passed **5,662 native checks** across 23 complete
affected targets. The **102 focused checks** overlap those targets and
are not additional tests. No tests were ignored. The standalone
`foreign_modules` example printed `42` after loading its Portable bundle and
linking `dd.add` through the embedder's own Wasmtime Engine, Store and Linker.

The complete pinned `language/module-code` and Function `apply`/`bind`/`call`
subsets, plus the four known failures, ran as **984 exact execution
IDs: 980 successes and the same four failures**, with zero crashes,
unsupported cases or timeouts. The actual suite exit remains 1. This is a
selected conformance checkpoint; the full 102,994-execution README chart remains
bound to historical compiler `b9ecfe64b`.

[Machine-readable verification](../foreign-modules-20261024.json) retains the
complete native inventories, commands, terminal counts, exact conformance IDs,
native snapshot, executable identity and resource receipts. Every substantial
step ran under the local kernel-enforced 4-GiB process-tree cap, zero swap, one
CPU and serial workers; no OOM kill occurred in the accepted checkpoint.
The selected replay used a payload-only 2-entry/128-MiB module retention limit
inside that unchanged kernel cap, without inferring cache-hit or live-byte counts.

[The product patch](../foreign-modules-20261024.patch) contains Cargo, Rust and
tooling changes against performance source `31c5958d2`; its checksum is in the
verification JSON. Restore that source using the
[performance evidence instructions](../performance-six-workstreams-20261024.md#evidence-and-refresh),
then apply this patch in a clean detached worktree. Documentation and evidence
are excluded from the patch. Performance timings remain bound to the earlier
frozen compiler; these checks do not retime the FFI compiler.

Focused refresh commands, run from the workspace root:

```sh
python3 scripts/limited_verification.py -- cargo test --locked --release --jobs 1 -p scriptw-ir --lib foreign::tests:: -- --test-threads=1
python3 scripts/limited_verification.py -- cargo test --locked --release --jobs 1 -p scriptw-aot-wasm --lib foreign::tests:: -- --test-threads=1
python3 scripts/limited_verification.py -- cargo test --locked --release --jobs 1 -p scriptw-aot-wasm --test emission foreign_functions:: -- --test-threads=1
python3 scripts/limited_verification.py -- cargo test --locked --release --jobs 1 -p scriptw-engine --lib module_loader::foreign_tests:: -- --test-threads=1
python3 scripts/limited_verification.py -- cargo test --locked --release --jobs 1 -p scriptw-engine --lib embedding::tests:: -- --test-threads=1
python3 scripts/limited_verification.py -- cargo test --locked --release --jobs 1 -p scriptw-engine --lib tests::foreign_capabilities:: -- --test-threads=1
python3 scripts/limited_verification.py -- cargo run --locked --release --jobs 1 -p scriptw-engine --example foreign_modules
```

For conformance, regenerate the four complete inventories with
`scriptw test262 list-all FILTER --suite-root test262/vendor/test262`, union
those IDs with the JSON's four known failures, and use
`scripts/replay-test262-executions.py` with `--workers 1 --reuse-case-workers`
under the limiter. Preserve the nonzero known-failure exit and verify the exact
ID set and native snapshot, including timeouts. Full affected native commands
and all selected IDs are retained in the JSON rather than a permissive filter
or expected-failure suppression.
