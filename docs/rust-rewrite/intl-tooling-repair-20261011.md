# Pinned Intl tooling repair — 2026-10-11

The complete Python tooling suite now passes **428 tests, zero failures, errors
or skips**. Its first run at `53307154f` reached only 366 methods and recorded
five failures and 26 errors. Missing primary source files prevented several
test classes from running; the remaining failures exposed stale identities.
Both original and repaired logs have hashes in the [receipt](intl-tooling-repair-20261011.json).

Two genuine inputs were missing from the cloud checkout because `*.txt` ignored
them. The restored Unicode17 `UnicodeData.txt` is 2,198,209 bytes and matches
SHA-256 `2e1efc1dcb59c575eedf5ccae60f95229f706ee6d031835247d843c11d96470c`.
CLDR47 `scriptMetadata.txt` is 13,031 bytes and matches
`f765a79559429de5cf1c3346df9eebb73be5e46cef4489df7052b878e230bb0b`.
These are the existing manifest/generator pins; no authority or validation rule
changes. Unicode's official GitHub mirror supplied the first file because the
cloud network policy denies unicode.org. The second comes from the exact pinned
CLDR commit. Explicit ignore exceptions keep both inputs in fresh checkouts.
Python bytecode is ignored separately.

The ordinary source-only generators refresh current Intl production and locked
dependency identities. Named-zone identity precedes the locale-zone kernel;
DateTime identity precedes its locale consumers; List precedes Duration.
Profile payloads remain byte-identical. All fourteen identity/alias generator
checks and nine formatting/repository guards pass. Complete pinned payload
reproduction is still running; its completed scope needs a separate receipt.
The first wrapper was deliberately stopped because it buffered the generator's
progress. Its replacement streams that progress with the original 900-second
stall budget; the stopped attempt supplies no complete reproduction verdict.

The full tooling run takes 136.894 test seconds / 140 watched seconds under
the inherited 32 GiB cloud cap and four-CPU quota. Cargo and libtest remain
serial. No timeout, expected-failure policy or assertion is weakened.
Complete Rust workspace/runtime, differential and current full pinned Test262
acceptance remain required. Task states and canonical conformance totals are
unchanged.
