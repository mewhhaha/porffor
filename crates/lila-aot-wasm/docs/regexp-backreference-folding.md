# Case-insensitive RegExp backreferences

The compiled RegExp matcher compares numbered and named backreferences using the
`i` modifier resolved at each reference. A capture retains its original text and
indices; modifiers surrounding the capture do not determine how a later reference
compares it. This applies in forward matching and inside lookbehind.

The instruction's second operand reserves bit 1 for resolved `ignoreCase`. Bit 0
continues to prove a numbered capture is nonempty and remains reserved for named
references. The matcher rejects other bits, invalid capture/name operands, and an
unavailable requested canonicalization table as corrupt compiled programs. Cycle
validation reads only the nonempty bit when deciding whether a reference consumes
input.

`lila-ir` owns the shared `RegExpCaseFolding` domain and its sorted nonidentity
canonicalization mappings. Character-set closure and backreference tables use the
same mappings. Unicode mode uses the vendored, verified Unicode 17.0.0 simple-fold
table (1,512 mappings); legacy mode uppercases one UTF-16 unit using Rust's Unicode
17.0.0 data, rejecting expansions and non-ASCII-to-ASCII mappings. Lone surrogates
are unchanged. This is simple canonicalization, so neither mode expands `ß` into
`SS`, and default Unicode folding does not use Turkic mappings.

The string pool emits one shared table for each required mode, with aligned
8-byte `(source: u32, canonical: u32)` rows. A module without case-insensitive
backreferences emits neither table. Requirements are recorded before program
deduplication because the existing program key omits flags: a legacy and Unicode
RegExp can share instruction bytes while needing different mapping tables. Static
programs and the existing finite runtime construction-candidate table use the same
collection path. The matcher performs binary searches in the selected table,
leaving characters unchanged when their keys are absent. No additional matcher ABI
parameter, runtime dependency, source evaluator, or general runtime RegExp parser
is introduced.

Comparison reads Unicode characters in `u`/`v` and UTF-16 units in legacy mode.
Both the captured span and the candidate are traversed in the active direction,
using private byte/UTF-16/surrogate cursors. A full match commits the candidate
cursor; a mismatch leaves the authoritative cursor for the existing choice and
capture rollback path. Undefined and empty captures consume nothing. Legacy
captures may start or end at either half of a surrogate pair; Unicode comparisons
cannot enter the middle of a paired scalar. Traversal does not assume that
canonicalization preserves UTF-16 or byte widths.

This change applies to patterns admitted by the existing AOT RegExp compiler.
Arbitrary runtime patterns and existing explicit unsupported pattern domains,
including case-insensitive finite class strings, remain separate implementation
work. The `.source` property retains the original pattern.

Focused verification is the IR `regexp_backreference_folding` integration target,
three `regexp_program_validation_tests` table/cycle tests, the native
`aot_regexp_backreference_folding` target, and the existing
`aot_regexp_backreference` regressions. These cover local modifiers, both directions,
undefined/empty captures, rollback, astral characters, legacy surrogate halves,
Unicode/legacy distinctions, shared-program table selection, original capture
text, lastIndex, and malformed reserved operands. Staging does not imply these
checks have run; the integrator records compilation and native results.
