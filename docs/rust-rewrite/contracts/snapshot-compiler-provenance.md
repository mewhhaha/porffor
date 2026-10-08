# Mandatory compiler provenance for execution evidence

Status: source is authored. No compilation, test, native
CLI execution, source guard or kernel-limiter bootstrap has run for this batch.
All remaining task source must precede serial verification under a confirmed
4096 MiB process-tree kernel cap with zero swap. T01, T03 and T26 remain open.

## Build and executing-image identity

The Engine build script embeds the existing compiler-input fingerprint, its
single scheme authority and the actual Git HEAD object identity. The fingerprint
includes dirty compiler input bytes; the commit identifies their checkout base,
not a claim that the checkout is clean. An archive without Git administration
records the explicit `unversioned-archive` revision. A checkout whose Git metadata
cannot be read fails the build instead of inventing archive provenance.

HEAD, its resolved ref, packed refs, a worktree's `.git` indirection and linked
worktree `commondir` routing participate in build invalidation. The fingerprint
scheme remains `lila-program-cache-compiler-v3`; its existing framed source hash
does not change. A source fingerprint is distinct from a plain file SHA-256.

`CompilerDigest` admits exactly 64 lowercase hexadecimal digits. Commit identity
admits the 40- or 64-digit Git object domains. `CompilerIdentity` accepts only
those checked parts and an exhaustive revision kind. Its current producer binds
the embedded build identity to the loaded executable with standard SHA-256. On
Linux it opens `/proc/self/exe` so replacement of the public binary path cannot
substitute a different image. Hashing uses a fixed streaming buffer; unreadable
or changing image bytes reject evidence. There is no empty-image fallback.

## Snapshot admission and ownership

Schema 8 requires one complete `compiler_identity` object:

- `source_fingerprint_scheme` and checked `source_fingerprint`;
- tagged `source_revision`, with a checked commit for `git-commit`;
- checked `executable_sha256`.

The strict wire codec rejects absent/null parts, unknown fields, schemes and
revision kinds. Wire presence is retained until admission so legacy absence
cannot be confused with a current identity or a null assertion.

`SnapshotProvenance` owns both version and compiler identity. Current snapshots
cannot be constructed with a bare version and optional proof. Legacy schemas
4–7 have closed unbound identities; historical typed execution metadata remains
historical and does not manufacture a compiler binding. No automatic upgrade,
cross-version aggregate join, current resume or publication adopts that evidence.

Writers and resume require the actual running compiler before creating output
or admitting reusable execution records. Complete aggregate verification joins
every node to the parent compiler identity in addition to pins, backend, matrix,
exact cases, counts and taxonomy. Verified summaries, progress and generated
backlogs preserve the checked producer. Explicit read-only schema-8 comparisons
can compare different compiler builds; each full aggregate must still prove its
own internally consistent producer and complete case evidence.

## Publication consumers

`lila compiler-identity` emits the mandatory native JSON contract without
constructing an ECMAScript Realm or compiling JavaScript. Extra arguments reject.
Publication JSON and text retain the verified real-suite producer. Future
generated README output includes its build fingerprint and executing-image hash;
the current protected status block remains unchanged until authentic publication.

Publication-session schema 3 stores the native identity beside its observed
checkout/source/suite/configuration inventory. The compiler query's image hash
must agree with before/after observed executable hashes. Observed checkout source
bytes never stand in for embedded build inputs. A checkpoint response must carry
exactly one complete native identity matching the session; the progress writer
requires that identity and refuses a mismatch before rewriting its high-water
mark. Old schema-1/2 sidecars are retained and require a fresh family name.

The supervisor's filesystem observations and locks remain recovery/integrity
checks. They do not authenticate deliberate edits or make inputs immutable during
an invocation. Native snapshot validation and full publisher validation retain
their independent obligations.

## Authored controls and pending acceptance

Controls cover malformed identities, unreadable images, missing/null current
proof, legacy proof injection, different-compiler writers/resume, mixed parent
and child producers, and legitimate read-only different-build comparison. CLI
controls connect query, snapshot, JSON/text and generated README producer fields.
Both existing driver inventories retain their complete controls; additional
controls reject native identity mismatches before progress or publication and
preserve the previous manifest bytes.

These controls are source only. After all task source is finished, establish the
kernel limit, compile once, run the affected controls, then broaden sequentially.
Retain the historical baseline without rebinding it, and produce a fresh full
pinned aggregate/backlog/status pair before claiming current conformance counts.
