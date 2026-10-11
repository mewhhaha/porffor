# Complete conformance source update — 2026-10-11

The vendored Test262 source is updated as a complete official tree to upstream
commit `2e0a56762801e275a9fdf96dc49d90ba0cddcf63`, whose content tree is
`6a4268a9354a545d41c3b62efebc478ed8c521f4`. The previous content tree was
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. The update contains 553 added,
23 removed and 747 modified files, with 57,013 total files. Removed
Atomics.pause argument checks belong to this complete upstream update:
ECMA-262 removed that optional parameter in June; no product tests are
selectively excluded.

Execution metadata now embeds exact ECMA-262 source revision
`5345883164f463e87f8b40aca4956157ecba8783` instead of an unversioned draft label.
The source-pin schema requires all three distinct lowercase 40-digit Git
identities and rejects unknown, missing or duplicate fields. The guard hashes
actual vendored bytes and modes into Git trees and verifies the declared content
tree. POSIX uses actual executable bits; Windows binds complete worktree paths
to index modes and still hashes actual bytes, including Git symlink stubs.
The outer Git attributes preserve upstream newline bytes on every host. The
vendored upstream attributes and content are unchanged. Historical snapshots,
backlog and published counts retain their original identities.

The embedded Wasm assertion, both resource preambles and property helper now
contain the complete current upstream functions, including formatting globals,
Test262Error's ordinary-call fallback and accessor verification. The SpecExec
preamble is synchronized as well. Byte-parity tests bind every synchronized
section to the upstream harness. New native controls run both embedded and
host-only profiles in sloppy and strict scripts. Upstream nativeErrors.js and
testIntlNumberFormat.js load through normal recursive harness discovery.

The archive is 9,587,030 bytes with SHA-256
`f8e99225426343b50672313f46c4c0b5e0f228f5d10e1ea6782c14ef34784cb4`.
The audited ECMA source has SHA-256
`5194892f5a5f595962929b7099e028b1db44c8b68c44f4ce20f77b266507d57d`.
Source-only discovery with the preceding product image found 102,994 execution
IDs and 778 Atomics IDs; these are inventories, with no execution verdict.
The current Rust census independently observes the same 102,994 execution IDs
across 53,616 physical test files. Host-free/dormant/active buckets contain
52,805/495/316 files and 101,422/975/597 executions; host script modes are
799 sloppy, 762 strict and 11 module executions. Agent requirements remain
109 files and 218 executions. The old census assertion fails with this complete
new tuple; its expectation is now updated to the independently observed values.
All nine final source/host/census checks pass, including actual oracle global
accessors and typed missing-capability rejection. The complete pinned-harness
integration target passes all five native tests in 42.50 test seconds, covering
both harness profiles in sloppy and strict scripts. Joined all-feature/all-target
types pass in 60 watched seconds; all twelve repository/source guards pass.
All 443 discovered tooling tests pass in 137.651 test seconds, and the separate
publication driver passes 45 tests in 22.940 seconds: 488 distinct tests with
zero failures, errors or skips. The [JSON receipt](conformance-source-pins-20261011.json)
records frozen sources and executable/transcript hashes, and preserves the
original census mismatch and initial narrower checkpoints. No full upstream
execution verdict is claimed.

New chunks/windows and Error stack-accessor cases are proposal tests whose
algorithms are specified by their upstream metadata and linked proposals; they
are not clauses of the pinned ECMA-262 source. Their product implementation and
fresh complete workspace, differential and pinned execution remain pending.
Refresh provenance with
`python3 -B scripts/check_conformance_source_pins.py`; publish conformance counts
only through the existing verified status publisher.

Review of the first focused checkpoint found that replacing the SpecExec sta
section had also removed its appended native-host facade. That oracle adapter
is restored byte for byte behind an explicit resource/host boundary. Upstream
resource parity checks the complete upstream prefix; separate host controls
exercise the adapter. Its restored source passes the final nine-check selection.

An auxiliary worker link failed with SIGBUS under disk pressure before the
harness retry. A later type check found generated inputs removed by cache
cleanup while their Cargo stamp remained valid. Invalidating that stamp forced
fresh current-runtime generation; the final unchanged-source type check passes.
Private receipts retain both build failures and all cleanup inventories.
