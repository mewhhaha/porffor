# Atomic compiled-code cache publication

All FunctionCache instances in a process now share one temporary-file identity
sequence. Publication reserves a same-directory temporary file with create_new,
writes the complete entry, closes the handle, and only then renames it over the
cache key. Existing temporary files and symlinks are never opened or truncated.
A failed write or rename removes the newly reserved file. This is atomic cache
publication, not a claim of power-loss durability or a cross-process LRU lock.

The prior per-instance counter could give two concurrent cache instances the
same PID/counter filename. A rename is not sufficient protection when writers
have already shared and truncated the same temporary file.

Run the consumed-path regressions with:

```sh
cargo test --locked -p lila-engine --lib cache::
```

Coverage includes distinct instances writing the same key, deterministic stale
file collisions, cleanup after publication failure, counter exhaustion, and
Unix symlink collisions. No Test262 aggregate or pass percentage is changed.
