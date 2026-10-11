# Native Intl cross-host evidence

The independent `Native Intl reproducibility` workflow captures Linux and macOS
results from the same clean checkout. It runs the complete Intl library,
calendar-domain and sixteen-calendar native test targets with one test thread,
including ignored tests. Each scope must have a nonzero complete passing census
with zero failures, ignores or filtered tests. These scopes exercise the pinned
native formatters as well as image admission.

The capture builds the default release CLI, records its complete compiler and
executing-image identities, exports explicit Minimal and Conformance bundles,
and re-admits each export through `intl inspect`. Retained evidence contains
every command, exit code, transcript digest, native count, actual bundle bytes
and canonical provider identity. The source fence binds the checkout commit and
Cargo lock; executable identity must match both the checkout and actual image.
An interrupted capture remains incomplete.

The comparison requires two different operating systems, equal source and
compiler-source fingerprints, complete equal native censuses, and byte-for-byte
equal exports and admitted identities for both profiles. Machine executable
hashes may differ. Full provider identity includes the ABI, profile, services,
capabilities, actual data digest and pinned data versions. A filename or matching
profile label is insufficient.

Use `python3 -B scripts/check-intl-cross-host.py capture --output NEW_DIRECTORY`
on each native host, then `python3 -B scripts/check-intl-cross-host.py compare
LINUX_DIRECTORY MACOS_DIRECTORY --output NEW_COMPARISON.json`. Output paths must
be new. The CI jobs retain unsuccessful capture evidence and require both native
jobs before comparing their artifacts.

Eleven synthetic artifact-admission controls pass in the staged tooling. They
exercise rejection of partial scopes, changed data, stale compilers, missing
commands and single-host evidence. They do not claim a native or cross-host
result. Actual independent-host capture and comparison remain pending.
