# Exact native Test262 cohorts

`lila test262 run-list <execution-list> [options]` runs a nonempty list of exact
`mode:path.js` identities through the normal supervised conformance runner.
Blank lines and whole-line comments are allowed; duplicates, unknown modes and
empty lists reject. The private checked `TestExecutionList` cannot contain an
empty or duplicate selection. Every requested identity must exist in the actual
discovered manifest, including its mode. One missing identity rejects the whole
native selection before any execution or partial passing snapshot.

The selection retains discovered case order, original source/harness/flags,
suite pins and exact identity-based manifest hashing. It cannot also request a
filter, shard or matrix ceiling. Existing worker-count/deadline rules, bounded
64-request reuse and fresh Engine/Realm/Store/GC/agent state remain owned by the
native runner. Native resume validates this selected manifest and actual image;
completed failures stay failures. A selected cohort is not a full pinned matrix
and cannot satisfy the two-fresh-full-run release gate or canonical publisher.

`test262 list-all` prints the complete native discovered inventory, with its
exact count and every mode/path. The ordinary `list` display remains bounded.
Discovery gives selection information only; every admitted execution still
reaches the actual compiler/runtime and can fail or be unsupported.

The optional `--reuse-case-workers` mode of
`scripts/replay-test262-executions.py` freezes the original input and actual
executing image. It partitions against that complete native inventory and runs
present identities in one `run-list` invocation. Every absent file or absent
mode also gets its original exact native CLI invocation and retains its explicit
infrastructure error in the original denominator. No absent/unsupported ID is
removed or represented as a pass. The default per-ID replay remains available.

The native snapshot supplies per-ID outcomes. The adapter checks schema, exact
completed IDs, compiler, backend, pins, counts, failure/timeout identities and
native exit status before sealing a result. It preserves raw commands/logs,
fixture byte hashes and snapshot hashes; hashes and snapshot parsing happen
once per batch. Resume rejects changed input, image, mode, worker count,
fixture bytes, partition, transcript or sealed snapshot, and evidence paths
outside the invocation. Interrupted native runs retain their original checkpoint
and resume the same selected cohort. Tooling controls use fake CLIs and do not
replace actual native/pinned execution acceptance.

This coherent batch is authored. Python admission/resume controls pass; native
compilation, full affected CLI/harness scopes, real interruption/resume,
historical 5,365-ID replay and before/after timing remain pending.
