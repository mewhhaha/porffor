# Module path identity policy

Entry canonicalization and dependency resolution now consume one independently
testable lexical normalizer. Its behavior is unchanged: first fold dot
components, then let the caller canonicalize filesystem symlinks and enforce
root confinement. In particular, `link/../dep.js` names the lexical parent's
file, even when `link` points to a directory with a different physical parent.

Physical-first normalization is not a semantics-neutral cleanup. URL-style
module resolution folds relative dot segments before resolving a file's real
path; Node documents this order in ESM_RESOLVE. Lila does not claim complete
Node resolution support, but this change preserves its existing ordering rather
than silently substituting operating-system path traversal semantics.

Six focused tests cover independence from file existence, missing components,
absolute-root preservation, lexical-versus-physical parents, surviving outside
symlinks and shared entry/dependency spellings. The exact consumed module can be
tested without compiling Wasmtime or the IR, and the full loader cohort checks
integration. No parser/interpreter is introduced.

```sh
rustc --edition=2021 --test crates/lila-engine/src/module_paths.rs -o /tmp/lila-module-path-tests
/tmp/lila-module-path-tests
cargo test --locked -p lila-engine --lib module_loader::
```

Reference: https://nodejs.org/api/esm.html#resolution-algorithm-specification

This is a regression-protection and test-isolation change, not newly implemented
JavaScript syntax or a new Test262 result. File-URL percent encoding and stronger
handle-relative filesystem isolation remain separate work.
