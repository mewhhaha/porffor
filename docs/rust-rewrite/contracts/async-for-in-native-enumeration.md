# Plain async ForIn native enumeration

`OrdinaryGeneratorForInIr` and `AsyncFunctionForInIr` enter the same native
ForIn pipeline through a private closed `ResumableForIn` owner. Its two actual
facades require the matching function execution kind. AsyncGenerator and
unowned loop graphs cannot acquire this complete owner. The original generator
pipeline preserves its head, key advance, original initialization, body and
cleanup instruction order.

Both owners retain the same four allocated invocation cells: the completed raw
head, the private native enumeration edge, the selected String key and the
StatementList value. Cursor admission requires the real resumable frame and
the exact allocated name and slot. Storage always loads the original
InvocationFrame's invocation environment, independent of current lexical hops.
No GC field, JavaScript iterator or parallel object representation is added.

The original head TDZ environment is entered or reattached before a resumed
head Await can inject rejection. The complete head compiles as an operand;
internal captures do not publish a loop StatementList value. After normal head
completion, the original TDZ is left and the outer environment saved before
boxing the raw source and creating the enumeration record. Nullish sources
retain their existing completed-cursor behavior. Creation observes no own-key
snapshot, and boxing happens once.

The sole existing native create/advance algorithm remains unchanged. Own keys
are captured lazily for the current object, the next-key index advances before
descriptor observation, and the original visited String set preserves
nonenumerable shadowing, Symbols, deletion and prototype behavior. Proxy
observations propagate their complete abrupt values through the original path.

Each advance retains its selected String and enters the original per-key
environment. The original eager Reference, binding or pattern initialization
then runs once. A resumed body reattaches that same record and never advances
the cursor or repeats initialization. The body uses a checked Iteration value
context from its exact allocated cells. Genuine source Empty-completion
wrappers suppress declaration prefixes and restore the previous value only on
Normal completion. Continue keeps the body value and returns to advance.

Pending Normal Await keeps the cursor and actual lexical records. Cleanup is
rebuilt before any resumed rejection; nested finalizers and pattern IteratorClose
complete before an outward transfer retires this cursor. ForIn itself performs
no IteratorClose. The original common exit retires the private edge and value
cells, commits the exhausted state, saves the restored outer environment and
dispatches the whole completion without converting it to a scalar error.
Direct and labelled async statements use only the checked carrier's actual
entry, advance and exit destinations.

Two authored compiler-artifact controls consume the paired real async ForIn
semantic fixtures. They validate emitted Wasm and follow the unchanged
four-field cursor, BindingCell, key-table and InvocationFrame topology, including
original cursor retirement, saved lexical edges and captured Reference cells.
These controls and source are uncompiled and unrun. Joined compilation,
focused regressions and broad verification remain required.
