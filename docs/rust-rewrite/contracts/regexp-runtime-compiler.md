# Runtime RegExp compiler

The AOT runtime compiles computed legacy Pattern strings directly in emitted
Wasm. This is a RegExp-only compiler feeding the existing ordered matcher; it
cannot parse or execute JavaScript source, and calls no host compiler or matcher.

## Grammar and outcomes

The runtime grammar composes sequence, ordered alternatives, numbered captures,
noncapturing groups, ordinary classes and ranges, dot and boundaries, greedy or
lazy finite/unbounded repetitions, numbered/forward/unmatched references and
positive/negative lookahead and scoped i/m/s modifiers. Flags d/g/i/m/s/y are
admitted. UTF-16 units include lone surrogates; an ungrouped astral pair's
quantifier applies only to its trail.
Nullable optional repetitions use the existing progress-choice/check protocol,
including capture clearing and rollback. Parsed numbered references permit an
unmatched capture to consume no input; a nonempty capture body does not prove
participation. Legacy case closure uses the same
retained canonicalization table as static compilation and backreferences.

Unicode modes u/v, named groups/references and lookbehind are explicit runtime
capability gaps. Unicode-property syntax belongs to the
unsupported Unicode modes; legacy p/P/k identity escapes retain legacy meaning.
The static compiler and candidate table still serve their supported grammar.
A static rejected row remains SyntaxError; an unsupported row or miss calls the
runtime compiler. Only an explicit capability result retains the transitional
small-pattern fallback. This is not whole-RegExp or full-suite conformance.

The private helper ABI takes seven i64 parameters: already-coerced source and
flags payloads, then five zeros. Its four results are descriptor handle, closed
status, UTF-16 source offset and typed detail. Status words are Compiled0,
SyntaxError1, Unsupported2, ResourceExhausted3 and CorruptProgram4. Only Compiled
carries a nonzero handle. The wrapper routes syntax errors to SyntaxError,
resource failures to RangeError, and internal invalid programs to Error, using
the current function's Realm. Unsupported remains distinct from no-match.

## Ownership and bounds

The pure helper owns one checked region above its entry heap checkpoint. It
validates flags and decodes WTF-8 into UTF-16 units, counts capture syntax before
decimal escapes, and builds a flat tree whose parents precede children. Parent
links form the parser group stack; lowering uses bounded three-word task records.
No source-dependent Wasm recursion or JS allocation occurs inside the compiler.

Each group owns its inherited modifier state in the arena. The parser restores
that state from the group owner before reading each term or alternative; captures,
noncapturing groups and lookaheads preserve the same lexical boundary. Scoped i
controls literal/class case closure and reference operands; scoped m/s use the
existing Inherit/ForceOn/ForceOff anchor/dot operands. No matcher flag stack or
public flag mutation is involved. Prefixes admit only i/m/s, at most one dash and
no duplicate or overlapping flags. An empty remove list after added flags is
valid; an entirely empty modifier list is SyntaxError. Both parsers share the
closed modifier alphabet and masks. Changing global i/m/s flags still recompiles
a clone because atoms outside local scopes inherit those global flags.

A postorder width pass rejects expansion before iteration. Full decimal spans
are compared before narrowing, so reversed huge bounds are SyntaxError. Finite
oversized bounds remain distinct from unbounded repetitions; state-free atoms
with zero emitted width can discard repetition without iterating the count.
Programs retain the existing 4096-instruction/65536-range limits. Instruction
operands, successors, progress and non-consuming cycles use shared closed opcode
facts. Exact repeatable-split counts are derived with bounded graph traversal.

Workspace address overflow and failed memory.grow return typed resource outcomes;
the generic trapping HeapAlloc helper is not used. Every failure clears the
owned region before restoring the entry checkpoint and allowing the wrapper to
allocate an Error. A failure before workspace allocation clears an empty range.
Successful instructions and ranges are serialized into the existing RGPB relative
descriptor, checked, then compacted to the entry checkpoint with memory.copy.
The compiler clears only the tail after the aligned descriptor before releasing
that storage. The descriptor and all pre-checkpoint allocations remain intact.
This zero-memory contract matters because ordinary allocation leaves some record
slots at their initial zero value; rewinding dirty parser storage could make a
fresh capture array appear non-extensible. No published descriptor points into
parser, task, source, folding or scratch storage. Later matcher scratch checkpoints occur
after publication; clones can share the handle with independent lastIndex.

A failed compile preserves the receiver's old program/source/flags/lastIndex.
Successful installation precedes the final strict lastIndex Set, which may throw
after the new program is installed. Constructor source/flags/program snapshots
remain coherent across coercion callbacks. Constructor/split/matchAll share an
existing program only when compilation flags agree; changing only g/d/y retains
supported static Unicode/named programs without requiring runtime recompilation.

## Verification boundary

`runtime_regexp_compiler_tests` adds test-only exports to copied artifacts and
calls the actual emitted compiler. Host imports trap if called. Returned bytes
must roundtrip through ValidatedRegExpProgram and match static descriptors for
selected common grammar. Separate controls cover resource rollback including
physical memory-growth refusal, zeroed released memory and allocator reuse,
descriptor persistence, computed pattern matches,
syntax/throw identity, recompile state and changed clone flags. Computed matching
fixtures use code-unit arrays and composed grammar fragments; literal regression
controls compile in a separate artifact so candidate rows cannot mask the runtime
parser. Scoped-modifier controls separately cover lexical restoration, captures,
lookahead, nullable repetitions, class closure, reference-site folding, rejected
prefixes and changed-global-flag clones. Descriptor comparison includes nested
m/s overrides and restored sensitive/reference operands. The original immutable
descriptor lifetime fixture remains unchanged.

Computed capture-array controls also allocate and mutate fresh arrays and objects
after successful, syntax-failing and resource-failing compilation; retained
programs must continue matching after later compiler invocations.

These new checks are staged for the next verification checkpoint. No full-suite
status is inferred from them; u/v/named/lookbehind runtime gaps remain
visible owners rather than ignored or expected-success tests.
