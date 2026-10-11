# NumberFormat source-generator reuse — 2026-10-11

The pinned NumberFormat producer avoids repeating locale currency overrides
across its 78 decimal systems. Bounded caches also reuse interned literal and
signed-pattern IDs, parsed UnicodeSet IDs and formatted immutable paths.
Successful parent lineages are reused only after their entire chain is consumed
and validated; incomplete or failed chains never become cache entries.

The caches preserve first-occurrence pool insertion and source selection.
Unicode properties and pooled IDs stay bound to their owning Extractor.
Original primary manifests, source validation, admitted locale/numbering domains
and generation algorithms remain unchanged.

Two isolated source-only samples cover eight varied locales: Arabic regional,
Basaa regional, US English, Swiss French, Central Kurdish, European Portuguese,
Japanese and root. The original producer takes 20.705 seconds. The first
memoized variant takes 18.565; the final one takes 15.145. Every diagnostic data,
provenance, policy, sample and coverage file is byte-identical, including gzip
streams. These samples establish this probe's reuse, not a general benchmark or
complete primary-source reproduction.

Four focused controls pass for separate ID/property owners, literal versus
compact tokens, partially consumed cyclic chains and failed-chain recovery.
The complete tooling suite also passes all 432 tests with zero failures,
errors or skips in 140.322 test seconds and 143 watched seconds.
The [receipt](intl-generator-memoization-20261011.json) retains exact hashes and
the two interrupted full-reproduction attempts. Neither interrupted run
supplies a complete reproduction verdict. The first wrapper hid producer
progress; the corrected wrapper streams it with the same 900-second stall
budget. The second run was stopped at 151/1,082 profiles to measure this change.

Complete pinned payload reproduction, Rust workspace/runtime, differential
and current full Test262 acceptance remain required. Canonical conformance
totals and task states are unchanged.
