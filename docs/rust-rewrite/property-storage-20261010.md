# Geometric property storage — 2026-10-10

Ordinary named properties now have a GC-owned storage record with separate
ordered entries and logical insertion extent. Capacity grows geometrically;
a typed `array.copy` copies the rooted prefix. Existing descriptor identities,
deleted holes, String/Symbol insertion order and integer-key ordering survive
growth. Rooted snapshots traverse only the logical extent. Arguments indexed
descriptors use geometric capacity too, with nullable slots representing absence
and an independently bounded parameter map. Named-property lookup remains linear.

The [machine-readable receipt](property-storage-20261010.json) records the base
commit, exact source and executable hashes, commands, logs and outcomes.

- All-feature/all-target workspace types pass before and after verification.
- All 969 unique backend controls pass. The first sweep exposed one stale source
  assertion for lexical loop binding modes; the lowering already preserved those
  modes. The corrected assertion and remaining targets pass. Only that assertion
  changed between the two recorded source freezes; duplicate passes are excluded.
- All 44 focused native controls pass: ordinary objects (8), sparse arrays (6),
  Arguments (9), rooted snapshots (4), global references (9), HasProperty (5) and
  Proxy descriptors (3). These include growth inside a getter with GC, property
  attributes, deletion/re-addition, Symbols, numeric order and spare-slot absence.
- Fresh product fake acceptance passes all 191 exact execution IDs over 190
  fixture files, including all 187 Wasm-safe IDs, with no failures or timeouts.
  Snapshot version, backend, outcome counts, exact membership and executing-image
  identity are checked. Four isolated cases, each with one compiler job, complete
  in 209.105 watched seconds; the 60,000-ms per-case limit remains unchanged.
- Formatting, module boundaries, task-plan, shortcut accounting and whitespace
  guards pass. The committed README publication guard runs before pushing.

Verification uses the inherited 32 GiB cloud memory cap and four-CPU quota with
serial Cargo/libtest and two retained modules bounded to 256 MiB. Existing native
case deadlines remain unchanged. Eight completed prior-checkpoint test images
were removed to recover 5,006,358,160 bytes; current/baseline CLI images, source,
libraries, metadata and logs were retained.

The fixed diagnostic creates 4,096 ordinary properties and Arguments indices in
three rounds, checking values, absence, key counts and deletion/re-addition.
Execution measures 110.275 seconds for the original `08762956b` executable and
105.292 seconds for this candidate. Compilation is reported separately in the
trace. These single samples establish correct execution for that input and do
not establish a general speedup. An attempted smaller diagnostic never executed
because its input file did not yet exist; its failed command is preserved.

The prior complete engine (3,184 passes) and CLI (943 passes, four existing
ignores) checkpoints apply to their recorded earlier source. This patch has the
focused native, complete backend and product fake coverage above, not a fresh
complete engine/CLI sweep. The missing September 30 aggregate of 5,365 exact
failure IDs remains unavailable. Task states and publisher-generated pinned
conformance totals are unchanged.

Raw logs, source maps, diagnostic input and private drivers remain under ignored
`target/property-growth-20261010/` and `target/watched/`. Their identities and
verification results are retained in the checked-in receipt.
