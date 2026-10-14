# Dynamic module host-load rejection kind

Dynamic import admission carries a closed rejection kind independently of its
load/dependency timing stage. A parser or module-link diagnostic remains an
intrinsic SyntaxError. A host request that cannot resolve a declared record
becomes an intrinsic TypeError, matching the existing missing-request fallback.
`ModuleUnresolved` identifies that host failure; no diagnostic text is parsed.

A closure's host loading finishes before linking. If the ahead-of-time graph
reports both a missing host target and a missing export, the host-load failure
therefore wins over the later link diagnostic, irrespective of unit ordering.
The owning request and its message survive into the asynchronous import job;
invalid closures do not evaluate. Public mutable TypeError or SyntaxError
constructors cannot affect either error.

A direct `import.source()` of an already loaded ECMAScript record still rejects
with SyntaxError because that record has no source representation. It does so
before loading that record's dependencies. Parse failures also remain distinct
from host load failures; this repair never converts every ModuleLoad-stage
rejection to TypeError. Static compile-time diagnostic taxonomy is unchanged.

The prior native dynamic load/link control retains its ID and all inputs while
asserting the distinct normative type for the unresolved target. New paired IR
and native controls cover unresolved source binding plus a missing export,
parse failure, ordinary missing export and direct source-phase rejection, with
mutated public error constructors. The two historical import-source execution
IDs and all affected previous passing IDs require replay before acceptance.
