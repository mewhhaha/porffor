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


The current `import-source` grammar repair retains this constructor split.
`import source from from 'target'` is a source-phase import whose local binding
is named `from`; `import source from 'target'` is an ordinary default import
whose local binding is named `source`. The parser previously excluded `from`
from source bindings and rejected the valid former declaration during parsing,
before the unresolved host request could become the existing TypeError job.
Lookahead now checks the actual unescaped delimiter for this contextual case.
No diagnostic constructor or load/dependency stage changes. Existing paired
load/link controls retain their IDs, constructors, stages and error ownership,
and now also include both original contextual source binding spellings.

The full pinned language `source-phase-import/import-source.js` Script and its
three original targets/ensure-link fixture are exercised in both Script modes.
The companion grammar controls retain escaped binding names, reject escaped
modifiers/delimiters and reserved bindings, preserve comments/line terminators
and ordinary default/named/namespace imports, and distinguish unresolved host,
actual parse, ordinary link and resolved-JavaScript source-representation
failures. These controls are source-authored and require actual execution.
