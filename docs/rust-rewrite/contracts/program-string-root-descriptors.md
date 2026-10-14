# Compact program String root initialization

Status: private source-prepared on 2026-10-14. Rust types, Wasm validation,
native controls and original conformance execution remain unverified.

Every program-owned String still occupies its original immutable GC pool slot.
The runtime's String ordinals and UTF-16 segment remain a separate prefix.
Previously main emitted one complete construction and slot write per source
String. A large finite corpus therefore expanded main's instructions even when
the JavaScript statements themselves had a compact representation.

The program now carries one checked passive descriptor image. Each 12-byte row
contains three little-endian u32 words: absolute pooled ordinal, byte offset
within the program UTF-16 segment, and exact UTF-16 code-unit length. The
constructor admits only program ordinals, aligned in-range spans and a
representable complete byte extent. Map traversal never determines a slot's
identity. Duplicate String values keep the already collected ordinal, including
source values equal to runtime-owned names and the empty String.

Program passive segment0 remains exact UTF-16; segment1 remains the program's
static data; segment2 contains only these descriptors. Neither the runtime
image nor the static-data address space receives the descriptor bytes. Runtime
pool construction remains its existing separate helper, with the same schema,
function/data domains and shared String values. Other passive owners use their
own segment and may not reinterpret these words as program string contents.

Main creates a rooted private descriptor ByteArray, then executes one bounded
row loop. The table wrapper exposes only read methods. Its row reader yields
an ordinal and opaque validated UTF-16 span locals, which the StringConstruction
producer accepts for array.new_data from program segment0. Publication consumes
that producer and writes the immutable String into its original rooted pool
slot. The temporary descriptor table and each transient string root are retired
after their consumers; no JavaScript reference to descriptor storage exists.

NUL, empty data, astral pairs and isolated surrogate units require no special
code path. Bounds and lengths describe raw UTF-16, not UTF-8 or source spelling.
No literal limit, candidate pruning, body-budget change or source-shape dispatch
is introduced. This patch changes only String root initialization; independent
array literal compaction has a different owner.

The private controls include a descriptor unit matrix that decodes actual rows
across the runtime/program boundary, and an encoded-body test for 16 versus 2048
source Strings while main's JavaScript operations remain fixed. The latter
validates emitted Wasm, requires one descriptor construction and one String
construction in main, checks descriptor/data growth, limits actual main growth
to 64 bytes, and requires the complete linked runtime bytes to be identical.
It avoids measuring array-literal emission as if it were String initialization.

Two native controls execute sloppy and strict scripts through Wasm AOT and
check explicit completion and output. They cover long/empty/NUL/paired/isolated
units, many unique source roots, equality with dynamically created strings,
GC retirement, borrowed created-Realm methods and overlapping runtime names.
The original pooled-string encoded-data control keeps its UTF-16 assertions
and also identifies the distinct descriptor data segment.

Actual measurements and original large Unicode executions must establish the
improvement. Source preparation alone does not eliminate a historical failure
or close full-suite acceptance.
