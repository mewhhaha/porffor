# RegExp immutable program boundary

Status: normative for static program serialization and the Wasm AOT matcher.
This prerequisite does not implement runtime pattern parsing.

## Authority and format

`ValidatedRegExpProgram` owns private immutable bytes. `from_program` validates
opcodes, operand domains, instruction targets, capture IDs, sorted range
slices, unique named-capture ownership and non-consuming cycles. It derives
split counts for scratch sizing. `from_bytes` validates and reconstructs the
canonical encoding; backend deduplication crosses both boundaries before
publishing static bytes. Raw `RegExpProgram` builders are not accepted as
serialized storage.

The allocation begins with eight little-endian u64 words. The exhaustive
`RegExpProgramWord` domain owns their offsets:

| Word | Meaning |
| --- | --- |
| MagicVersion | `RGPB`, version 1 |
| ByteLength | Exact allocation byte length |
| InstructionCount | 24-byte instruction count, 1 through 4096 |
| CaptureCount | u32 numbered capture count, independent of instruction count |
| RangeCount | 8-byte range count, at most 65536 |
| SplitCount | Ordinary and progress split count |
| RepeatableSplitCount | Splits reachable through a control-flow cycle |
| NamedGroupTableOffset | Allocation-relative offset, or zero |

Instructions immediately follow the 64-byte header. Ranges immediately follow
instructions. An optional named-group table immediately follows ranges. With
no named groups, the range section ends at the allocation end.

The named table is `NRGT` version 2: magic, named-group count, total candidate
count and relative records offset 32. Each 24-byte record contains a packed
relative name offset/byte length, relative candidate offset and candidate
count. Records are followed by their contiguous u64 capture-ID sequences,
then their contiguous nonempty UTF-8 name bytes in record order. There are no
external name pointers or trailing bytes. All relative offsets are based at
the named table, so relocating the allocation changes no encoded byte.

Each static descriptor allocation is aligned to eight bytes. The object handle
packs its u32 allocation address in the high half and exact u32 byte length in
the low half. Zero means no compiled program. The sole object field is
`HEAP_REGEXP_PROGRAM_PAYLOAD_OFFSET`; all obsolete scalar program slots are
removed. Finite runtime candidate rows contain only source, flags, handle and
the existing closed entry-kind word.

## Read ownership and lifetime

The emitted layout decoder checks physical linear-memory reachability, header
identity, exact outer extent and canonical section positions before exposing
locals. Physical memory size is an outer safety check; it does not authorize
instruction, range or name reads. The matcher bounds each instruction fetch
and range slice by the descriptor's own counts. Both character classes and
word-boundary assertions consume the range count. Named records, candidates,
IDs and names are checked eagerly, including names with no backreference.
Named result properties and backreferences then consume these owned sections.

Capture count remains independent of instruction count: zero-repetition
lowering can erase capture instructions while preserving capture slots. The
wrapper retains the addressable-capture resource check. It validates the header
before deriving scratch requirements. The
matcher repeats that validation at its private call boundary. Corruption uses
the existing typed corrupt-program failure route, and scratch is rewound
before throwing. The shared seven-parameter helper type is retained: parameters
0, 2, 3, 4 and 6 carry handle, input, start index, match flags and scratch;
parameters 1 and 5 are reserved zeroes. Counts and section pointers cannot be
passed independently.

| Owner or copy path | Program ownership |
| --- | --- |
| Static pool and finite candidate rows | Aligned immutable allocation plus packed handle |
| Literal and constant/dynamic-selected initialization | Install one handle |
| RegExp constructor clone and `compile` | Copy or atomically replace the handle with source/flags initialization |
| Builtin `split` and `matchAll` clones | Share the handle; retain separate object state |
| `exec`, `test`, `search`, `match`, `replace` | Read the handle through the compiled-program wrapper |
| Object header inventory | Marks the packed program payload as a pointer-bearing field |
| Matcher scratch and result carriers | Never own or rewrite program bytes |

Static allocations remain before the heap start for the artifact's lifetime.
Rewinding matcher scratch cannot reclaim programs or their name bytes. The
pointer-bearing field inventory documents ownership; it is not a new runtime
collector or a claim that runtime-compiled allocations have been implemented.
A runtime compiler must publish immutable descriptors with a lifetime retained
by every sharing object before it can use this interface.

## Verification and remaining work

IR controls round-trip composed programs and reject corrupt header/section
extents, name aliases, candidate references and instruction operands. Backend
controls decode actual stored handles, verify alignment/deduplication and
header installation, and preserve case-fold table collection before program
deduplication. Native controls mutate embedded descriptor bytes without a
product corruption hook; they require a JavaScript Error without changing the lastIndex
value, including attempts to borrow name bytes as ranges. A separate native
control exercises clone, compile, named results/indices, replacement, matchAll,
split, failed compile and repeated execution across scratch rewinds.

Run the IR descriptor tests, the backend RegExp unit and structural tests, and
`cargo test -p lila-engine regexp_program_boundary_tests -- --test-threads=1`
after integration. These controls do not establish full RegExp conformance.
The runtime RegExp parser/compiler, shared grammar tables, closed opcode
construction, iterative parsing/validation and unified resource policy remain
separate dependencies. Computed patterns absent from the finite table continue
to use the existing fallback and can still fail as unsupported.
