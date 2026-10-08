# Complete plain async classic loops

Plain async For, While and DoWhile source now uses the original complete classic-loop phase owner. The shared checked region carries a closed Generator/Async/AsyncGenerator protocol. Original protocol validators still consume every source point; a region with a different protocol, a missing phase, an aliased completion cell or an erased Await cannot construct a complete loop.

Initialization, test, body and update have separate ranges even when eager. DoWhile starts with its body. The same source planner feeds enclosing With, ForIn, Switch and nested iterator owners, preserving actual ForAwaitNext/Close kinds. Analysis retains the FunctionBody domain only for the actual source admitted by that plan. Value branches and declaration Empty completions remain owned by the complete loop.

The original native loop consumes these regions with the actual Async execution kind. Its lexical head and per-iteration records, retained whole StatementList value, labelled targets, pending finalizers and Await settlement remain shared. Pattern heads initialize the original analyzed binding cells after their awaited keys or defaults. Synchronous ordinary execution and mixed resource capabilities retain their own checked admissions.

The specialized awaited-While owner remains available only for a body without
suspensions or complete child phases. One source predicate selects that owner in
both planning and lowering. Its checked Await/If prefix now consumes the same
logical-assignment and optional Call Reference owners; class-owning conditions
select the complete classic region because they retain a separate class lifecycle.

Constructor controls and paired strict/sloppy Wasm fixtures are authored for phase order, per-iteration closures, head TDZ, GC, nested Switch/Try/loops, labelled Continue/Break, rejected head/test/update awaits and Return adoption after finalization. Complete non-iterator resource scope joins now share the original capability. No compilation, execution or conformance result is claimed for this dry source batch.
