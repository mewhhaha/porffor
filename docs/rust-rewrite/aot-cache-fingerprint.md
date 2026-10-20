# Program-Wasm cache compiler identity

The program cache must miss whenever an input capable of changing compilation
changes. The v3 fingerprint covers the frontend, IR, AOT backend, runtime, Intl,
engine source/configuration, workspace manifests and lockfile, and patched
vendored source. It also covers build scripts and non-Rust embedded resources.
Cargo.lock alone does not identify local edits to path or patched dependencies.

Inputs are sorted and length-framed before hashing; checkout location and file
creation order do not change identity. Missing inputs and non-regular inputs
fail the build rather than silently retaining a stale identity. Generated
`target` directories and Git administrative directories are excluded.

Coverage is intentionally conservative outside an explicit list of project test
paths. Cargo integration-test trees, the IR `src/tests` tree, and six Engine
modules included only under `cfg(test)` do not participate in production source
identity. Inline tests in production files and all vendored files remain covered.
No generic filename or vendor-test exclusion is applied.

Both Engine and runtime-image build scripts use the same exclusion-aware rerun
inputs. They watch wholly production subtrees recursively, preserving addition,
removal and rename detection for scanned resource/data directories. Mixed
source/test roots watch retained children instead of their recursive ancestor.
Adding a reachable Rust module in such a root edits a watched module declaration;
new top-level build inputs require a watched manifest/build-script edit. A file
that is unreachable and unreferenced in a mixed root may not trigger an immediate
Cargo rerun; it cannot change production compilation. Moving a production input
into an excluded test tree requires changing this explicit exclusion policy.

The v3 framing and scheme remain unchanged. The policy implementation itself is
a hashed compiler input, so this policy change invalidates older compiler source
identities without permitting an old digest to alias the new input set.
Changes to other covered crate files may still invalidate program-Wasm entries
even when they are not semantic changes.
This does not invalidate the independently keyed Cranelift stencil cache and
does not claim a new Test262 result.

Validation:

```sh
cargo test --locked -p scriptw-engine --test structure -- compiler_fingerprint::
```
