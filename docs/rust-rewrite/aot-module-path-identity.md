# Physical module path identity

Entry canonicalization and dependency resolution now consume one shared path
normalizer. Existing paths are canonicalized before any lexical dot-component
folding. A request such as `link/../dep.js` must name the parent of the symlink's
target; folding `link/..` first can select a different JavaScript module or hide
an outside-root physical target behind an inside-root decoy.

The existing lexical fallback remains for virtual or missing path components.
The caller still checks confinement and file existence. This change does not
provide package resolution, URL-based imports, or race-free filesystem isolation.

Six focused tests cover ordinary canonical identity, missing-component fallback,
absolute-root preservation, symlink-parent selection, outside-root physical
identity and entry/dependency aliases. The exact consumed module can be tested
without compiling Wasmtime or the IR; the full loader cohort checks integration.

```sh
rustc --edition=2021 --test crates/lila-engine/src/module_paths.rs -o /tmp/lila-module-path-tests
/tmp/lila-module-path-tests
cargo test --locked -p lila-engine --lib module_loader::
```

No Test262 aggregate or execution denominator is changed.
