# Duplicate named-group pattern

Status: implemented and verified for the two specialized duplicate-named-group
RegExp patterns in the String matcher.

## Boundary

The private `builtins/string/duplicate_named_group_pattern.rs` child alone owns
the capability-free
`DuplicateNamedGroupPattern::{AlternativeCaptures, IteratedBackreference}`
domain and the pattern-parameterized emitter. One borrowed exhaustive match
owns the complete candidate/result table: the alternative-capture pattern
recognizes `abc` and `ad`, while the iterated-backreference pattern recognizes
`aac`. No Boolean, default or wildcard selection remains.

The String parent can request only the two semantic matchers. It cannot name or
construct the raw pattern, import the child policy, or call the raw emitter.
The child wrappers bind each semantic operation to exactly one variant before
entering the shared algorithm.

The shared initial `null` result and the `has_indices` local remain outside and
flow unchanged into every result-table emission.

## Durable evidence

`duplicate_named_group_pattern_structure.rs` recursively pins the private
module and exact two variants, lack of convenience capabilities, zero parent
raw-policy names, the single exhaustive match, all three candidate/result rows,
`has_indices` forwarding, the common `null`, both semantic producer mappings,
six child domain mentions and a private definition-plus-two-call census.

The moved four-line domain, 80-line emitter and combined 84-line semantic owner
retain SHA-256
`38391f8c3eaadf1cd997b13fffba38dccf8a017955d3bb75b48eb3e587af7280`,
`bcd1693a0ff5292fa826e8449162eb85e7dedcec857aa0b76ef7b9d5c3bdd387`
and
`3a5aa0f6afbd361cf6e88724d0c2e4a4bb1f559b5b0a81a15affd68c455063ee`.
The 35-line semantic-wrapper selection has SHA-256
`d495b7a0b0a0ee0cfa8d9f21628b28c543321764a0bf5a374de7e05ee8349a15`.
The resulting 20,883-line parent and 125-line child have SHA-256
`6a8e1b8fb5d7f05b0bfaba1d8196dab577aac30a5a65cbde37c10291321cc984`
and
`9cb88aa5ee221e66911a1070062e7e15242aaa91585562dbfba51d4c709ee560`.
The former raw caller selections had SHA-256
`7d46d1145d159fdd79dd837b88c614afce6b1802c45824763db1784888b6fc7c`
and
`ebcd8f01c1dffd8337d5f3d881ee12b5806423902e095fb745775f02c3e087dd`;
the narrowed seven-line calls have SHA-256
`df2dbb7ba47072782b0eb835fa1512bb3905c6396c89b78436606693ce0aaccd`
and
`354f3a010a275a988edfee244f03b57064e7d1bbd72aa2febcdd2e149683a50a`.

## Verification

At the Batch AC shared checkpoint, `cargo xc` is green, the bounded structure
target passes `3/3`, the exact CLI fixture passes `1/1`, and the exact String
match ordinary-groups and indices-groups leaves pass all `4/4` variants with
every failure bucket at zero. Batch AC preserves the raw emitter byte-for-byte
and changes only parent call spelling to semantic wrappers. The semantic golden
was not rerun.

## Deferrals

This contract does not generalize RegExp parsing, named-capture allocation,
backreference execution or indices, or complete String, RegExp, T18 or T19.
