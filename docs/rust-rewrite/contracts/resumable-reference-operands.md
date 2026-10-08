# Resumable Reference operands

Ordinary generators and mixed async generators use one consuming retained
assignment Reference owner. Its closed Identifier, ordinary property, private
and Super variants carry the original source reference through the complete
RHS. A selected write consumes that owner. Identifier logical skips retire the
captured environment handle; property skips perform no write.

The ordinary generator compound source owns its actual AST assignment target.
Base and raw-key suspensions precede the logical branch entry; only the chosen
RHS gets a continuation. The same source allocator and lowerer agree on all
selected, skipped and exit phases. Linear counters reject selected branches.
No key, private brand, HomeObject or receiver is reconstructed from the RHS.

GetValue precedes an eager compound RHS; numeric coercion follows the completed
RHS. Plain private and Super writes retain their original base without getting
the property or performing an early brand check. Super captures retain the base
selected before the RHS even if source code later mutates the HomeObject's
prototype. The original native Reference operations own the actual Get, Set,
strictness and abrupt completion rules.

The closed operand protocol also feeds existing private/Super reads, private-in,
numeric updates, dynamic import operands and Annex B call targets. Super reads
and updates retain the evaluated this value before a suspended key. Delete
retains its original ReferenceError without coercing the raw Super key. A lawful
sloppy call target evaluates the original callee and arguments, then rejects
before assignment RHS evaluation. Parser early errors remain authoritative.

Strict/sloppy Engine controls cover key conversion and accessor order, dynamic
Number/BigInt coercion, logical selected/skipped paths, saved Super bases,
borrowed receivers, private brand timing, GC and whole injected Return/Throw.
The separate sloppy fixture covers actual Annex B calls and interrupted
arguments. These controls are authored and unrun; the combined compile and
runtime checkpoint remains due.
