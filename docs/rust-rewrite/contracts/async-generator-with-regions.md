# Mixed async-generator With source regions

The actual FunctionBody With source owns complete head and body ranges through
the same checked mixed allocator as classic loops and If. Eager With still has
two phases. The source head completes outside the new Object Environment Record;
the lowerer applies ToObject once and publishes its exact result in an existing
activation cell before entering the original analyzed With record.

Analysis mints a private mixed With token only from the checked actual AST source
in the FunctionBody domain. Dispatch checks that source identity and consumes the
token. Foreign iterator and resource suffixes keep their original routing. Source
and analysis consume one physical resource-suffix predicate. A source-body shape
proof excludes suspended foreign iterator owners, for-await implicit phases and
resource heads or suffixes before the new With source is minted. Eager foreign
forms retain their original algorithms; nested callable bodies are separate.

The body uses a checked mixed region with the original With environment chain.
It wraps the complete source statement, so a nested Block keeps its own lexical
environment. Nested checked With, classic loop, If and Try phases share the same
source tape. The With body's actual scope ancestry contributes to the private
resume-environment certificate; its head stays outside that record. There is one
GC object/environment representation and no new interpreter or fallback path.

Original Identifier Reference selection precedes a suspended RHS. Plain writes
use the write-only retained Reference; compound/logical writes retain the original
GetValue and the same selected ObjectER or outer binding. Skipped logical arms
release that Reference. PutValue uses the original operation, including its own
binding rechecks, rather than resolving With or unscopables again. Whole values
remain rooted through Yield, Await, promise adoption, queued completion and GC.

IR controls inspect actual phase tape, original hidden row, separate nested
lexical cells, eager ownership and honest foreign/strict refusals. Real Wasm-AOT
fixtures cover head conversion, captured closures, nested phases and record
cleanup, original Identifier writes, skipped logical RHS, injected whole Throw
and Return through awaited/yielding finalizers. Positive With fixtures are sloppy;
strict With is an early-error control. These controls are authored but unrun until
the complete source batch reaches its mandatory verification checkpoint.
