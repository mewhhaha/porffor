# Indexed ordinary properties — 2026-10-10

Ordinary named-property lookup and deletion use a registered typed helper and
an auxiliary hash index. The ordered property table remains responsible for
observable key order and rooted completion snapshots.

Buckets contain an ordered entry position plus one; zero marks an empty
bucket. A deleted entry leaves a tombstone so colliding keys remain reachable.
New entries can reuse tombstones in the index, but append to the ordered table
for correct String/Symbol re-addition order. The power-of-two bucket array has
twice the ordered capacity. Growth copies the ordered prefix and rebuilds the
index from live entries, using each entry's retained full hash. Capacity checks
precede allocation and doubling.

Lookup compares full hashes and then complete semantic key equality. String
hashing reads exact UTF-16 code units, including unpaired surrogates. Symbols
use their stable identity field, shared with Map/Set hashing; descriptions and
GC addresses never define identity. Collection Number, BigInt and object-key
hashing retains its existing rules. All temporary entries, keys and replacement
arrays remain rooted through their last use.

The shared append owner checks every caller, including intrinsic installation,
for an existing key before creating a new entry. Repeated publication replaces
that entry's descriptor and preserves its original position. This makes key
uniqueness an invariant of the physical mutation boundary.

The new native control deliberately creates hash collisions, deletes collision
heads and middles, grows the table, re-adds keys, checks nonconfigurable
descriptors, distinguishes exact UTF-16 strings, and mixes property-first with
Map-first Symbol hashing. The existing artifact-size control requires one
shared lookup body with a real nonself consumer, within its original limits.

The frozen run passes all 969 backend controls and 193 native controls, with
zero failures or ignores. Native coverage includes the complete 150-control
GC-entry target, UTF-16 keys, Arguments descriptors, rooted snapshots, global
references and execution environments, HasProperty and Proxy descriptors.
All-feature/all-target workspace types, the CLI build, formatting, module
boundaries, task accounting and shortcut inventory pass. A fresh product fake
run passes all 191 exact execution IDs over 190 files, including all 187 raw
Wasm-safe IDs, with zero failures/timeouts in 206.111 watched seconds. The
snapshot's executable digest matches the tested CLI.

The unchanged three-round 4,096-key diagnostic passes in 88.424 seconds of Wasm
execution, compared with 105.292 seconds for geometric storage alone and
110.275 seconds at the original baseline. These single samples do not establish
a general speedup. This batch does not include a fresh full engine/CLI sweep.
Commands, source and executable hashes, exact case checks and cleanup records
are in the [machine-readable receipt](property-index-20261010.json).
The earlier [geometric-storage receipt](property-storage-20261010.md) retains
the before/index-free evidence and fixed diagnostic source. The historical
September 30 aggregate with 5,365 failures remains unavailable; this change
neither substitutes for that replay nor changes task states or publisher totals.
